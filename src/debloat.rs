use crate::jssyntax::{
    CLOSE_STATEMENT, ELSE, ELSE_CLAUSE, FUNC_DECL, IF, IF_STATEMENT, OPEN_BRACKET, PROGRAM,
    SEMICOLON, STMT_BLK,
};
use crate::node::{self, Node};
use crate::util;
use tree_sitter::{Point, Range};
use tree_sitter_traversal::Order;

fn aggregate<'a>(
    child: &Node<'a>,
    range_skip: &mut Point,
    debloated: &mut Vec<&'a str>,
    first_stmt_blk: bool,
) {
    let Range { end_point, .. } = child.info.range();
    if end_point > *range_skip {
        debloated.push(child.text);
        if !first_stmt_blk && !child.text.contains(";") {
            debloated.push(SEMICOLON);
        }
        *range_skip = end_point;
    }
}

pub fn debloat_control_flow<'a>(nodes: &Vec<Node<'a>>, code: &'a str) -> String {
    assert!(nodes[0].kind() == PROGRAM);
    let mut debloated = vec![];
    let mut range_skip = Point { row: 0, column: 0 };

    let mut first_stmt_blk = true;
    let mut node = &nodes[1];
    // TODO: FIXME. (use `node::run_subtree` in the outer loop)
    loop {
        match node.kind() {
            FUNC_DECL => {
                node::run_subtree(node, code, |child| {
                    match child.kind() {
                        // skip aggregation if a node is a one of the followings
                        // TODO: Consider other control flows such as switch, for-loop, etc
                        IF_STATEMENT | IF | ELSE | ELSE_CLAUSE | STMT_BLK | OPEN_BRACKET
                        | CLOSE_STATEMENT => {
                            if node.kind() == FUNC_DECL
                                && child.kind() == OPEN_BRACKET
                                && first_stmt_blk
                            {
                                debloated.push(child.text);
                                first_stmt_blk = false;
                            }
                        }
                        _ => {
                            aggregate(&child, &mut range_skip, &mut debloated, first_stmt_blk);
                        }
                    }
                    None
                });

                first_stmt_blk = true;
                debloated.push(CLOSE_STATEMENT);
            }
            _ => {
                aggregate(&node, &mut range_skip, &mut debloated, first_stmt_blk);
            }
        }
        if let Some(next_node) = node::get_next_node(nodes, &node) {
            node = next_node;
        } else {
            break;
        }
    }
    debloated.join("\n")
}

pub fn debloat(filename: &str, debloated_filename: &str) {
    let code = util::read_file(&filename).unwrap();
    let tree = node::get_tree(&code);
    let nodes = node::get_nodes(tree.walk(), Order::Pre, &code);
    let debloated_code = debloat_control_flow(&nodes, &code);
    util::jscode2file(debloated_filename, &debloated_code);
}
