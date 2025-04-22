// Test the BFS code for the hypergraph
use argh::FromArgs;
// use metapartition::hypergraph;
use hypergraph::hypergraph::HyperGraph;

#[derive(FromArgs)]
/// BFS program
struct Args {
    /// hgr file
    #[argh(option, short = 'h')]
    hgr: Option<String>,
}

pub fn main() {
    println!("BFS tester");
    let args: Args = argh::from_env();

    let mut hgr;
    if args.hgr.is_some() {
        hgr = hypergraph::hypergraph::HyperGraph::load(&args.hgr.unwrap(), None);
    } else {
        hgr = hypergraph::hypergraph::HyperGraph::hm_sample();
    }
    hgr.show();

    let sources = vec![0 as usize];
    let distance = hgr.bfs(&sources, 100);

    for i in 0..distance.len() {
        println!("Vertex {}: {}", i, distance[i]);
    }
}
