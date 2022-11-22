use colored::*;

use std::ops;

pub const IF_STATEMENT: &str = "if_statement";
pub const IF: &str = "if";
pub const ELSE: &str = "else";
pub const ELSE_CLAUSE: &str = "else_clause";
pub const STATEMENT_BLOCK: &str = "statement_block";
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
pub const RETURN_STMT: &str = "return_statement";
pub const RETURN: &str = "return";

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
    Object(String),
}
impl JSTyp {
    fn sub_mul_div(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Unknown, _) | (_, Self::Unknown) => Self::Unknown,
            (Self::BigInt, Self::BigInt) => Self::BigInt,
            (Self::Symbol, _) | (_, Self::Symbol) | (Self::BigInt, _) | (_, Self::BigInt) => {
                unreachable!("Actual type error")
            }
            (Self::Object(_), _) | (_, Self::Object(_)) => Self::String,
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
            | (Self::Object(_), Self::Object(_)) => true,
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
            (Self::BigInt, Self::Object(_))
            | (Self::Object(_), Self::BigInt)
            | (Self::String, _)
            | (_, Self::String) => Self::String,
            (Self::Symbol, _) | (_, Self::Symbol) | (Self::BigInt, _) | (_, Self::BigInt) => {
                unreachable!("Actual type error")
            }
            (Self::Object(_), _) | (_, Self::Object(_)) => Self::String,
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
    pub fn execute(&self, a: &JSTyp, b: &JSTyp) -> JSTyp {
        match self {
            Self::Eq | Self::Neq | Self::Gt | Self::Ge | Self::Lt | Self::Le => {
                if !a.is_same_typ(b) {
                    println!(
                        "{} {:?} {:?} {:?}",
                        "[Detected cmp violation]".red(),
                        a,
                        self,
                        b
                    );
                }
                JSTyp::Bool
            }
            Self::Seq | Self::Sneq => JSTyp::Bool,
            Self::Add => {
                self.arithmetic_typ_check(a, b);
                a.clone() + b.clone()
            }
            Self::Sub => {
                self.arithmetic_typ_check(a, b);
                a.clone() - b.clone()
            }
            Self::Mul => {
                self.arithmetic_typ_check(a, b);
                a.clone() * b.clone()
            }
            Self::Div => {
                self.arithmetic_typ_check(a, b);
                a.clone() / b.clone()
            }
        }
    }
    fn arithmetic_typ_check(&self, a: &JSTyp, b: &JSTyp) {
        match self {
            Self::Add => match (a, b) {
                (JSTyp::Number, JSTyp::Number) | (JSTyp::String, JSTyp::String) => {}
                _ => println!(
                    "{}, {:?} {:?} {:?}",
                    "[Detected arithmetic violation]".red(),
                    a,
                    self,
                    b
                ),
            },
            Self::Sub | Self::Mul | Self::Div => match (a, b) {
                (JSTyp::Number, JSTyp::Number) => {}
                _ => println!(
                    "{}, {:?} {:?} {:?}",
                    "[Detected arithmetic violation]".red(),
                    a,
                    self,
                    b
                ),
            },
            _ => unreachable!("Not expected arithmetic type"),
        }
    }
}
