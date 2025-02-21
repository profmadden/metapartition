
use crate::hypergraph::HyperGraph;
use std::os::raw::{c_int, c_uint, c_ulong};
use std::fmt;
use kahypar_r;

pub enum Partitioner {
    H, // hMetis
    M, // METIS
    K, // KaHyPar,
    MT, // mt-KaHyPar,
    D, // Dumb Partitioner
    X, // Meta-partitioner -- try multiple versions
}

pub enum Objective {
    C, // Minimum cut
    D, // Minimum sum-of-degrees
}
pub struct Metapartitioner {
    pub num_starts: usize,
    pub k: usize,
    pub partitioner_type: Partitioner,
    pub objective: Objective,
    pub seed: u64,
}


struct VW {
    weight: c_int,
    index: usize,
}

impl Metapartitioner {
    pub fn new() -> Metapartitioner {
        Metapartitioner {
            num_starts: 1,
            k: 2,
            partitioner_type: Partitioner::K,
            objective: Objective::C,
            seed: 8675309,
        }
    }
    /// Partitions the graph, using the partitioner and k values indicated.  Will
    /// run multiple starts, selecting the best for the supplied objective.  Returns
    /// the bin assignment as the first vector, the total weight in each bin
    /// in the second, and the cost based on the selected objective as the third
    /// value.
    pub fn hg_partition(&self, hg: &HyperGraph) -> (Vec<c_int>,Vec<c_int>,usize) {
        match self.partitioner_type {
            Partitioner::K => {return self.hg_ka_partition(hg);},
            _ => {println!("Not supported"); return (Vec::new(),Vec::new(),0); }
        }
    }

    pub fn hg_ka_partition(&self, hg: &HyperGraph) -> (Vec<c_int>,Vec<c_int>,usize) {
        let mut partition = hg.part.clone();
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
                1 as u64 // Seed
            );
        }
        let (bins, cut) = self.evaluate(&hg, &partition);
        (partition, bins, cut)
    }
    
    
    
    // The dumb partitioner.  Sorts the vertices by weight (descending), then assigns
    // the vertices to bins to minimize the difference in weights
    pub fn partition_dumb(&self, hg: &HyperGraph) -> Vec<c_int> {
        let mut bins = vec![0 as c_int; self.k];

        let mut verts = Vec::with_capacity(hg.vtxwt.len());
        for i in 0..hg.vtxwt.len() {
            verts.push(VW{
                weight: hg.vtxwt[i],
                index: i,
            });
        }

        let mut part = Vec::with_capacity(verts.len());
        
        verts.sort_by_key(|k| k.weight);
        for i in 0..verts.len() {
            part.push(i as c_int % 2);
        }

        part
    }

    pub fn show(&self, hg: &HyperGraph, part: &Vec<c_int>, bins: &Vec<c_int>, cut: usize) {
        println!("Graph: {} vertices, {} edges.  Cut {}", hg.vtxwt.len(), hg.eind.len() - 1, cut);
        for b in 0..bins.len() {
            println!("Bin {} weight: {}", b, bins[b]);
        }
        let mut max_v = part.len();
        if max_v > 10 { max_v = 10;}
        for i in 0..max_v {
            println!("Vertex {} mapped to bin {}", i, part[i]);
        }
    }

    pub fn evaluate(&self, hg: &HyperGraph, part: &Vec<c_int>) -> (Vec<c_int>,usize) {
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
                cut = cut + 1;
            }
        }

        (bins, cut)
    }

}