use crate::debloat::LOC_ANNOT;
use crate::jssyntax::COMMENT;
use crate::jssyntax::{JSOp, JSTyp};
use crate::node::Node;
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
    let mut p = node.info.parent();
    while let Some(parent) = p {
        if let Some(next_sib) = parent.next_sibling() {
            if next_sib.kind() == COMMENT {
                let loc = {
                    let loc_annot = &code[next_sib.byte_range()];
                    &loc_annot[LOC_ANNOT.len() + 1..]
                };
                println!(
                    "{} {:?} {:?} {:?} \n{} ({})",
                    format!("[{}]", prefix).red(),
                    lhs_typ,
                    op,
                    rhs_typ,
                    loc2code(loc),
                    loc,
                );
                return;
            }
        }
        p = parent.parent();
    }
}

fn loc2code(loc: &str) -> String {
    let (filename, row) = {
        let loc = loc.split(":").collect::<Vec<_>>();
        (loc[0], loc[1].parse::<usize>().unwrap())
    };
    let code = util::read_file(filename).unwrap();
    code.split("\n").collect::<Vec<_>>()[row].to_string()
}
