use crate::jssyntax::{
    JSOp, JSTyp, BINARY_EXPR, CLOSE_STATEMENT, ELSE, ELSE_CLAUSE, EQ, FUNC_DECL, IF, IF_STATEMENT,
    LEXICAL_DECL, OPEN_BRACKET, PROGRAM, SEMICOLON, STATEMENT_BLOCK, VAR_DECL,
};
use crate::node::{self, Node};
use std::collections::HashMap;
use tree_sitter_traversal::Order;

type VarMap = HashMap<(usize, String), JSTyp>;

pub fn infer_func<'a>(nodes: &Vec<Node<'a>>, code: &str) {
    assert_eq!(nodes[0].kind(), FUNC_DECL);
    // TODO: Replace the loop with concise framework
    let mut vars = HashMap::new();
    let mut node = &nodes[1];
    let mut scope = 0;
    loop {
        match node.kind() {
            LEXICAL_DECL => run_lexical_decl(&mut scope, &mut vars, node, code),
            BINARY_EXPR => run_binary_expr(&mut scope, &mut vars, node, code),
            _ => unimplemented!(),
        }
    }
}

fn run_lexical_decl<'a>(scope: &mut usize, vars: &mut VarMap, node: &Node<'a>, code: &str) {
    assert_eq!(node.kind(), LEXICAL_DECL);
    node::run_subtree(node, code, |child| match child.kind() {
        OPEN_BRACKET => *scope += 1,
        CLOSE_BRACKET => *scope -= 1,
        IDENT => {
            vars.insert((*scope, child.text.to_string()), JSTyp::Undefined);
        }
        _ => {}
    })
}

fn run_binary_expr<'a>(scope: &mut usize, vars: &mut VarMap, node: &Node<'a>, code: &str) {
    /*
    assert_eq!(node.kind(), BINARY_EXPR);
    let (lhs, rhs, op) = (None, None, None);
    node::run_subtree(node, code, |child| match child.kind() {
        IDENT => {
            if lhs.is_none() {
                lhs = vars.get(&(*scope, child.text.to_string()));
            } else {
                rhs = vars.get(&(*scope, child.text.to_string()));
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
        _ => unimplemented!(),
    });
    let (lhs, rhs, op) = (lhs.unwrap(), rhs.unwrap(), op.unwrap());
    //op.execute(&lhs, &rhs)
    */
    unimplemented!()
}

fn merge_typ() {}
