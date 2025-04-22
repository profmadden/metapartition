#[cfg(feature = "hmetis")]
use hmetis_r;
// use metapartition::hypergraph::HyperGraph;
use metapartition::metapartitioner::Metapartitioner;
use metapartition::metapartitioner::Partitioner;

use argh::FromArgs;
#[derive(FromArgs)]
/// Metapartitioner wrapper
struct Args {
    /// hgr file
    #[argh(option, short = 'h')]
    hgr: Option<String>,

    /// fix file
    #[argh(option, short = 'f')]
    fix: Option<String>,

    /// hmetis partitioner
    #[argh(switch, short = 'H')]
    hmetis: bool,

    /// dumb partitioner
    #[argh(switch, short = 'd')]
    dumb: bool,

    /// seed for the partitioner
    #[argh(option, short = 's')]
    seed: Option<u64>,

    /// imbalance factor
    #[argh(option, short = 'b')]
    balance: Option<f32>,

    /// partitioner type
    #[argh(switch, short = 'M')]
    mtkahypar: bool,

    /// run BFS
    #[argh(switch)]
    bfs: bool,

    /// run Dijkstra
    #[argh(switch)]
    dijkstra: bool,
}

fn main() {
    println!("Generic meta partitioner, with hypergraph support.");
    let args: Args = argh::from_env();

    let mut mp = Metapartitioner::new();
    println!(
        "Metapartiton: default partitioner is {}",
        Metapartitioner::name(&mp.partitioner_type)
    );

    if args.seed.is_some() {
        mp.seed = args.seed.unwrap();
    }
    if args.hmetis {
        mp.partitioner_type = Partitioner::H;
    }
    if args.mtkahypar {
        mp.partitioner_type = Partitioner::MT;
    }
    if args.dumb {
        mp.partitioner_type = Partitioner::D;
    }
    if args.balance.is_some() {
        mp.imbalance = args.balance.unwrap();
    }
    if Metapartitioner::available(&mp.partitioner_type) {
        println!(
            "Partitioner {} is available",
            Metapartitioner::name(&mp.partitioner_type)
        );
    } else {
        println!(
            "Partitioner {} is NOT available",
            Metapartitioner::name(&mp.partitioner_type)
        );
        return;
    }

    if args.hgr.is_some() {
        let hg = hypergraph::hypergraph::HyperGraph::load(&args.hgr.unwrap(), args.fix);

        if args.bfs {
            let sources = vec![0];
            let distances = hg.bfs(&sources, 100);
            for i in 0..distances.len() {
                println!("Vertex {} distance {}", i, distances[i]);
            }
            return;
        }

        if args.dijkstra {
            println!("Running Dijkstra");
            let sources = vec![0];
            let mut edgelength = Vec::new();
            for edge in 0..hg.eind.len() - 1 {
                let start = hg.eind[edge];
                let end = hg.eind[edge + 1];
                let len = end - start;
                edgelength.push(len as usize);
            }
            let distances = hg.dijkstra(&sources, &edgelength);
            for i in 0..distances.len() {
                println!("Vertex {} distance {}", i, distances[i]);
            }
            return;
        }

        let (part, bins, cut) = mp.hg_partition(&hg);
        println!("Cut is {}  bin0: {} bin1: {}", cut, bins[0], bins[1]);
        mp.show(&hg, &part, &bins, cut);
        return;
    } else {
        println!("Specify a hypergraph file with -hgr to select a graph to partition.");
    }
}
