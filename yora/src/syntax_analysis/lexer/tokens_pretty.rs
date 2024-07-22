use std::fmt;

use super::Token;
use super::TokenKind;

impl fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}\t{}\t\"{}\"",
            self.line, self.col, self.kind, self.str
        )
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TokenKind::Keyword => "key",
                TokenKind::Operator => "op",
                TokenKind::Separator => "sep",
                TokenKind::Identifier => "id",
                TokenKind::IntLit => "int",
                TokenKind::BoolLit => "bool",
                TokenKind::StringLit => "string",
                TokenKind::CharLit => "char",
            }
        )
    }
}
