use crate::jssyntax::{
    BREAK, BREAK_STMT, CASE, CLOSE_BRACKET, CLOSE_PARENTHESIS, COLON, CONTINUE, CONTINUE_STMT, DO,
    DO_STMT, ELSE, ELSE_CLAUSE, EMPTY_STMT, FOR, FOR_IN_STMT, FOR_STMT, FUNC_DECL, IF,
    IF_STATEMENT, OPEN_BRACKET, OPEN_PARENTHESIS, PROGRAM, SEMICOLON, STMT_BLK, SWITCH,
    SWITCH_BODY, SWITCH_CASE, SWITCH_STMT, WHILE, WHILE_STMT,
};
use crate::node::{self, Node};
use crate::util;
use tree_sitter::{Point, Range};
use tree_sitter_traversal::Order;

pub const LOC_ANNOT: &str = "// [Loc]";
pub const NON_BRANCH_ANNOT: &str = "[Non-branch]";
pub const PARENT_NODE_ID_ANNOT: &str = "[Parent-ID]";

fn mapping_source<'a>(node: &Node<'a>, filename: &str) -> String {
    let range = node.info.range().start_point;
    format!(
        "{} {}:{}:{}",
        LOC_ANNOT,
        filename,
        range.row + 1,
        range.column + 1
    )
}

fn aggregate<'a>(
    child: &Node<'a>,
    range_skip: &mut Point,
    debloated: &mut Vec<String>,
    first_stmt_blk: bool,
    filename: &str,
) {
    let Range { end_point, .. } = child.info.range();
    if end_point > *range_skip {
        let mut text = if !first_stmt_blk && !child.text.contains(";") {
            format!("{};", child.text)
        } else {
            child.text.to_string()
        };
        text = format!("{} {}", text, mapping_source(child, filename));
        if !node::is_in_ctrl_flow(child) {
            text = format!("{}, {}", text, NON_BRANCH_ANNOT);
        }
        text = format!(
            "{}, {} {},",
            text,
            PARENT_NODE_ID_ANNOT,
            child.info.parent().unwrap().id()
        );
        debloated.push(text);
        *range_skip = end_point;
    }
}

pub fn debloat_control_flow<'a>(nodes: &Vec<Node<'a>>, code: &'a str, filename: &str) -> String {
    assert!(nodes[0].kind() == PROGRAM);
    let mut debloated = vec![];
    let mut range_skip = Point { row: 0, column: 0 };

    let mut first_stmt_blk = true;
    let mut node = &nodes[1];
    loop {
        match node.kind() {
            FUNC_DECL => {
                node::run_subtree(node, code, |child| {
                    let parent = child.info.parent().unwrap();
                    match child.kind() {
                        // skip aggregation if a node is a one of the followings
                        // TODO: Consider other control flows such as switch, for-loop, etc
                        IF_STATEMENT | IF | ELSE | ELSE_CLAUSE | STMT_BLK | CLOSE_BRACKET
                        | SWITCH_CASE | SWITCH_BODY | SWITCH_STMT | CASE | SWITCH | COLON
                        | FOR_STMT | FOR | BREAK_STMT | BREAK | EMPTY_STMT | WHILE_STMT | WHILE
                        | DO_STMT | DO | CONTINUE_STMT | CONTINUE => {}
                        // Debloat '(' and ')' if they appears in 'for statement'
                        OPEN_PARENTHESIS | CLOSE_PARENTHESIS if parent.kind() == FOR_STMT => {}
                        OPEN_BRACKET => {
                            if first_stmt_blk {
                                debloated.push(OPEN_BRACKET.to_string());
                                first_stmt_blk = false;
                            }
                        }
                        SEMICOLON
                            if parent.kind() == BREAK_STMT
                                || parent.kind() == EMPTY_STMT
                                || parent.kind() == CONTINUE_STMT => {}
                        /*
                        FOR_IN_STMT => {
                            let mut a = child.clone();
                            a.text = &a.text[3..];
                            aggregate(
                                &a,
                                &mut range_skip,
                                &mut debloated,
                                first_stmt_blk,
                                filename,
                            );
                        }
                        */
                        _ => {
                            aggregate(
                                &child,
                                &mut range_skip,
                                &mut debloated,
                                first_stmt_blk,
                                filename,
                            );
                        }
                    }
                    None
                });

                first_stmt_blk = true;
                debloated.push(CLOSE_BRACKET.to_string());
            }
            _ => {
                aggregate(
                    &node,
                    &mut range_skip,
                    &mut debloated,
                    first_stmt_blk,
                    filename,
                );
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
    let debloated_code = debloat_control_flow(&nodes, &code, filename);
    util::jscode2file(debloated_filename, &debloated_code);
}
