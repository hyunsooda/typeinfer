use std::ops;

pub const IF_STATEMENT: &str = "if_statement";
pub const IF: &str = "if";
pub const ELSE: &str = "else";
pub const ELSE_CLAUSE: &str = "else_clause";
pub const STATEMENT_BLOCK: &str = "statement_block";
pub const OPEN_BRACKET: &str = "{";
pub const CLOSE_STATEMENT: &str = "}";
pub const PROGRAM: &str = "program";
pub const FUNC_DECL: &str = "function_declaration";
pub const SEMICOLON: &str = ";";
pub const LEXICAL_DECL: &str = "lexical_declaration";
pub const LET: &str = "let";
pub const VAR_DECL: &str = "variable_declarator";
pub const IDENT: &str = "identifier";
pub const BINARY_EXPR: &str = "binary_expression";

pub const EQ: &str = "==";
pub const NEQ: &str = "!=";
pub const SEQ: &str = "==";
pub const SNEQ: &str = "!=";
pub const GT: &str = ">";
pub const GE: &str = ">=";
pub const LT: &str = "<";
pub const LE: &str = "<=";

pub enum JSTyp {
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
            (Self::BigInt, Self::BigInt) => Self::BigInt,
            (Self::Symbol, _) | (_, Self::Symbol) | (Self::BigInt, _) | (_, Self::BigInt) => {
                unreachable!("Actual type error")
            }
            (Self::Object(_), _) | (_, Self::Object(_)) => Self::String,
            _ => Self::Number,
        }
    }
}

impl ops::Add for JSTyp {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        match (self, rhs) {
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

pub enum JSOp {
    Eq,
    Neq,
    Seq,
    Sneq,
    Gt,
    Ge,
    Lt,
    Le,
}
/*
impl JSOp {
    pub fn execute(&self, a: &JSTyp, b:&JSTyp) -> JSTyp {
        match self {
            Self::Eq =>  a==b,
            Self::Neq =>  a !=b,
            Self::Seq =>  a==b && is_same_typ(a, b),
            Self::Sneq => a !=b && is_same_typ(a, b) ,
            Self::Gt => a > b,
            Self::Ge =>  a >= b,
            Self::Lt => a < b,
            Self::Le =>  a<=b,
        }
    }
}
*/
