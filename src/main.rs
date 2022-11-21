pub mod debloat;
pub mod infer;
pub mod instrument;
pub mod jssyntax;
pub mod node;
pub mod util;

use tree_sitter_traversal::Order;

fn main() {
    let filename = "example/example.js";
    let code = util::read_file(&filename).unwrap();
    let tree = node::get_tree(&code);
    let nodes = node::get_nodes(tree.walk(), Order::Pre, &code);

    let debloated_code = debloat::debloat_control_flow(&nodes, &code);
    let debloated_filename = "debloated.js";
    util::jscode2file(debloated_filename, &debloated_code);
    for node in nodes {
        println!("{:?}", node);
    }
}
