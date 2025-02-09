use metapartition::hypergraph::HyperGraph;
use metapartition::metapartitioner::Metapartitioner;
fn main() {
    println!("Generic meta partitioner, with hypergraph support.");
    let hg = HyperGraph::hm_sample();
    println!("Hypergraph details: {}", hg);
    let mp = Metapartitioner::new();
    let (part, bins, cut) = mp.hg_partition(&hg);
    println!("Cut is {}", cut);

}
