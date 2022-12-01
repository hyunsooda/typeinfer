use crate::node::Node;
use crate::report;

use std::ops;

pub const IF_STATEMENT: &str = "if_statement";
pub const IF: &str = "if";
pub const ELSE: &str = "else";
pub const ELSE_CLAUSE: &str = "else_clause";
pub const OPEN_BRACKET: &str = "{";
pub const CLOSE_BRACKET: &str = "}";
pub const CLOSE_STATEMENT: &str = "}";
pub const PROGRAM: &str = "program";
pub const FUNC_DECL: &str = "function_declaration";
pub const SEMICOLON: &str = ";";
pub const LEXICAL_DECL: &str = "lexical_declaration";
pub const LET: &str = "let";
pub const VAR_DECL: &str = "variable_declarator";
pub const IDENT: &str = "identifier";
pub const BINARY_EXPR: &str = "binary_expression";
pub const ASSIGNMENT_STMT: &str = "assignment_expression";
pub const STMT_BLK: &str = "statement_block";
pub const EXPR_STMT: &str = "expression_statement";
pub const NUMBER: &str = "number";
pub const STRING: &str = "string";
pub const STRING_FRAGMENT: &str = "string_fragment";
pub const RETURN_STMT: &str = "return_statement";
pub const RETURN: &str = "return";
pub const OBJECT: &str = "object";
pub const NULL: &str = "null";
pub const COMMENT: &str = "comment";
pub const FORMAL_PARAMS: &str = "formal_parameters";
pub const CALL_EXPR: &str = "call_expression";
pub const MEMBER_EXPR: &str = "member_expression";
pub const ARGS: &str = "arguments";
pub const FALSE: &str = "false";
pub const TRUE: &str = "true";
pub const UNDEFINED: &str = "undefined";
pub const SWITCH_STMT: &str = "switch_statement";
pub const SWITCH_CASE: &str = "switch_case";
pub const SWITCH_BODY: &str = "switch_body";
pub const SWITCH: &str = "switch";
pub const CASE: &str = "case";
pub const COLON: &str = ":";
pub const BREAK_STMT: &str = "break_statement";
pub const BREAK: &str = "break";
pub const FOR_STMT: &str = "for_statement";
pub const FOR: &str = "for";
pub const OPEN_PARENTHESIS: &str = "(";
pub const CLOSE_PARENTHESIS: &str = ")";
pub const EMPTY_STMT: &str = "empty_statement";
pub const WHILE_STMT: &str = "while_statement";
pub const WHILE: &str = "while";
pub const DO_STMT: &str = "do_statement";
pub const DO: &str = "do";
pub const CONTINUE_STMT: &str = "continue_statement";
pub const CONTINUE: &str = "continue";
pub const FOR_IN_STMT: &str = "for_in_statement";
pub const PARENTHESIZED_EXPR: &str = "parenthesized_expression";
pub const DOUBLE_QUOTE: &str = "\"";
pub const PAIR: &str = "pair";

pub const EQ: &str = "==";
pub const NEQ: &str = "!=";
pub const SEQ: &str = "===";
pub const SNEQ: &str = "!==";
pub const GT: &str = ">";
pub const GE: &str = ">=";
pub const LT: &str = "<";
pub const LE: &str = "<=";

pub const ADD: &str = "+";
pub const SUB: &str = "-";
pub const MUL: &str = "*";
pub const DIV: &str = "/";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum JSTyp {
    Unknown, // Top
    Bool,
    Null,
    Undefined,
    Number,
    BigInt,
    String,
    Symbol,
    Object,
}
impl JSTyp {
    fn sub_mul_div(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Unknown, _) | (_, Self::Unknown) => Self::Unknown,
            (Self::BigInt, Self::BigInt) => Self::BigInt,
            (Self::Symbol, _) | (_, Self::Symbol) | (Self::BigInt, _) | (_, Self::BigInt) => {
                unreachable!("Actual type error")
            }
            (Self::Object, _) | (_, Self::Object) => Self::String,
            _ => Self::Number,
        }
    }
    fn is_same_typ(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool, Self::Bool)
            | (Self::Null, Self::Null)
            | (Self::Undefined, Self::Undefined)
            | (Self::Number, Self::Number)
            | (Self::BigInt, Self::BigInt)
            | (Self::String, Self::String)
            | (Self::Symbol, Self::Symbol)
            | (Self::Object, Self::Object) => true,
            _ => false,
        }
    }
}

impl ops::Add for JSTyp {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Unknown, _) | (_, Self::Unknown) => Self::Unknown,
            (Self::BigInt, Self::BigInt) => Self::BigInt,
            (Self::BigInt, Self::Object)
            | (Self::Object, Self::BigInt)
            | (Self::String, _)
            | (_, Self::String) => Self::String,
            (Self::Symbol, _) | (_, Self::Symbol) | (Self::BigInt, _) | (_, Self::BigInt) => {
                unreachable!("Actual type error")
            }
            (Self::Object, _) | (_, Self::Object) => Self::String,
            _ => Self::Number,
        }
    }
}
impl ops::Sub for JSTyp {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self.sub_mul_div(rhs)
    }
}
impl ops::Mul for JSTyp {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        self.sub_mul_div(rhs)
    }
}
impl ops::Div for JSTyp {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        self.sub_mul_div(rhs)
    }
}

#[derive(Debug, Clone)]
pub enum JSOp {
    // Comparison
    Eq,
    Neq,
    Seq,
    Sneq,
    Gt,
    Ge,
    Lt,
    Le,

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
}
impl JSOp {
    pub fn to_string(&self) -> String {
        match self {
            Self::Eq => "==".to_string(),
            Self::Neq => "!=".to_string(),
            Self::Seq => "===".to_string(),
            Self::Sneq => "!==".to_string(),
            Self::Gt => ">".to_string(),
            Self::Ge => ">=".to_string(),
            Self::Lt => "<".to_string(),
            Self::Le => "<=".to_string(),
            Self::Add => "+".to_string(),
            Self::Sub => "-".to_string(),
            Self::Mul => "*".to_string(),
            Self::Div => "/".to_string(),
        }
    }
    pub fn execute<'a>(&self, a: &JSTyp, b: &JSTyp, node: &Node<'a>, code: &str) -> JSTyp {
        match self {
            Self::Eq | Self::Neq | Self::Gt | Self::Ge | Self::Lt | Self::Le => {
                if !a.is_same_typ(b) {
                    report::report_typ_op_violation(
                        node,
                        code,
                        a,
                        b,
                        self,
                        "Detected cmp violation",
                    );
                }
                JSTyp::Bool
            }
            Self::Seq | Self::Sneq => JSTyp::Bool,
            Self::Add => {
                self.arithmetic_typ_check(a, b, node, code);
                a.clone() + b.clone()
            }
            Self::Sub => {
                self.arithmetic_typ_check(a, b, node, code);
                a.clone() - b.clone()
            }
            Self::Mul => {
                self.arithmetic_typ_check(a, b, node, code);
                a.clone() * b.clone()
            }
            Self::Div => {
                self.arithmetic_typ_check(a, b, node, code);
                a.clone() / b.clone()
            }
        }
    }
    fn arithmetic_typ_check<'a>(&self, a: &JSTyp, b: &JSTyp, node: &Node<'a>, code: &str) {
        match self {
            Self::Add => match (a, b) {
                (JSTyp::Number, JSTyp::Number) | (JSTyp::String, JSTyp::String) => {}
                _ => report::report_typ_op_violation(
                    node,
                    code,
                    a,
                    b,
                    self,
                    "Detected arithmetic violation",
                ),
            },
            Self::Sub | Self::Mul | Self::Div => match (a, b) {
                (JSTyp::Number, JSTyp::Number) => {}
                _ => report::report_typ_op_violation(
                    node,
                    code,
                    a,
                    b,
                    self,
                    "Detected arithmetic violation",
                ),
            },
            _ => unreachable!("Not expected arithmetic type"),
        }
    }
}
