use crate::jssyntax::{
    ARGS, ASSIGNMENT_STMT, BINARY_EXPR, BREAK, BREAK_STMT, CALL_EXPR, CASE, CLOSE_BRACKET,
    CLOSE_PARENTHESIS, COLON, CONTINUE, CONTINUE_STMT, DO, DOUBLE_QUOTE, DO_STMT, ELSE,
    ELSE_CLAUSE, EMPTY_STMT, EXPR_STMT, FOR, FORMAL_PARAMS, FOR_IN_STMT, FOR_STMT, FUNC_DECL,
    IDENT, IF, IF_STATEMENT, LEXICAL_DECL, OBJECT, OPEN_BRACKET, OPEN_PARENTHESIS, PAIR,
    PARENTHESIZED_EXPR, PROGRAM, RETURN_STMT, SEMICOLON, STMT_BLK, STRING, STRING_FRAGMENT, SWITCH,
    SWITCH_BODY, SWITCH_CASE, SWITCH_STMT, VAR_DECL, WHILE, WHILE_STMT,
};
use crate::node::{self, Node};
use crate::util;
use std::collections::HashSet;
use tree_sitter::{Point, Range};
use tree_sitter_traversal::Order;

struct ScopeEnv {
    lvl: usize,
    lvl_visited: Vec<usize>,
    lvl_occupied: HashSet<(usize, usize)>, // <scope level, # of scope level visited>
}
impl Default for ScopeEnv {
    fn default() -> Self {
        Self {
            lvl: 0,
            lvl_visited: vec![],
            lvl_occupied: HashSet::new(),
        }
    }
}
impl ScopeEnv {
    fn count_lvl_visited(&self, lvl: usize) -> usize {
        self.lvl_visited.iter().filter(|&n| *n == lvl).count()
    }
}

pub const LOC_ANNOT: &str = "// [Loc]";
pub const NON_BRANCH_ANNOT: &str = "[Non-branch]";
pub const PARENT_NODE_ID_ANNOT: &str = "[Parent-ID]";

fn mapping_source<'a>(node: &Node<'a>, filename: &str) -> String {
    let range = node.info.range().start_point;
    format!("{} {}:{}:{}", LOC_ANNOT, filename, range.row, range.column)
}

fn get_scoped_ident<'a>(
    vars: &mut HashSet<String>,
    ident_node: &Node<'a>,
    scope_env: &mut ScopeEnv,
) -> String {
    assert_eq!(ident_node.kind(), IDENT);
    let mut n_visited = scope_env.count_lvl_visited(scope_env.lvl);
    let mut lvl = scope_env.lvl as i64;
    let parent_kind = ident_node.info.parent().unwrap().kind();

    if parent_kind == VAR_DECL || parent_kind == FORMAL_PARAMS {
        scope_env.lvl_occupied.insert((lvl as usize, n_visited));
        vars.insert(ident_node.text.to_string());
        if parent_kind == FORMAL_PARAMS {
            format!("{}_{}_{}", ident_node.text, 1, 1)
        } else {
            format!("{}_{}_{}", ident_node.text, lvl, n_visited)
        }
    } else if vars.get(ident_node.text).is_some() {
        let mut ident = None;
        while lvl >= 0 {
            if scope_env
                .lvl_occupied
                .get(&(lvl as usize, n_visited))
                .is_some()
            {
                ident = Some(format!("{}_{}_{}", ident_node.text, lvl, n_visited));
                break;
            }
            lvl -= 1;
            n_visited = scope_env.count_lvl_visited(lvl as usize);
        }
        if ident.is_none() {
            format!("{}_{}_{}", ident_node.text, scope_env.lvl, n_visited)
        } else {
            ident.unwrap()
        }
    } else {
        scope_env.lvl_occupied.insert((lvl as usize, n_visited));
        vars.insert(ident_node.text.to_string());
        format!("{}_{}_{}", ident_node.text, scope_env.lvl, n_visited)
    }
}

fn aggregate<'a>(
    debloated: &mut Vec<String>,
    child: &Node<'a>,
    text: &str,
    first_stmt_blk: bool,
    filename: &str,
) {
    let mut text = if !first_stmt_blk && !text.contains(";") && !text.contains("function") {
        format!("{}{}", text, SEMICOLON.to_string())
    } else {
        text.to_string()
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
}

fn append_text(text: &str, to_append: &str) -> String {
    if text.len() == 0 {
        to_append.to_string()
    } else {
        format!("{text} {to_append}")
    }
}

pub fn debloat_control_flow<'a>(nodes: &Vec<Node<'a>>, code: &'a str, filename: &str) -> String {
    assert!(nodes[0].kind() == PROGRAM);
    let mut debloated = vec![];

    let mut first_stmt_blk = true;
    let mut node = &nodes[1];
    let mut scope_env = ScopeEnv::default();
    let mut vars = HashSet::new();
    let mut last_row = 0;
    loop {
        let mut text = "".to_string();
        node::run_subtree(node, code, |child, last| {
            let parent = child.info.parent().unwrap();
            let Range { start_point, .. } = child.info.range();
            if last_row < start_point.row {
                last_row = start_point.row;
                if text.len() > 0 {
                    aggregate(&mut debloated, &child, &text, first_stmt_blk, filename);
                    text = "".to_string();
                }
            } else {
                if last {
                    if text.len() > 0 {
                        text = append_text(&text, child.text);
                        aggregate(&mut debloated, &child, &text, first_stmt_blk, filename);
                    }
                    text = "".to_string();
                }
            }
            match child.kind() {
                PROGRAM | FUNC_DECL | FORMAL_PARAMS | STMT_BLK | ASSIGNMENT_STMT | CALL_EXPR
                | LEXICAL_DECL | VAR_DECL | PARENTHESIZED_EXPR | EXPR_STMT | BINARY_EXPR
                | IF_STATEMENT | IF | ELSE | ELSE_CLAUSE | SWITCH_CASE | SWITCH_BODY
                | SWITCH_STMT | CASE | SWITCH | FOR_STMT | FOR | BREAK_STMT | BREAK
                | CONTINUE_STMT | CONTINUE | EMPTY_STMT | WHILE_STMT | WHILE | ARGS
                | RETURN_STMT | STRING | OBJECT | PAIR => {
                    return None;
                }
                OPEN_BRACKET => {
                    if first_stmt_blk {
                        if node.kind() == FUNC_DECL {
                            text = append_text(&text, OPEN_BRACKET);
                        }
                        first_stmt_blk = false;
                    }
                    scope_env.lvl += 1;
                    scope_env.lvl_visited.push(scope_env.lvl);
                    if parent.kind() == OBJECT {
                        text = append_text(&text, OPEN_BRACKET);
                    }
                }
                CLOSE_BRACKET => {
                    if parent.kind() == OBJECT {
                        text = append_text(&text, CLOSE_BRACKET);
                    }
                    scope_env.lvl -= 1;
                }
                SEMICOLON
                    if parent.kind() == BREAK_STMT
                        || parent.kind() == EMPTY_STMT
                        || parent.kind() == CONTINUE_STMT => {}
                STRING_FRAGMENT => {
                    text = append_text(&text, child.text);
                }
                DOUBLE_QUOTE => {
                    text = append_text(&text, DOUBLE_QUOTE);
                }
                COLON => {
                    if parent.kind() != SWITCH_CASE {
                        text = append_text(&text, child.text);
                    } else {
                        aggregate(&mut debloated, &child, &text, first_stmt_blk, filename);
                        text = "".to_string();
                    }
                }
                IDENT => match parent.kind() {
                    FUNC_DECL | CALL_EXPR => {
                        text = append_text(&text, child.text);
                    }
                    _ => {
                        let ident = get_scoped_ident(&mut vars, child, &mut scope_env);
                        text = append_text(&text, &ident);
                    }
                },
                _ => {
                    text = append_text(&text, child.text);
                }
            }
            None
        });

        if node.kind() == FUNC_DECL {
            debloated.push(CLOSE_BRACKET.to_string());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testSSA() {
        let filename = "test/debloat/scope.js";
        let debloated_filename = "test/debloat/scope_debloated.js";
        debloat(filename, debloated_filename);
        let debloated_code = util::read_file(debloated_filename).unwrap();
        let debloated_code = debloated_code
            .split("\n")
            .map(|line| {
                let mut s = "".to_string();
                for (idx, l) in line.chars().enumerate() {
                    if l == '/' && line.len() > idx && line.chars().nth(idx + 1).unwrap() == '/' {
                        s.pop();
                        break;
                    }
                    s = format!("{}{}", s, l);
                }
                s
            })
            .collect::<Vec<_>>()
            .join("\n");

        let expected = r#"let a_0_0 = 1
(true);
let a_1_1 = 2;
a_1_1 = 3;
let a_1_2 = 4;
a_1_2 = 5;
(true);
a_1_2 = 6;
(true);
let a_3_1 = 7;
a_3_1 = 8;
let a_2_2 = 9;
a_2_2 = 10;
a_1_2 = 11;
a_0_0 = 12;"#;
        assert_eq!(debloated_code, expected);
        std::fs::remove_file(debloated_filename).unwrap();
    }
}
