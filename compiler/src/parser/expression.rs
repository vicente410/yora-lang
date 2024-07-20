use crate::core::PrimitiveType;
use crate::op::*;
use crate::{Token, TokenKind};

#[derive(Debug, PartialEq, Clone)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub line: usize,
    pub col: usize,
    pub r#type: Option<PrimitiveType>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExpressionKind {
    Call(String, Vec<Expression>),
    Lit(String),
    Id(String),
    Array(Vec<Expression>),
}

impl Expression {
    pub fn new(kind: ExpressionKind, token: &Token) -> Expression {
        let r#type = match token.kind {
            TokenKind::BoolLit => Some(PrimitiveType::Bool),
            TokenKind::IntLit => Some(PrimitiveType::Int),
            TokenKind::CharLit => Some(PrimitiveType::Char),
            TokenKind::StringLit => Some(PrimitiveType::Arr(Box::new(PrimitiveType::Char))),
            TokenKind::Operator => Some(Op::get_type(&Op::from_str(&token.str))),
            _ => None,
        };

        Expression {
            kind,
            line: token.line,
            col: token.col,
            r#type,
        }
    }

    pub fn format(&self, prefix: &str) -> String {
        format!("{}\n{}", self.to_str(), self.walk(prefix))
    }

    pub fn walk(&self, prefix: &str) -> String {
        let mut result = String::new();
        let sons: Vec<_> = self.get_sons();
        let mut index = sons.len();

        for son in sons {
            let string = &son.to_str();
            index -= 1;

            if index == 0 {
                result.push_str(&format!("{}└── {}\n", prefix, string));
                if !self.get_sons().is_empty() {
                    result.push_str(&son.walk(&format!("{}    ", prefix)));
                }
            } else {
                result.push_str(&format!("{}├── {}\n", prefix, string));
                if !self.get_sons().is_empty() {
                    result.push_str(&son.walk(&format!("{}│   ", prefix)));
                }
            }
        }

        result
    }

    pub fn to_str(&self) -> &str {
        match &self.kind {
            ExpressionKind::Call(name, ..) => name,
            ExpressionKind::Lit(lit) => lit,
            ExpressionKind::Id(id) => id,
            ExpressionKind::Array(..) => "array",
        }
    }

    fn get_sons(&self) -> Vec<Expression> {
        match &self.kind {
            ExpressionKind::Call(_, args) => args.to_vec(),
            ExpressionKind::Lit(..) => Vec::new(),
            ExpressionKind::Id(..) => Vec::new(),
            ExpressionKind::Array(values) => values.to_vec(),
        }
    }
}
