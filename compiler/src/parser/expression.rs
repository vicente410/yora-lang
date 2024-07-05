use std::fmt;

use crate::op::Op;

#[derive(Debug, PartialEq, Clone)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub line: usize,
    pub col: usize,
}

impl Expression {
    pub fn new(kind: ExpressionKind, line: usize, col: usize) -> Expression {
        Expression { kind, line, col }
    }

    fn to_str(&self) -> &str {
        match &self.kind {
            ExpressionKind::Identifier(str)
            | ExpressionKind::BoolLit(str)
            | ExpressionKind::IntLit(str)
            | ExpressionKind::StringLit(str) => str,
            ExpressionKind::Sequence(..) => "seq",
            ExpressionKind::If(..) => "if",
            ExpressionKind::IfElse(..) => "if_else",
            ExpressionKind::Loop(..) => "loop",
            ExpressionKind::While(..) => "while",
            ExpressionKind::Continue => "continue",
            ExpressionKind::Break => "break",
            ExpressionKind::Declare(..) => "declare",
            ExpressionKind::Assign(..) => "assign",
            ExpressionKind::Array(..) => "array",
            ExpressionKind::Call(name, ..) => &name,
            ExpressionKind::Not(..) => "!",
            ExpressionKind::Op(_, op, _) => op.to_str(),
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n{}", self.to_str(), walk(self, ""))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExpressionKind {
    Call(String, Box<Expression>),

    // Literals
    BoolLit(String),
    IntLit(String),
    StringLit(String),

    // Control flow
    Sequence(Vec<Expression>),
    If(Box<Expression>, Box<Expression>),
    IfElse(Box<Expression>, Box<Expression>, Box<Expression>),
    Loop(Box<Expression>),
    While(Box<Expression>, Box<Expression>),
    Continue,
    Break,

    // Variables
    Identifier(String),
    Declare(Box<Expression>, Box<Expression>),
    Assign(Box<Expression>, Box<Expression>),
    Array(Vec<Expression>),

    // Operators
    Not(Box<Expression>),
    Op(Box<Expression>, Op, Box<Expression>),
}

fn walk(expr: &Expression, prefix: &str) -> String {
    let mut result = String::new();
    let sons: Vec<_> = get_sons(expr.clone());
    let mut index = sons.len();

    for son in sons {
        let string = &son.to_str();
        index -= 1;

        if index == 0 {
            result.push_str(&format!("{}└── {}\n", prefix, string));
            if !get_sons(expr.clone()).is_empty() {
                result.push_str(&walk(&son, &format!("{}    ", prefix)));
            }
        } else {
            result.push_str(&format!("{}├── {}\n", prefix, string));
            if !get_sons(expr.clone()).is_empty() {
                result.push_str(&walk(&son, &format!("{}│   ", prefix)));
            }
        }
    }

    result
}

fn get_sons(expr: Expression) -> Vec<Expression> {
    match expr.kind {
        ExpressionKind::Identifier(..)
        | ExpressionKind::BoolLit(..)
        | ExpressionKind::IntLit(..)
        | ExpressionKind::StringLit(..) => Vec::new(),
        ExpressionKind::Sequence(seq) => seq,
        ExpressionKind::If(cond, seq) => vec![*cond, *seq],
        ExpressionKind::IfElse(cond, if_seq, else_seq) => vec![*cond, *if_seq, *else_seq],
        ExpressionKind::Loop(seq) => vec![*seq],
        ExpressionKind::While(cond, seq) => vec![*cond, *seq],
        ExpressionKind::Continue => Vec::new(),
        ExpressionKind::Break => Vec::new(),
        ExpressionKind::Declare(dest, src) => vec![*dest, *src],
        ExpressionKind::Assign(dest, src) => vec![*dest, *src],
        ExpressionKind::Array(contents) => contents,
        ExpressionKind::Call(.., args) => vec![*args],
        ExpressionKind::Op(dest, _, src) => vec![*dest, *src],
        ExpressionKind::Not(src) => vec![*src],
    }
}
