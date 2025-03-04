use std::os::raw::{c_int, c_uint, c_ulong};
use std::fmt;
use lineio;

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
    /// Internally, we go with KaHyPar naming, which flips eptr and eind
    /// relative to hMetis. God, what a headache.
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

    fn to_ints(s: &String) -> Vec<i32> {
        s.split_whitespace().map(|v| v.parse().unwrap()).collect()
    }

    pub fn load(hgr: &String, fix: Option<String>) -> HyperGraph {
        let mut lineio = lineio::LineIO::new(hgr);
        let s = lineio.getline().unwrap();
        println!("Load {}", s);
        let vals = HyperGraph::to_ints(&s);
        let mode;
        if vals.len() == 3 {
            mode = vals[2];
        } else {
            mode = 0;
        }
        // hMetis node and edge weighting scheme
        let eweight = (mode == 1) || (mode == 11);
        let vweight = (mode == 10) || (mode == 11);


        let num_he = vals[0];
        let num_v = vals[1];

        let mut g = HyperGraph::new();
        
        let mut ind_index = 0;
        let mut ptr_index = 0;
        for edge in 0..num_he {
            let s = lineio.getline().unwrap();
            let vals = HyperGraph::to_ints(&s);
            g.eind.push(ptr_index);
            ind_index += 1;

            // Subtract 1 off of the vertex numbers.
            // Seriously, hMetis guys?  Why???
            for v in &vals {
                // println!("Add vertex {} to hyperege {}", *v, edge);
                g.eptr.push((*v - 1) as u32);
                ptr_index = ptr_index + 1;
            }
            
            if eweight {
                // Last element was actually the edge weight.  Have to
                // add 1 back on to get the weight correct
                println!("Take last element off of edge {}", edge);
                g.hewt.push((g.eptr.pop().unwrap() + 1) as c_int);
                ptr_index = ptr_index - 1;
            } else {
                g.hewt.push(1);
            }
        }
        g.eind.push(ptr_index);

        for v in 0..num_v {
            if vweight {
                let s = lineio.getline().unwrap();
                let vals = HyperGraph::to_ints(&s);
                g.vtxwt.push(vals[0]);
            } else {
                g.vtxwt.push(1);
            }
            g.part.push(-1);
        }

        if fix.is_some() {
            let mut lineio = lineio::LineIO::new(&fix.unwrap().clone());
            for v in 0..num_v as usize {
                let s = lineio.getline().unwrap();
                let vals = HyperGraph::to_ints(&s);
                g.part[v] = vals[0];
            }
        }
        
        g

    }
    pub fn save(&self, hgr: Option<&String>, mode: usize, fix: Option<&String>, part: Option<&String>, stats: Option<&String>) {
        println!{"EPTR: {:?}", self.eptr};
        println!("EIND: {:?}", self.eind);

        print!("{} {}", self.hewt.len(), self.vtxwt.len());
        if mode != 0 {
            println!(" {}", mode);
        } else {
            println!("");
        }
        // hMetis node and edge weighting scheme
        let eweight = (mode == 1) || (mode == 11);
        let vweight = (mode == 10) || (mode == 11);

        for he in 0..self.hewt.len() {
            println!("Hyper edge {} is index {} to {}", he, self.eind[he], self.eind[he + 1]);
            for ind in self.eind[he]..self.eind[he + 1] {
                print!("{} ", self.eptr[ind as usize] + 1);
            }
            if eweight {
                println!("{}", self.hewt[he]);
            } else {
                println!("");
            }
        }
        if vweight {
            for v in 0..self.vtxwt.len() {
                println!("{}", self.vtxwt[v]);
            }
        }
        
    }
}

impl fmt::Display for HyperGraph {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HyperGraph: {} vertices, {} edges", self.vtxwt.len(), self.hewt.len())
    }
}
