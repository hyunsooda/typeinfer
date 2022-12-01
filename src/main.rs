pub mod callgraph;
pub mod debloat;
pub mod infer;
pub mod instrument;
pub mod jssyntax;
pub mod node;
pub mod report;
pub mod util;

use crate::node::Node;
use std::collections::HashMap;
use tree_sitter_traversal::Order;

fn dump_origin_source() {
    let code = util::read_file("example/example.js").unwrap();
    let tree = node::get_tree(&code);
    let nodes = node::get_nodes(tree.walk(), Order::Pre, &code);
    let filename = "node-dump.txt";
    dump_node(&nodes, filename);
}

fn main() {
    dump_origin_source();

    // 1. Debloat origin source code to remove control flows
    let filename = "example/example.js";
    let debloated_filename = "debloated.js";
    let dump_filename = "debloated-dump.txt";
    debloat::debloat(filename, debloated_filename);

    // 2. Run infer
    let code = util::read_file(&debloated_filename).unwrap();
    let tree = node::get_tree(&code);
    let mut nodes = node::get_nodes(tree.walk(), Order::Pre, &code);
    assert_eq!(nodes[0].kind(), jssyntax::PROGRAM);
    let target_callsites = callgraph::gather_callsites("foo", &nodes[0], &code);
    println!("callsites of foo: {:?}", target_callsites);
    nodes.remove(0);
    dump_node(&nodes, dump_filename);
    let mut vars = HashMap::new();
    let param_typs = &target_callsites[0].1;
    // TODO: FIXME (Replace with actual parameter name in every colllected callsites)
    infer::run_func(&mut vars, param_typs, &nodes, &nodes[0], &code);
}

fn dump_node<'a>(nodes: &Vec<Node<'a>>, filename: &str) {
    let mut node_str = format!("{:?}", nodes[0]);
    for node in nodes.iter().skip(1) {
        node_str = format!("{}\n{:?}", node_str, node);
    }
    util::write_file(filename, &node_str);
}
