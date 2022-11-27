use crate::jssyntax::{JSOp, JSTyp};
use crate::node::{self, Node};
use crate::util;
use colored::*;

/// Report type viloation
pub fn report_typ_op_violation<'a>(
    node: &Node<'a>,
    code: &str,
    lhs_typ: &JSTyp,
    rhs_typ: &JSTyp,
    op: &JSOp,
    prefix: &str,
) {
    let annot = node::get_annot(node, code);
    let loc = node::get_loc(annot);
    println!(
        "{} {:?} {} {:?} \n{} ({})",
        format!("[{}]", prefix).red(),
        lhs_typ,
        op.to_string(),
        rhs_typ,
        loc2code(loc),
        loc.yellow(),
    );
}

fn loc2code(loc: &str) -> String {
    let (filename, row) = {
        let loc = loc.split(":").collect::<Vec<_>>();
        (loc[0], loc[1].parse::<usize>().unwrap())
    };
    let code = util::read_file(filename).unwrap();
    code.split("\n").collect::<Vec<_>>()[row - 1].to_string()
}
