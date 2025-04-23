// use crate::hypergraph::HyperGraph;
use hypergraph::hypergraph::HyperGraph;

#[cfg(feature = "hmetis")]
use hmetis_r;
#[cfg(feature = "kahypar")]
use kahypar_r;
#[cfg(feature = "mtkahypar")]
use mtkahypar_r;
use std::fmt;
use std::os::raw::{c_float, c_int, c_uint, c_ulong};

#[derive(Copy,Clone)]
pub enum Partitioner {
    H,  // hMetis
    M,  // METIS
    K,  // KaHyPar,
    MT, // mt-KaHyPar,
    D,  // Dumb Partitioner
    X,  // Meta-partitioner -- try multiple versions
}

#[derive(Copy,Clone)]
pub enum Objective {
    C, // Minimum cut
    D, // Minimum sum-of-degrees SOED
    K, // K minus 1
}
#[derive(Copy,Clone)]
pub struct Metapartitioner {
    pub num_starts: usize,
    pub k: usize,
    pub partitioner_type: Partitioner,
    pub objective: Objective,
    pub seed: u64,
    pub imbalance: f32,
}

struct VW {
    weight: c_int,
    index: usize,
}

impl Metapartitioner {
    /// Creates a new metapartitioner context, setting the
    /// partitioner type to the default for the compile
    /// configuration.  On Macs, this is likely hMetis
    /// (using the hMetis 2.0 library built for x86_64-apple-darwin).
    /// On a Linux installation, the default will be mt-KaHyPar.
    pub fn new() -> Metapartitioner {
        let mut mp = Metapartitioner {
            num_starts: 1,
            k: 2,
            partitioner_type: Partitioner::K,
            objective: Objective::C,
            seed: 8675309,
            imbalance: 0.01,
        };
        // Initial default is kahypar, if available
        #[cfg(feature = "kahypar")]
        {
            mp.partitioner_type = Partitioner::K;
        }
        // hMetis is preferred to KaHyPar, if available.  hMetis
        // handles fixed vertices well (and for applications in circuit
        // placement, that's essential).
        #[cfg(feature = "hmetis")]
        {
            mp.partitioner_type = Partitioner::H;
        }
        // If mt-KaHyPar is available, prefer that over both
        // hMetis and KaHyPar.  mt-KaHyPar handles fixed terminals.
        // Some experiments will be conducted to try to compare these
        #[cfg(feature = "mtkahypar")]
        {
            mp.partitioner_type = Partitioner::MT;
        }

        mp
    }

    /// Returns true if the specified partitioner is compiled
    /// into the library, false otherwise.
    pub fn available(partitioner: &Partitioner) -> bool {
        match partitioner {
            #[cfg(feature = "kahypar")]
            Partitioner::K => {
                return true;
            }
            #[cfg(feature = "mtkahypar")]
            Partitioner::MT => {
                return true;
            }
            #[cfg(feature = "hmetis")]
            Partitioner::H => {
                return true;
            }
            Partitioner::D => {
                return true;
            }
            _ => {
                return false;
            }
        }
    }
    /// Returns a char string for the readable name of the partitioner.
    pub fn name(partitioner: &Partitioner) -> &'static str {
        match partitioner {
            Partitioner::K => {
                return "KaHyPar";
            }
            Partitioner::MT => {
                return "mt-KaHyPar";
            }
            Partitioner::H => {
                return "hMetis";
            }
            Partitioner::D => {
                return "dumb";
            }
            _ => {
                return "Unknown";
            }
        }
    }

    /// Partitions the graph, using the partitioner and k values indicated.  Will
    /// run multiple starts, selecting the best for the supplied objective.  Returns
    /// the bin assignment as the first vector, the total weight in each bin
    /// in the second, and the cost based on the selected objective as the third
    /// value.
    pub fn hg_partition(&self, hg: &HyperGraph) -> (Vec<c_int>, Vec<c_int>, usize) {
        match self.partitioner_type {
            #[cfg(feature = "kahypar")]
            Partitioner::K => {
                return self.hg_ka_partition(hg);
            }
            #[cfg(feature = "hmetis")]
            Partitioner::H => {
                return self.hg_hm_partition(hg);
            }
            #[cfg(feature = "mtkahypar")]
            Partitioner::MT => {
                return self.hg_mtka_partition(hg);
            }
            Partitioner::D => {
                return self.partition_dumb(hg);
            }
            _ => {
                println!("Partitioner not supported");
                return (Vec::new(), Vec::new(), 0);
            }
        }
    }

    pub fn hg_ka_partition(&self, hg: &HyperGraph) -> (Vec<c_int>, Vec<c_int>, usize) {
        let mut partition = hg.part.clone();
        // println!("Balance {}", self.imbalance);
        let mut fixed = None;
        for i in 0..hg.part.len() {
            if hg.part[i] != -1 {
                fixed = Some(i);
                // println!("Fixed vertex {}", i);
            }
        }
        unsafe {
            kahypar_r::partition(
                hg.vtxwt.len() as u32,
                (hg.eind.len() - 1) as u32,
                hg.hewt.as_ptr(),
                hg.vtxwt.as_ptr(),
                hg.eind.as_ptr(),
                hg.eptr.as_ptr(),
                partition.as_mut_ptr(),
                self.k as i32,
                self.num_starts as i32, // Passes
                self.seed as u64,       // Seed
                self.imbalance as c_float,
            );
        }
        // Check to see if the fixed cells have flipped sides
        if fixed.is_some() {
            if partition[fixed.unwrap()] != hg.part[fixed.unwrap()] {
                // KaHyPar seemed to be flipping some fixed vertices(?).
                // Or it may have been another bug somewhere.
                // If it looks like the partition has been mirrored, print a
                // warning message.
                println!("**** KaHyPar may have flipped the partition ****");
                for i in 0..partition.len() {
                    partition[i] = 1 - partition[i];
                }
            }
        }
        let (bins, cut) = self.evaluate(&hg, &partition);
        (partition, bins, cut)
    }

    pub fn hg_mtka_partition(&self, hg: &HyperGraph) -> (Vec<c_int>, Vec<c_int>, usize) {
        let mut partition = hg.part.clone();
        // println!("In the MT interface");
        // In mt-KaHyPar, bot the eptr and eind are long ints,
        // compared to KaHyPar, where the eptr is an unsigned int. Thus,
        // need to create a ulong vector
        // println!("Balance {}", self.imbalance);
        let mut eptr_ulong = Vec::new();
        for v in &hg.eptr {
            eptr_ulong.push(*v as c_ulong);
        }
        unsafe {
            #[cfg(feature = "mtkahypar")]
            mtkahypar_r::mtkahypar_partition(
                hg.vtxwt.len() as u32,
                (hg.eind.len() - 1) as u32,
                hg.hewt.as_ptr(),
                hg.vtxwt.as_ptr(),
                eptr_ulong.as_ptr(), //hg.eptr.as_ptr(),
                hg.eind.as_ptr(),
                partition.as_mut_ptr(),
                self.k as i32,
                self.num_starts as i32, // Passes
                self.seed as u64,       // Seed
                self.imbalance as c_float,
            );
        }
        let (bins, cut) = self.evaluate(&hg, &partition);
        (partition, bins, cut)
    }

    pub fn hg_hm_partition(&self, hg: &HyperGraph) -> (Vec<c_int>, Vec<c_int>, usize) {
        let mut partition = hg.part.clone();
        unsafe {
            let mut eind_int = Vec::with_capacity(hg.eind.len());
            for v in &hg.eind {
                eind_int.push(*v as c_int);
                // println!("Convert eind {}", *v);
            }
            let mut eptr_int = Vec::with_capacity(hg.eptr.len());
            for v in &hg.eptr {
                eptr_int.push(*v as c_int);
            }
            // NOTE THE SWAP of eind and eptr.  Different usage in
            // kahypar universe versus hmetis.  Weirdness.
            #[cfg(feature = "hmetis")]
            hmetis_r::hm_partition(
                hg.vtxwt.len() as u32,
                (hg.eind.len() - 1) as u32,
                hg.hewt.as_ptr(),
                hg.vtxwt.as_ptr(),
                eptr_int.as_ptr(),
                eind_int.as_ptr(),
                partition.as_mut_ptr(),
                self.k as i32,
                self.num_starts as i32, // Passes
                self.seed as u64,       // Seed
                (self.imbalance * 100.0) as i32,
            );
        }
        // println!("Back from the hmetis call");
        let (bins, cut) = self.evaluate(&hg, &partition);
        (partition, bins, cut)
    }

    // The dumb partitioner.  Sorts the vertices by weight (descending), then assigns
    // the vertices to bins to minimize the difference in weights
    pub fn partition_dumb(&self, hg: &HyperGraph) -> (Vec<c_int>, Vec<c_int>, usize) {
        let mut bins = vec![0, 0];
        let mut verts = Vec::with_capacity(hg.vtxwt.len());
        let mut part = vec![0; hg.vtxwt.len()];
        
        for i in 0..hg.vtxwt.len() {
            if hg.part[i] == -1 {
                verts.push(VW {
                    weight: hg.vtxwt[i],
                    index: i,
                });
            } else {
                // println!("Fix vertex {} to side {} weight {}", i, hg.part[i], hg.vtxwt[i]);
                bins[hg.part[i] as usize] += hg.vtxwt[i];
                part[i] = hg.part[i];
            }
        }

        verts.sort_by_key(|k| -k.weight);
        for i in 0..verts.len() {
            let id = verts[i].index;
            if bins[0] < bins[1] {
                part[id] = 0;
                bins[0] += hg.vtxwt[id];
                // println!("Place vertex {} on 0, weight {}", id, hg.vtxwt[id]);
            } else {
                part[id] = 1;
                bins[1] += hg.vtxwt[id];
                // println!("Place vertex {} on 1, weight {}", id, hg.vtxwt[id]);
            }

        }
        let (bins, cut) = self.evaluate(hg, &part);

        (part, bins, cut)
    }

    pub fn show(&self, hg: &HyperGraph, part: &Vec<c_int>, bins: &Vec<c_int>, cut: usize) {
        println!(
            "Graph: {} vertices, {} edges.  Cut {}",
            hg.vtxwt.len(),
            hg.eind.len() - 1,
            cut
        );
        for b in 0..bins.len() {
            println!("Bin {} weight: {}", b, bins[b]);
        }
        let mut max_v = part.len();
        if max_v > 16 {
            max_v = 16;
        }
        for i in 0..max_v {
            println!("Vertex {} mapped to bin {}", i, part[i]);
        }
    }

    pub fn evaluate(&self, hg: &HyperGraph, part: &Vec<c_int>) -> (Vec<c_int>, usize) {
        let mut bins = vec![0 as c_int; self.k];
        for i in 0..part.len() {
            bins[part[i] as usize] = bins[part[i] as usize] + hg.vtxwt[i];
        }

        let mut cut = 0;
        for i in 1..hg.eind.len() {
            let mut netbin = vec![false; self.k];
            let mut count = 0;
            // println!("Net {i} from {} to {}", hg.eind[i - 1], hg.eind[i]);
            for vptr in hg.eind[i - 1]..hg.eind[i] {
                let v = hg.eptr[vptr as usize];
                // println!("Vertex {} is in partition {}", v, part[v as usize]);
                if !netbin[part[v as usize] as usize] {
                    count = count + 1;
                    netbin[part[v as usize] as usize] = true;
                }
            }
            if count > 1 {
                cut = cut + hg.hewt[i - 1];
            }
        }

        (bins, cut as usize)
    }
}
