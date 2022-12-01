use crate::debloat::{LOC_ANNOT, PARENT_NODE_ID_ANNOT};
use crate::jssyntax::{COMMENT, ELSE_CLAUSE, IF_STATEMENT, STMT_BLK};
use crate::node;
use tree_sitter::{Parser, Tree, TreeCursor};
use tree_sitter::{Point, Range};
use tree_sitter_traversal::{traverse, Order};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Node<'a> {
    pub info: tree_sitter::Node<'a>,
    pub text: &'a str,
}
impl<'a> Node<'a> {
    pub fn kind(&self) -> &str {
        self.info.kind()
    }
}

pub fn get_tree(code: &str) -> Tree {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_javascript::language())
        .unwrap();
    parser.parse(&code, None).unwrap()
}

pub fn get_nodes<'a>(tree_cursor: TreeCursor<'a>, order: Order, code: &'a str) -> Vec<Node<'a>> {
    let ordered_nodes: Vec<tree_sitter::Node<'a>> = traverse(tree_cursor, order).collect();
    ordered_nodes
        .into_iter()
        .map(|node| Node {
            info: node,
            text: &code[node.byte_range()],
        })
        .collect::<Vec<_>>()
}

fn find_node<'a>(nodes: &'a Vec<Node<'a>>, target: &tree_sitter::Node<'a>) -> Option<&'a Node<'a>> {
    nodes.iter().find(|node| node.info.id() == target.id())
}

pub fn get_next_node<'a>(nodes: &'a Vec<Node<'a>>, target: &Node<'a>) -> Option<&'a Node<'a>> {
    if let Some(next_node) = target.info.next_sibling() {
        find_node(nodes, &next_node)
    } else {
        None
    }
}

pub fn run_subtree<'a>(
    node: &Node<'a>,
    code: &'a str,
    mut f: impl FnMut(&Node<'a>, bool) -> Option<Range>,
) {
    let mut run_skip = Point { row: 0, column: 0 };
    let nodes = get_nodes(node.info.walk(), Order::Pre, code);
    for (idx, child) in nodes.iter().skip(1).enumerate() {
        if child.info.range().start_point >= run_skip {
            if let Some(skip) = f(&child, idx == nodes.len() - 2) {
                run_skip = skip.end_point;
            }
        }
    }
}

/// returns true if the parent path includes branch
pub fn is_in_ctrl_flow<'a>(node: &Node<'a>) -> bool {
    let mut p = node.info.parent();
    while let Some(parent) = p {
        match parent.kind() {
            // TODO: Consider other branchs such as switch and for-loop
            IF_STATEMENT | ELSE_CLAUSE => return true,
            _ => {}
        }
        p = parent.parent();
    }
    false
}

pub fn get_annot<'a>(node: &Node<'a>, code: &'a str) -> &'a str {
    if let Some(next_sib) = node.info.next_sibling() {
        if next_sib.kind() == COMMENT {
            return &code[next_sib.byte_range()];
        }
    }

    let mut p = node.info.parent();
    while let Some(parent) = p {
        if let Some(next_sib) = parent.next_sibling() {
            if next_sib.kind() == COMMENT {
                return &code[next_sib.byte_range()];
            }
        }
        p = parent.parent();
    }

    // function comment
    if let Some(p) = node.info.parent() {
        let stmt_blk_node = p.next_sibling().unwrap();
        assert_eq!(stmt_blk_node.kind(), STMT_BLK);
        let children = node::get_nodes(stmt_blk_node.walk(), Order::Pre, &code);
        return &code[children[2].info.byte_range()];
    }
    unreachable!();
}

pub fn get_loc<'a>(annot: &'a str) -> &'a str {
    &annot[LOC_ANNOT.len() + 1..annot.find(",").unwrap()]
}

pub fn get_parent_id(annot: &str) -> usize {
    let parent_id_annot = {
        let annot =
            &annot[annot.find(PARENT_NODE_ID_ANNOT).unwrap() + PARENT_NODE_ID_ANNOT.len() + 1..];
        &annot[..annot.find(",").unwrap()]
    };
    parent_id_annot.parse::<usize>().unwrap()
}
