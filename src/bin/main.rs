use rand::{
    prelude::*,
    rngs::StdRng
};
use std::{
    io::{self, Write}
};

use asa_graphs::graph::ASAGraph;

fn main() {
    let mut rng = StdRng::seed_from_u64(35);

    let mut graph = ASAGraph::<i32, 3>::new("test");

    for _i in 0..1128 {
        let number = rng.gen_range(0..128);
        graph.insert(&number);
    }
    // for i in 0..128 {
    //     graph.insert(&i);
    // }

    graph.print_tree();
}