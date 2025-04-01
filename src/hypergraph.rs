use std::os::raw::{c_int, c_uint, c_ulong};
use std::fmt;
use std::collections::VecDeque;
use std::collections::BinaryHeap;
use std::cmp::Reverse;
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
            vtxwt: Vec::new(), // vertex weight
            hewt: Vec::new(), // hyperedge weight
            part: Vec::new(), // partition
            eind: Vec::new(), // edge index
            eptr: Vec::new() // edge pointer
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

    /// Fixes a set of vertices to a specified side of the partition
    pub fn fix(&mut self, vertices: &Vec<usize>, part: c_int) {
        for v in vertices {
            self.part[*v] = part;
        }
    }

    /// Adds additional weight to one side or another, 
    pub fn bias(&mut self, b: f32) {
        // Splitting exactly in half?  Nothing to do
        if b == 0.5 {
            return;
        }

        let mut fix0 = None;
        let mut fix1 = None;
        let mut a = 0;
        for i in 0..self.vtxwt.len() {
            a = a + self.vtxwt[i];
            if self.part[i] == 0 {
                fix0 = Some(i);
            }
            if self.part[i] == 1 {
                fix1 = Some(i);
            }
        }
        // Add weight to the 1 side, so that the 0 side has less
        if b < 0.5 {
            let add_wt = 1;
            if fix1.is_none() {
                // Create a vertex on side 1 if we need it
                self.vtxwt.push(add_wt);
                self.part.push(1);
            } else {
                self.vtxwt[fix1.unwrap()] += add_wt;
            }
            return;
        }
        let add_wt = 1;
        if fix0.is_none() {
            self.vtxwt.push(add_wt);
            self.part.push(0);
        } else {
            self.vtxwt[fix0.unwrap()] += add_wt;
        }
    }

    fn to_ints(s: &String) -> Vec<i32> {
        s.split_whitespace().map(|v| v.parse().unwrap()).collect()
    }

    pub fn load(hgr: &String, fix: Option<String>) -> HyperGraph {
        let mut lineio = lineio::LineIO::new(hgr).unwrap();
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
        println!("eind length: {} eptr length {}", g.eind.len(), g.eptr.len());

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

        let mut numfixed = 0;
        if fix.is_some() {
            let mut lineio = lineio::LineIO::new(&fix.unwrap().clone()).unwrap();
            for v in 0..num_v as usize {
                let s = lineio.getline().unwrap();
                let vals = HyperGraph::to_ints(&s);
                g.part[v] = vals[0];
                if vals[0] != -1 {
                    numfixed = numfixed + 1;
                }
            }
            println!("{} fixed vertices", numfixed);
        }

        
        g

    }

    /// Saves a hypergraph in the hMetis format.  Also can save the fix
    /// file (with indications on which vertices are fixed), and can
    /// also generate a file containing the partitioning.
    /// File names are passed in as options (use None to prevent file
    /// creation)
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

    pub fn show(&self) {
        println!("Hypergraph has {} edges, {} vertices", self.eptr.len() - 1, self.vtxwt.len());
    }

    pub fn vertex_edge_container(&self) -> Vec<Vec<usize>>
    {
        // Get mapping of edges to vertices
        let mut num_edges = 0;
        if self.eptr.is_empty()
        {
            num_edges = 0;
        }
        else
        {
            num_edges = self.eind.len() - 1;
        }

        let mut vert_edge_container: Vec<Vec<usize>> = vec![vec![]; self.vtxwt.len()] ;
        // println!("Reserve space for {} vertices", self.vtxwt.len());
        for edge in 0..num_edges 
        {
            let start = self.eind[edge] as usize;
            let end = self.eind[edge + 1] as usize;
            // println!("Edge {} indexes [{} to {})", edge, start, end);
            for i in start..end 
            {
                let vertex  = self.eptr[i] as usize;
                // println!("Vertex {} [index {}] contains edge {}", vertex, i, edge);
                vert_edge_container[vertex].push(edge);
            }
        }

        vert_edge_container
    }

    // Will need to build a list of edges that touch each vertex

    /// Performs a breadth-first search from the list of source vertices,
    /// returning a vector containing the distance from the start for
    /// each vertex.  Twice the number of vertices is used as the
    /// "infinite" value.  Edges with more than "limit" connections
    /// are skipped.

    pub fn bfs(&self, sources: &Vec<usize>, limit: usize) -> Vec<usize> {

        /*// Get mapping of edges to vertices
        let mut num_edges = 0;
        if self.eptr.is_empty()
        {
            num_edges = 0;
        }
        else
        {
            num_edges = self.eptr.len() - 1;
        }

        let mut vert_edge_container: Vec<Vec<usize>> = vec![vec![]; self.vtxwt.len()] ;

        for edge in 0..num_edges 
        {
            let start = self.eptr[edge] as usize;
            let end = self.eptr[edge + 1] as usize;
            for i in start..end 
            {
                let vertex  = self.eind[i] as usize;
                vert_edge_container[vertex].push(edge);
            }
        }
*/
        // Create a new vector
        let mut result = Vec::new();

        // Define infinite as the number of vertices times 2
        let inf = self.vtxwt.len() * 2;

        // Create a vector to keep track of visited vertices using the vec! macro
        let visited = vec![false; self.vtxwt.len()];

        let mut queue = VecDeque::new();

        for _i in 0..self.vtxwt.len() {
            result.push(inf);
        }

        // Push back every element in sources to the queue
        for i in sources
        {
            queue.push_back(*i);
            result[*i] = 0;
        }

        let vert_edge_container = self.vertex_edge_container();

        // Some pattern matches, essentially, this while loop continues as long as queue.pop_front() returns some value
        while let Some(curr) = queue.pop_front()
        {
            let current_distance = result[curr];
            println!("BFS pops vertex {} distance {}", curr, current_distance);
            if current_distance > limit
            {
                continue;
            }
            
            for &edge in &vert_edge_container[curr]
            {
                let start = self.eind[edge] as usize;
                let end = self.eind[edge + 1] as usize;
                println!("Checking edge {}, indexes {} to {}", edge, start, end);

                // bfs to search every vert in hyperedge
                for i in start..end
                {
                    let neighbor = self.eptr[i] as usize;
                    println!("Neighbor {} has current distance {}", neighbor, result[neighbor]);
                    // update path if shorter
                    if result[neighbor] > current_distance + 1
                    {
                        println!("Neighbor {} has old distance {}, updating", neighbor, result[neighbor]);
                        result[neighbor] = current_distance + 1;
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        result
    }

    pub fn dijkstra(&self, sources: &Vec<usize>, edgelength: &Vec<usize>) -> Vec<usize> 
    {
    
        // Create a vector to hold distances
         let mut distances = Vec::new();
         let inf = self.vtxwt.len() * 2;
    
         // Initialize all distances to infinity
         for _vertex in 0..self.vtxwt.len()
         {
             distances.push(inf);
         }
        
         let mut pq = BinaryHeap::new();
    
         for &i in sources
         {
            distances[i] = 0; // set distance to self to 0
            pq.push(Reverse((0, i))); // push source vertices to priority queue
         }
    
         let vert_edge_container = self.vertex_edge_container();
    
         while let Some(Reverse((dist, vertex))) = pq.pop()
         {
             // Don't think I need this
             //let curr_vert = vertex;
             //let curr_dist = dist;
    
             let curr_distance = distances[vertex];
    
             // Check if the distance we're popping is greater than our best distance,
             // If so don't even bother, go to the next
    
             if dist > curr_distance
             {
                 continue;
             }
    
             // Now for each vertex, go through each of its neighbors
             for &edge in &vert_edge_container[vertex]
             {
                 let start = self.eptr[edge] as usize;
                 let end = self.eptr[edge + 1] as usize;
    
                 for i in start..end
                 {
                     let neighbor = self.eind[i] as usize;
                     let weight  = self.hewt[edge] as usize;

                     let new_distance = curr_distance + weight;

                     // Check if distance + weight is less than the distance to get to the neighbor
                     if new_distance < distances[neighbor] 
                     {
                        // Update distance to neighbor to be the new disance and push the vertex-dist pair to pq
                        distances[neighbor] = new_distance;
                        pq.push(Reverse((new_distance, neighbor)))
                     }
                 }
             }
         }
    
         // Return the distance vector
         distances 
    }
}



 impl fmt::Display for HyperGraph {
     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
         write!(f, "HyperGraph: {} vertices, {} edges", self.vtxwt.len(), self.hewt.len())
     }
 }
