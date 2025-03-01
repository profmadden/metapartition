use metapartition::hypergraph::HyperGraph;
use metapartition::metapartitioner::Metapartitioner;
#[cfg(feature = "hmetis")]
use hmetis_r;
use metapartition::metapartitioner::Partitioner;

fn main() {
    println!("Generic meta partitioner, with hypergraph support.");
    let hg = HyperGraph::hm_sample();
    println!("Hypergraph details: {}", hg);
    let mut mp = Metapartitioner::new();
    let (part, bins, cut) = mp.hg_partition(&hg);
    println!("Cut is {}", cut);
    mp.show(&hg, &part, &bins, cut);

    println!("Calling hmetis partitioner");
    mp.partitioner_type = metapartition::metapartitioner::Partitioner::H;
    let (part, bins, cut) = mp.hg_partition(&hg);
    println!("Cut is {}", cut);
    mp.show(&hg, &part, &bins, cut);

    println!("Done.");
    
    // unsafe {
    //  hmetis_r::hm_hello();
    // }
}
