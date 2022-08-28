use rand::{
    prelude::*,
    rngs::StdRng
};

use asa_graphs::simple::graph::ASAGraph as ASAGraphSimple;
use asa_graphs::neural::graph::ASAGraph as ASAGraphNeural;

fn main() {
    let mut rng = StdRng::seed_from_u64(35);

    let mut graph_simple = ASAGraphSimple::<i32, 3>::new("test");

    for _i in 0..10_000 {
        let number = rng.gen_range(0..58);
        graph_simple.insert(&number);
    }

    println!("graph_simple");
    graph_simple.print_graph();
    println!();

    let mut rng = StdRng::seed_from_u64(35);

    let mut graph_neural = ASAGraphNeural::<i32, 3>::new("test");

    for _i in 0..10_000 {
        let number = rng.gen_range(0..58);
        graph_neural.insert(&number);
    }

    println!("graph_neural");
    graph_neural.print_graph();
}