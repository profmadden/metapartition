
use crate::hypergraph::HyperGraph;
use std::os::raw::{c_int, c_uint, c_ulong};
use std::fmt;

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
            partitioner_type: Partitioner::D,
            objective: Objective::C,
        }
    }
    /// Partitions the graph, using the partitioner and k values indicated.  Will
    /// run multiple starts, selecting the best for the supplied objective.  Returns
    /// the bin assignment as the first vector, the total weight in each bin
    /// in the second, and the cost based on the selected objective as the third
    /// value.
    pub fn partition(&self, hg: &HyperGraph) -> (Vec<c_int>,Vec<c_int>,usize) {
        (Vec::new(), Vec::new(), 0)
    }

    // The dumb partitioner.  Sorts the vertices by weight (descending), then assigns
    // the vertices to bins to minimize the difference in weights
    

    pub fn partition_dumb(&self, hg: &HyperGraph) -> Vec<c_int> {
        let mut bins = Vec::new();
        for _i in 0..self.k {
            bins.push(0 as c_int);
        }
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

}