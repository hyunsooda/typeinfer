pub mod debloat;
pub mod infer;
pub mod instrument;
pub mod jssyntax;
pub mod node;
pub mod util;

use tree_sitter_traversal::Order;

fn main() {
    // 1. Debloat origin source code to remove control flows
    let filename = "example/example.js";
    let debloated_filename = "debloated.js";
    debloat::debloat(filename, debloated_filename);

    // 2. Run infer
    let code = util::read_file(&debloated_filename).unwrap();
    let tree = node::get_tree(&code);
    let mut nodes = node::get_nodes(tree.walk(), Order::Pre, &code);
    assert_eq!(nodes[0].kind(), jssyntax::PROGRAM);
    nodes.remove(0);
    infer::run_func(&nodes, &code);

    /*
    for node in nodes {
        println!("{:?}", node);
    }
    */
}
