use crate::jssyntax::{
    JSOp, JSTyp, ADD, ASSIGNMENT_STMT, BINARY_EXPR, CLOSE_BRACKET, CLOSE_STATEMENT, DIV, ELSE,
    ELSE_CLAUSE, EQ, EXPR_STMT, FUNC_DECL, GE, GT, IDENT, IF, IF_STATEMENT, LE, LEXICAL_DECL, LT,
    MUL, NEQ, NUMBER, OPEN_BRACKET, PROGRAM, RETURN, RETURN_STMT, SEMICOLON, SEQ, SNEQ,
    STATEMENT_BLOCK, STMT_BLK, STRING, SUB, VAR_DECL,
};
use crate::node::{self, Node};
use std::collections::{HashMap, HashSet};
use tree_sitter::{Point, Range};

type VarMap = HashMap<(usize, String), HashSet<JSTyp>>;

fn insert_var(vars: &mut VarMap, scope: usize, var: &str, typ: JSTyp) {
    vars.entry((scope, var.to_string()))
        .or_insert(HashSet::new())
        .insert(typ);
}

pub fn run_func<'a>(nodes: &Vec<Node<'a>>, code: &str) {
    assert_eq!(nodes[0].kind(), FUNC_DECL);
    let mut vars = HashMap::new();
    // TODO: FIXME (Replace with actual parameter name in every colllected callsites)
    insert_var(&mut vars, 0, "a", JSTyp::Undefined);

    let mut node = &nodes[0];
    let mut scope = 0;
    node::run_subtree(node, code, |child| {
        match child.kind() {
            STMT_BLK => run_stmt_blk(&mut scope, &mut vars, nodes, child, code),
            _ => {}
        }
        Some(child.info.range())
    });
}

fn run_stmt_blk<'a>(
    scope: &mut usize,
    vars: &mut VarMap,
    nodes: &Vec<Node<'a>>,
    node: &Node<'a>,
    code: &str,
) {
    assert_eq!(node.kind(), STMT_BLK);
    let mut node = node;
    node::run_subtree(node, code, |child| {
        match child.kind() {
            LEXICAL_DECL => {
                run_lexical_decl(scope, vars, child, code);
            }
            ASSIGNMENT_STMT => {
                run_assignment_stmt(scope, vars, child, code);
            }
            EXPR_STMT => {
                run_expr_stmt(scope, vars, child, code);
            }
            STMT_BLK => {
                run_stmt_blk(scope, vars, nodes, child, code);
            }
            RETURN_STMT => {
                let ident = run_return_stmt(scope, vars, child, code);
                println!("vars: {:?}", vars);
                stop(&child);
            }
            _ => {}
        }
        Some(child.info.range())
    })
}

fn run_return_stmt<'a>(
    scope: &mut usize,
    vars: &mut VarMap,
    node: &Node<'a>,
    code: &str,
) -> String {
    assert_eq!(node.kind(), RETURN_STMT);
    let mut ident = "".to_string();
    node::run_subtree(node, code, |child| {
        match child.kind() {
            IDENT => ident = child.text.to_string(),
            _ => {}
        }
        Some(child.info.range())
    });
    ident
}

fn run_expr_stmt<'a>(scope: &mut usize, vars: &mut VarMap, node: &Node<'a>, code: &str) {
    assert_eq!(node.kind(), EXPR_STMT);
    node::run_subtree(node, code, |child| {
        match child.kind() {
            BINARY_EXPR => {
                run_binary_expr(scope, vars, child, code, &None);
            }
            ASSIGNMENT_STMT => {
                run_assignment_stmt(scope, vars, child, code);
            }
            _ => {}
        }
        Some(child.info.range())
    })
}

fn run_lexical_decl<'a>(scope: &mut usize, vars: &mut VarMap, node: &Node<'a>, code: &str) {
    assert_eq!(node.kind(), LEXICAL_DECL);
    node::run_subtree(node, code, |child| {
        match child.kind() {
            OPEN_BRACKET => {
                *scope += 1;
            }
            CLOSE_BRACKET => {
                *scope -= 1;
            }
            IDENT => {
                insert_var(vars, *scope, child.text, JSTyp::Undefined);
            }
            _ => {}
        }
        None
    })
}

fn run_assignment_stmt<'a>(scope: &mut usize, vars: &mut VarMap, node: &Node<'a>, code: &str) {
    assert_eq!(node.kind(), ASSIGNMENT_STMT);
    let mut lhs = "";
    let mut eq_before = true;
    node::run_subtree(node, code, |child| {
        match child.kind() {
            IDENT if eq_before == true => {
                lhs = child.text;
            }
            EQ => {
                eq_before = false;
            }
            NUMBER => {
                insert_var(vars, *scope, lhs, JSTyp::Number);
            }
            STRING => {
                insert_var(vars, *scope, lhs, JSTyp::String);
            }
            BINARY_EXPR => {
                let typ = run_binary_expr(scope, vars, child, code, &None);
                insert_var(vars, *scope, lhs, typ);
            }
            _ => {}
        }
        Some(child.info.range())
    })
}

fn run_binary_expr<'a>(
    scope: &mut usize,
    vars: &mut VarMap,
    node: &Node<'a>,
    code: &str,
    last_calc_typ: &Option<JSTyp>,
) -> JSTyp {
    assert_eq!(node.kind(), BINARY_EXPR);
    let (mut lhs, mut rhs, mut op) = (None, None, None);
    node::run_subtree(node, code, |child| {
        match child.kind() {
            BINARY_EXPR => {
                lhs = Some(run_binary_expr(scope, vars, child, code, &lhs));
            }
            IDENT | NUMBER => {
                if last_calc_typ.is_some() && lhs.is_none() {
                    lhs = last_calc_typ.clone();
                    rhs = kind2typ(child, vars, *scope, child.text);
                } else if lhs.is_none() {
                    lhs = kind2typ(child, vars, *scope, child.text);
                } else {
                    rhs = kind2typ(child, vars, *scope, child.text);
                }
            }

            EQ => op = Some(JSOp::Eq),
            NEQ => op = Some(JSOp::Neq),
            SEQ => op = Some(JSOp::Seq),
            SNEQ => op = Some(JSOp::Sneq),
            GT => op = Some(JSOp::Gt),
            GE => op = Some(JSOp::Ge),
            LT => op = Some(JSOp::Lt),
            LE => op = Some(JSOp::Le),

            ADD => op = Some(JSOp::Add),
            SUB => op = Some(JSOp::Sub),
            MUL => op = Some(JSOp::Mul),
            DIV => op = Some(JSOp::Div),
            _ => {
                println!("[UNIMPLEMENTED:run_binary_expr] {:?}", child);
            }
        }
        Some(child.info.range())
    });
    let (lhs, rhs, op) = (lhs.unwrap(), rhs.unwrap(), op.unwrap());
    op.execute(&lhs, &rhs)
}

fn stop<'a>(node: &Node<'a>) {
    println!("STOP! {:?}", node);
    loop {}
}

fn kind2typ<'a>(node: &Node<'a>, vars: &mut VarMap, scope: usize, text: &str) -> Option<JSTyp> {
    match node.kind() {
        IDENT => {
            let typs = vars.get(&(scope, text.to_string())).unwrap();
            if typs.len() > 1 {
                return Some(JSTyp::Unknown);
            } else {
                return typs.iter().next().cloned();
            }
        }
        NUMBER => Some(JSTyp::Number),
        _ => unimplemented!(),
    }
}
