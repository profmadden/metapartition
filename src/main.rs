use metapartition::hypergraph::HyperGraph;
use metapartition::metapartitioner::Metapartitioner;
#[cfg(feature = "hmetis")]
use hmetis_r;
use metapartition::metapartitioner::Partitioner;

use argh::FromArgs;
#[derive(FromArgs)]
/// Metapartitioner wrapper
struct Args {
    /// hgr file
    #[argh(option, short='h')]
    hgr: Option<String>,

    /// fix file
    #[argh(option, short='f')]
    fix: Option<String>,

    /// hmetis partitioner
    #[argh(switch, short='H')]
    hmetis: bool,

    /// seed for the partitioner
    #[argh(option, short='s')]
    seed: Option<u64>,

    /// imbalance factor
    #[argh(option, short='b')]
    balance: Option<f32>,

    /// partitioner type
    #[argh(switch, short='M')]
    mtkahypar: bool,
}

fn main() {


    let args: Args = argh::from_env();
    let hg = HyperGraph::load(&args.hgr.unwrap(), args.fix);
    let mut mp = Metapartitioner::new();
    if args.seed.is_some() {
        mp.seed = args.seed.unwrap();
    }
    if args.hmetis {
        mp.partitioner_type = Partitioner::H;
    }
    if args.mtkahypar {
        mp.partitioner_type = Partitioner::MT;
    }
    if args.balance.is_some() {
        mp.imbalance = args.balance.unwrap();
    }
    let (part, bins, cut) = mp.hg_partition(&hg);
    println!("Cut is {}  bin0: {} bin1: {}", cut, bins[0], bins[1]);
    mp.show(&hg, &part, &bins, cut);
    // hg.save(None, 0, None, None, None);
    return;

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
