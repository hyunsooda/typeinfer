use crate::debloat::NON_BRANCH_ANNOT;
use crate::jssyntax::{
    JSOp, JSTyp, ADD, ASSIGNMENT_STMT, BINARY_EXPR, CALL_EXPR, CLOSE_BRACKET, COMMENT, DIV, EQ,
    EXPR_STMT, FALSE, FORMAL_PARAMS, FUNC_DECL, GE, GT, IDENT, LE, LEXICAL_DECL, LT, MUL, NEQ,
    NULL, NUMBER, OBJECT, OPEN_BRACKET, RETURN_STMT, SEQ, SNEQ, STMT_BLK, STRING, SUB, TRUE,
    UNDEFINED,
};
use crate::node::{self, Node};
use std::collections::{HashMap, HashSet};
use tree_sitter_traversal::Order;

type VarMap = HashMap<(usize, String), HashSet<(usize, JSTyp)>>; // <(scope, variable), (parent node id, jstyp)>
fn varmap_to_string(varmap: &VarMap) -> String {
    let mut s = "".to_string();
    let mut keys = varmap.keys().collect::<Vec<_>>();
    keys.sort();
    for k in keys {
        let (_, var) = k;
        let mut typ_str = "".to_string();
        for (_, typ) in varmap.get(k).unwrap() {
            typ_str = format!("{} {:?},", typ_str, typ);
        }
        s = format!("{} {}: ({})", s, var, &typ_str[1..typ_str.len() - 1]);
    }
    s
}

fn is_overwritable(
    vars: &VarMap,
    scope: usize,
    var: &str,
    parent_id: usize,
) -> (bool, Option<JSTyp>) {
    for ((scope_, var_), typ_set) in vars.into_iter() {
        for (pid, cur_typ) in typ_set.iter() {
            if *scope_ == scope && var_ == var && *pid == parent_id {
                return (true, Some(cur_typ.clone()));
            }
        }
    }
    (false, None)
}

fn is_set(vars: &VarMap, scope: usize, var: &str, typ: &JSTyp) -> bool {
    for ((scope_, var_), typ_set) in vars.into_iter() {
        for (_, cur_typ) in typ_set.iter() {
            if *scope_ == scope && var_ == var && cur_typ == typ {
                return true;
            }
        }
    }
    false
}

fn insert_var(vars: &mut VarMap, scope: usize, var: &str, typ: JSTyp, parent_id: usize) {
    let (overwritable, prev_typ) = is_overwritable(&vars, scope, var, parent_id);
    if overwritable {
        let h = vars.get_mut(&(scope, var.to_string())).unwrap();
        h.remove(&(parent_id, prev_typ.unwrap()));
    }

    if !is_set(&vars, scope, var, &typ) {
        vars.entry((scope, var.to_string()))
            .or_insert(HashSet::new())
            .insert((parent_id, typ));
    }
}

fn overwrite_var(vars: &mut VarMap, scope: usize, var: &str, typ: JSTyp, parent_id: usize) {
    let mut s = HashSet::new();
    s.insert((parent_id, typ));
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
    node::run_subtree(node, code, |child, last| {
        match child.kind() {
            FORMAL_PARAMS => {
                for (idx, param_child) in get_func_params(child, code).iter().enumerate() {
                    let parent_id = node::get_parent_id(node::get_annot(param_child, code));
                    insert_var(
                        vars,
                        0,
                        param_child.text,
                        param_typs[idx].clone(),
                        parent_id,
                    );
                }
            }
            STMT_BLK => run_stmt_blk(&mut scope, vars, nodes, child, code),
            _ => {}
        }
        Some(child.info.range())
    });
}

fn get_func_params<'a>(node: &Node<'a>, code: &'a str) -> Vec<Node<'a>> {
    assert_eq!(node.kind(), FORMAL_PARAMS);
    let mut params = vec![];
    node::run_subtree(node, code, |child, last| {
        match child.kind() {
            IDENT => params.push(child.clone()),
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
    node::run_subtree(node, code, |child, last| {
        match child.kind() {
            LEXICAL_DECL => {
                run_lexical_decl(scope, vars, child, code);
            }
            ASSIGNMENT_STMT => {
                let (lhs, typ, parent_id) = run_assignment_stmt(scope, vars, child, code);
                insert_var(vars, *scope, lhs, typ.clone(), parent_id);
            }
            EXPR_STMT => {
                run_expr_stmt(scope, vars, child, code);
            }
            STMT_BLK => {
                run_stmt_blk(scope, vars, nodes, child, code);
            }
            RETURN_STMT => {
                run_return_stmt(child, code);
                println!("vars: {}", varmap_to_string(&vars));
            }
            _ => {}
        }
        Some(child.info.range())
    })
}

fn run_return_stmt<'a>(node: &Node<'a>, code: &str) -> String {
    assert_eq!(node.kind(), RETURN_STMT);
    let mut ident = "".to_string();
    node::run_subtree(node, code, |child, last| {
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
    node::run_subtree(node, code, |child, last| {
        match child.kind() {
            BINARY_EXPR => {
                run_binary_expr(scope, vars, child, code, &None);
                return Some(child.info.range());
            }
            ASSIGNMENT_STMT => {
                let (lhs, typ, parent_id) = run_assignment_stmt(scope, vars, child, code);
                if !overwrite_typ(*scope, vars, &node, code, lhs, typ.clone()) {
                    insert_var(vars, *scope, lhs, typ.clone(), parent_id);
                }
                return Some(child.info.range());
            }
            _ => {}
        }
        None
    })
}

fn run_lexical_decl<'a>(scope: &mut usize, vars: &mut VarMap, node: &Node<'a>, code: &str) {
    assert_eq!(node.kind(), LEXICAL_DECL);
    let mut ident = None;
    let mut typ = JSTyp::Undefined;
    node::run_subtree(node, code, |child, last| {
        match child.kind() {
            OPEN_BRACKET => {
                *scope += 1;
            }
            CLOSE_BRACKET => {
                *scope -= 1;
            }
            IDENT => {
                if let Some(ident_) = ident {
                    let parent_id = node::get_parent_id(node::get_annot(node, code));
                    insert_var(vars, *scope, ident_, typ.clone(), parent_id);
                }
                ident = Some(child.text);
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
        None
    });
    let parent_id = node::get_parent_id(node::get_annot(node, code));
    insert_var(vars, *scope, ident.unwrap(), typ, parent_id);
}

fn run_assignment_stmt<'a>(
    scope: &mut usize,
    vars: &mut VarMap,
    node: &Node<'a>,
    code: &'a str,
) -> (&'a str, JSTyp, usize) {
    assert_eq!(node.kind(), ASSIGNMENT_STMT);
    let mut lhs = "";
    let mut typ = JSTyp::Undefined;
    let mut eq_before = true;
    node::run_subtree(node, code, |child, last| {
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
    let parent_id = node::get_parent_id(node::get_annot(node, code));
    //insert_var(vars, *scope, lhs, typ.clone(), parent_id);
    (lhs, typ, parent_id)
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
    node::run_subtree(node, code, |child, last| {
        match child.kind() {
            BINARY_EXPR => {
                lhs = Some(run_binary_expr(scope, vars, child, code, &lhs));
            }
            IDENT | TRUE | FALSE | NULL | UNDEFINED | NUMBER | STRING | OBJECT | CALL_EXPR => {
                if node.kind() != CALL_EXPR || is_symbol_call(child, code) {
                    if last_calc_typ.is_some() && lhs.is_none() {
                        lhs = last_calc_typ.clone();
                        rhs = Some(kind2typ(child, vars, *scope, child.text, code));
                    } else if lhs.is_none() {
                        lhs = Some(kind2typ(child, vars, *scope, child.text, code));
                    } else {
                        rhs = Some(kind2typ(child, vars, *scope, child.text, code));
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
) -> bool {
    let next_sib = node.info.next_sibling().unwrap();
    if next_sib.kind() == COMMENT && code[next_sib.byte_range()].contains(NON_BRANCH_ANNOT) {
        println!("vars: {}", varmap_to_string(&vars));
        let parent_id = node::get_parent_id(node::get_annot(node, code));
        overwrite_var(vars, scope, var, typ, parent_id);
        true
    } else {
        false
    }
}

// TODO: Remove me
fn stop<'a>(node: &Node<'a>) {
    println!("STOP! {:?}", node);
    loop {}
}

fn kind2typ<'a>(node: &Node<'a>, vars: &mut VarMap, scope: usize, text: &str, code: &str) -> JSTyp {
    match node.kind() {
        IDENT => {
            let typs = vars.get(&(scope, text.to_string())).unwrap();
            if typs.len() > 1 {
                return JSTyp::Unknown;
            } else {
                //return typs.iter().next().cloned();
                let (_, typ) = typs.iter().next().cloned().unwrap();
                return typ;
            }
        }
        NUMBER => number2typ(node),
        STRING => JSTyp::String,
        NULL => JSTyp::Null,
        UNDEFINED => JSTyp::Undefined,
        TRUE | FALSE => JSTyp::Bool,
        CALL_EXPR if is_symbol_call(node, code) => JSTyp::Symbol,
        OBJECT => JSTyp::Object,
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
