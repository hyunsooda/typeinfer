use crate::debloat::NON_BRANCH_ANNOT;
use crate::jssyntax::{
    JSOp, JSTyp, ADD, ASSIGNMENT_STMT, BINARY_EXPR, CALL_EXPR, CLOSE_BRACKET, COMMENT, DIV, EQ,
    EXPR_STMT, FALSE, FORMAL_PARAMS, FUNC_DECL, GE, GT, IDENT, LE, LEXICAL_DECL, LT, MUL, NEQ,
    NULL, NUMBER, OBJECT, OPEN_BRACKET, RETURN_STMT, SEQ, SNEQ, STMT_BLK, STRING, SUB, TRUE,
    UNDEFINED,
};
use crate::node::{self, Node};
use std::collections::{HashMap, HashSet};
use tree_sitter::{Point, Range};
use tree_sitter_traversal::{traverse, Order};

type VarMap = HashMap<(usize, String), HashSet<JSTyp>>;

fn insert_var(vars: &mut VarMap, scope: usize, var: &str, typ: JSTyp) {
    vars.entry((scope, var.to_string()))
        .or_insert(HashSet::new())
        .insert(typ);
}

fn overwrite_var(vars: &mut VarMap, scope: usize, var: &str, typ: JSTyp) {
    let mut s = HashSet::new();
    s.insert(typ);
    vars.insert((scope, var.to_string()), s);
}

pub fn run_func<'a>(
    vars: &mut VarMap,
    param_typs: &Vec<JSTyp>,
    nodes: &Vec<Node<'a>>,
    node: &Node<'a>,
    code: &str,
) {
    assert_eq!(node.kind(), FUNC_DECL);
    let mut scope = 0;
    node::run_subtree(node, code, |child| {
        match child.kind() {
            FORMAL_PARAMS => {
                for (idx, param) in get_func_params(child, code).iter().enumerate() {
                    insert_var(vars, 0, param, param_typs[idx].clone());
                }
            }
            STMT_BLK => run_stmt_blk(&mut scope, vars, nodes, child, code),

            _ => {}
        }
        Some(child.info.range())
    });
}

fn get_func_params<'a>(node: &Node<'a>, code: &'a str) -> Vec<&'a str> {
    assert_eq!(node.kind(), FORMAL_PARAMS);
    let mut params = vec![];
    node::run_subtree(node, code, |child| {
        match child.kind() {
            IDENT => params.push(child.text),
            _ => {}
        }
        Some(child.info.range())
    });
    params
}

fn run_stmt_blk<'a>(
    scope: &mut usize,
    vars: &mut VarMap,
    nodes: &Vec<Node<'a>>,
    node: &Node<'a>,
    code: &str,
) {
    assert_eq!(node.kind(), STMT_BLK);
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
                run_return_stmt(child, code);
                println!("vars: {:?}", vars);
            }
            _ => {}
        }
        Some(child.info.range())
    })
}

fn run_return_stmt<'a>(node: &Node<'a>, code: &str) -> String {
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
                return Some(child.info.range());
            }
            ASSIGNMENT_STMT => {
                let (lhs, typ) = run_assignment_stmt(scope, vars, child, code);
                overwrite_typ(*scope, vars, &node, code, lhs, typ);
                return Some(child.info.range());
            }
            _ => {}
        }
        None
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

fn run_assignment_stmt<'a>(
    scope: &mut usize,
    vars: &mut VarMap,
    node: &Node<'a>,
    code: &'a str,
) -> (&'a str, JSTyp) {
    assert_eq!(node.kind(), ASSIGNMENT_STMT);
    let mut lhs = "";
    let mut typ = JSTyp::Undefined;
    let mut eq_before = true;
    node::run_subtree(node, code, |child| {
        match child.kind() {
            IDENT if eq_before == true => {
                lhs = child.text;
            }
            EQ => {
                eq_before = false;
            }
            TRUE | FALSE => {
                typ = JSTyp::Bool;
            }
            NULL => {
                typ = JSTyp::Null;
            }
            UNDEFINED => {
                typ = JSTyp::Undefined;
            }
            NUMBER => {
                typ = number2typ(child);
            }
            STRING => {
                typ = JSTyp::String;
            }
            CALL_EXPR if is_symbol_call(child, code) => {
                typ = JSTyp::Symbol;
            }
            OBJECT => {
                typ = JSTyp::Object;
            }
            BINARY_EXPR => {
                typ = run_binary_expr(scope, vars, child, code, &None);
            }
            _ => {}
        }
        Some(child.info.range())
    });
    insert_var(vars, *scope, lhs, typ.clone());
    (lhs, typ)
}

pub fn is_symbol_call<'a>(node: &Node<'a>, code: &str) -> bool {
    assert_eq!(node.kind(), CALL_EXPR);
    let children = node::get_nodes(node.info.walk(), Order::Pre, code);
    if children[1].kind() == IDENT && children[1].text == "Symbol" {
        true
    } else {
        false
    }
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
            IDENT | TRUE | FALSE | NULL | UNDEFINED | NUMBER | STRING | OBJECT | CALL_EXPR => {
                if node.kind() != CALL_EXPR || is_symbol_call(child, code) {
                    if last_calc_typ.is_some() && lhs.is_none() {
                        lhs = last_calc_typ.clone();
                        rhs = kind2typ(child, vars, *scope, child.text, code);
                    } else if lhs.is_none() {
                        lhs = kind2typ(child, vars, *scope, child.text, code);
                    } else {
                        rhs = kind2typ(child, vars, *scope, child.text, code);
                    }
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
            _ => {}
        }
        Some(child.info.range())
    });
    let (lhs, rhs, op) = (lhs.unwrap(), rhs.unwrap(), op.unwrap());
    op.execute(&lhs, &rhs, node, code)
}

fn overwrite_typ<'a>(
    scope: usize,
    vars: &mut VarMap,
    node: &Node<'a>,
    code: &str,
    var: &str,
    typ: JSTyp,
) {
    if let Some(next_sib) = node.info.next_sibling() {
        if next_sib.kind() == COMMENT && code[next_sib.byte_range()].contains(NON_BRANCH_ANNOT) {
            println!("vars{:?}", vars);
            overwrite_var(vars, scope, var, typ);
        }
    }
}

// TODO: Remove me
fn stop<'a>(node: &Node<'a>) {
    println!("STOP! {:?}", node);
    loop {}
}

fn kind2typ<'a>(
    node: &Node<'a>,
    vars: &mut VarMap,
    scope: usize,
    text: &str,
    code: &str,
) -> Option<JSTyp> {
    match node.kind() {
        IDENT => {
            let typs = vars.get(&(scope, text.to_string())).unwrap();
            if typs.len() > 1 {
                return Some(JSTyp::Unknown);
            } else {
                return typs.iter().next().cloned();
            }
        }
        NUMBER => Some(number2typ(node)),
        STRING => Some(JSTyp::String),
        NULL => Some(JSTyp::Null),
        UNDEFINED => Some(JSTyp::Undefined),
        BOOL => Some(JSTyp::Bool),
        CALL_EXPR if is_symbol_call(node, code) => Some(JSTyp::Symbol),
        // TODO: Object
        _ => unimplemented!(),
    }
}

pub fn number2typ<'a>(node: &Node<'a>) -> JSTyp {
    assert_eq!(node.kind(), NUMBER);
    if node.text.ends_with("n") {
        JSTyp::BigInt
    } else {
        JSTyp::Number
    }
}
