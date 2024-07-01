use std::fmt;
use std::io;

use crate::Expression;
use crate::ExpressionKind;

fn walk(expr: &Expression, prefix: &str) -> io::Result<()> {
    let sons: Vec<_> = get_sons(expr.clone());
    let mut index = sons.len();

    for son in sons {
        let string = get_expr_str(&son);
        index -= 1;

        if index == 0 {
            println!("{}└── {}", prefix, string);
            if !get_sons(expr.clone()).is_empty() {
                walk(&son, &format!("{}    ", prefix))?;
            }
        } else {
            println!("{}├── {}", prefix, string);
            if !get_sons(expr.clone()).is_empty() {
                walk(&son, &format!("{}│   ", prefix))?;
            }
        }
    }

    Ok(())
}

fn get_expr_str(expr: &Expression) -> String {
    match &expr.kind {
        ExpressionKind::Identifier(str)
        | ExpressionKind::BoolLit(str)
        | ExpressionKind::IntLit(str) => str.as_str(),
        ExpressionKind::Sequence(..) => "seq",
        ExpressionKind::If(..) => "if",
        ExpressionKind::Loop(..) => "loop",
        ExpressionKind::Break => "break",
        ExpressionKind::Declare(..) => "dec",
        ExpressionKind::Assign(..) => "ass",
        ExpressionKind::Exit(..) => "exit",
        ExpressionKind::Add(..) => "+",
        ExpressionKind::Sub(..) => "-",
        ExpressionKind::Mul(..) => "*",
        ExpressionKind::Div(..) => "/",
        ExpressionKind::Mod(..) => "%",
        ExpressionKind::Not(..) => "!",
        ExpressionKind::And(..) => "and",
        ExpressionKind::Or(..) => "or",
        ExpressionKind::Eq(..) => "==",
        ExpressionKind::Neq(..) => "!=",
        ExpressionKind::Lt(..) => "<",
        ExpressionKind::Leq(..) => "<=",
        ExpressionKind::Gt(..) => ">",
        ExpressionKind::Geq(..) => ">=",
    }
    .to_string()
}

fn get_sons(expr: Expression) -> Vec<Expression> {
    match expr.kind {
        ExpressionKind::Identifier(..)
        | ExpressionKind::BoolLit(..)
        | ExpressionKind::IntLit(..) => Vec::new(),
        ExpressionKind::Sequence(seq) => seq,
        ExpressionKind::If(cond, seq) => vec![*cond, *seq],
        ExpressionKind::Loop(seq) => vec![*seq],
        ExpressionKind::Break => Vec::new(),
        ExpressionKind::Declare(dest, src) => vec![*dest, *src],
        ExpressionKind::Assign(dest, src) => vec![*dest, *src],
        ExpressionKind::Exit(src) => vec![*src],
        ExpressionKind::Add(dest, src)
        | ExpressionKind::Sub(dest, src)
        | ExpressionKind::Mul(dest, src)
        | ExpressionKind::Div(dest, src)
        | ExpressionKind::Mod(dest, src)
        | ExpressionKind::And(dest, src)
        | ExpressionKind::Or(dest, src)
        | ExpressionKind::Eq(dest, src)
        | ExpressionKind::Neq(dest, src)
        | ExpressionKind::Lt(dest, src)
        | ExpressionKind::Leq(dest, src)
        | ExpressionKind::Gt(dest, src)
        | ExpressionKind::Geq(dest, src) => vec![*dest, *src],
        ExpressionKind::Not(src) => vec![*src],
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        println!("{}", get_expr_str(self));
        let _ = walk(self, "");
        write!(f, "")
    }
}
