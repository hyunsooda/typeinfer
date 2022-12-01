use crate::infer;
use crate::jssyntax::{
    JSTyp, ARGS, CALL_EXPR, FALSE, IDENT, NULL, NUMBER, OBJECT, PROGRAM, STRING, TRUE, UNDEFINED,
};
use crate::node::{self, Node};

/// returns a pair of (node, params) for every target callsites
pub fn gather_callsites<'a>(
    target_func_ident: &str,
    node: &Node<'a>,
    code: &'a str,
) -> Vec<(Node<'a>, Vec<JSTyp>)> {
    assert_eq!(node.kind(), PROGRAM);
    let mut callsites = vec![];
    node::run_subtree(node, code, |child, last| {
        match child.kind() {
            CALL_EXPR => {
                let (call_expr_node, param_typs) = run_call_expr(target_func_ident, child, code);
                if let Some(call_expr_node) = call_expr_node {
                    callsites.push((call_expr_node, param_typs));
                }
                return Some(child.info.range());
            }
            _ => {}
        }
        None
    });
    callsites
}

fn run_call_expr<'a>(
    target_func_ident: &str,
    node: &Node<'a>,
    code: &'a str,
) -> (Option<Node<'a>>, Vec<JSTyp>) {
    assert_eq!(node.kind(), CALL_EXPR);
    let mut func_node = None;
    let mut args = vec![];
    node::run_subtree(node, code, |child, last| {
        match child.kind() {
            IDENT if target_func_ident == child.text => func_node = Some(child.clone()),
            ARGS => args = run_arguments(child, code),
            // TODO: Consider `MEMBER_EXPR` for method callsites
            _ => {}
        }
        Some(child.info.range())
    });
    (func_node, args)
}

fn run_arguments<'a>(node: &Node<'a>, code: &'a str) -> Vec<JSTyp> {
    assert_eq!(node.kind(), ARGS);
    let mut typs = vec![];
    node::run_subtree(node, code, |child, last| {
        match child.kind() {
            TRUE | FALSE => typs.push(JSTyp::Bool),
            NULL => typs.push(JSTyp::Null),
            UNDEFINED => typs.push(JSTyp::Undefined),
            NUMBER => typs.push(infer::number2typ(child)),
            STRING => typs.push(JSTyp::String),
            CALL_EXPR if infer::is_symbol_call(child, code) => {
                typs.push(JSTyp::Symbol);
            }
            OBJECT => {
                typs.push(JSTyp::Object);
            }
            _ => {}
        }
        Some(child.info.range())
    });
    typs
}
