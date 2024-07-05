use std::fmt;

use crate::Expression;
use crate::ExpressionKind;
use crate::Op;

fn walk(expr: &Expression, prefix: &str) -> String {
    let mut result = String::new();
    let sons: Vec<_> = get_sons(expr.clone());
    let mut index = sons.len();

    for son in sons {
        let string = get_expr_str(&son);
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

fn get_expr_str(expr: &Expression) -> String {
    match &expr.kind {
        ExpressionKind::Identifier(str)
        | ExpressionKind::BoolLit(str)
        | ExpressionKind::IntLit(str)
        | ExpressionKind::StringLit(str) => str.as_str(),
        ExpressionKind::Sequence(..) => "seq",
        ExpressionKind::If(..) => "if",
        ExpressionKind::IfElse(..) => "if_else",
        ExpressionKind::Loop(..) => "loop",
        ExpressionKind::While(..) => "while",
        ExpressionKind::Continue => "continue",
        ExpressionKind::Break => "break",
        ExpressionKind::Declare(..) => "dec",
        ExpressionKind::Assign(..) => "ass",
        ExpressionKind::Call(name, ..) => name,
        ExpressionKind::Not(..) => "!",
        ExpressionKind::Op(_, op, _) => match op {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Div => "/",
            Op::Mod => "%",
            Op::And => "and",
            Op::Or => "or",
            Op::Eq => "==",
            Op::Neq => "!=",
            Op::Lt => "<",
            Op::Leq => "<=",
            Op::Gt => ">",
            Op::Geq => ">=",
        },
    }
    .to_string()
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
        ExpressionKind::Call(.., args) => vec![*args],
        ExpressionKind::Op(dest, _, src) => vec![*dest, *src],
        ExpressionKind::Not(src) => vec![*src],
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n{}", get_expr_str(self), walk(self, ""))
    }
}
