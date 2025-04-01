// Test the BFS code for the hypergraph
use metapartition::hypergraph;

pub fn main() {
    println!("BFS tester");
    let mut hgr = hypergraph::HyperGraph::hm_sample();
    let sources = vec![0 as usize];
    let distance = hgr.bfs(&sources, 100);

    for i in 0..distance.len() {
        println!("Vertex {}: {}", i, distance[i]);
    }

}