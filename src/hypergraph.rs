use std::os::raw::{c_int, c_uint, c_ulong};
use std::fmt;

pub struct HyperGraph {
    pub vtxwt: Vec<c_int>,
    pub hewt: Vec<c_int>,
    pub part: Vec<c_int>,
    pub eind: Vec<c_ulong>,
    pub eptr: Vec<c_uint>,
}


impl HyperGraph {
    pub fn new() -> HyperGraph {
        HyperGraph {
            vtxwt: Vec::new(),
            hewt: Vec::new(),
            part: Vec::new(),
            eind: Vec::new(),
            eptr: Vec::new()
        }        
    }
    /// The hMetis manual gives a small 7-vertex, 4-hyperedge example
    /// graph.  The routine returns this graph.  Some weirdness in
    /// the naming of edge index versus edge pointer in the hMetis/KaHyPar
    /// interfaces?
    pub fn hm_sample() -> HyperGraph {
        // Generic hMetis graph example
        HyperGraph {
            vtxwt: vec![1, 1, 1, 1, 1, 1, 1],
            hewt: vec![1, 1, 1, 1],
            part: vec![-1, -1, -1, -1, -1, -1, -1],
            eind: vec![0, 2, 6, 9, 12],
            eptr: vec![0, 2, 0, 1, 3, 4, 3, 4, 6, 2, 5, 6],
        }
    }
}

impl fmt::Display for HyperGraph {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HyperGraph: {} vertices, {} edges", self.vtxwt.len(), self.hewt.len())
    }
}
