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
    mut f: impl FnMut(&Node<'a>) -> Option<Range>,
) {
    let mut run_skip = Range {
        start_byte: 0,
        end_byte: 0,
        start_point: Point { row: 0, column: 0 },
        end_point: Point { row: 0, column: 0 },
    };
    for child in get_nodes(node.info.walk(), Order::Pre, code).iter().skip(1) {
        if run_skip < child.info.range() {
            if let Some(skip) = f(&child) {
                run_skip = skip;
            }
        }
    }
}
