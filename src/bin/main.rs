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

    for _i in 0..10_000 {
        let number = rng.gen_range(0..58);
        graph.insert(&number);
    }

    graph.print_graph();
}