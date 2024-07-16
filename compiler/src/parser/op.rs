use std::fmt;
use std::process;

use crate::core::PrimitiveType;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Eq,
    Neq,
    Lt,
    Leq,
    Gt,
    Geq,
}

impl Op {
    pub fn from_str(str: &str) -> Op {
        match str {
            "+" => Op::Add,
            "-" => Op::Sub,
            "*" => Op::Mul,
            "/" => Op::Div,
            "%" => Op::Mod,
            "and" => Op::And,
            "or" => Op::Or,
            "==" => Op::Eq,
            "!=" => Op::Neq,
            "<" => Op::Lt,
            "<=" => Op::Leq,
            ">" => Op::Gt,
            ">=" => Op::Geq,
            _ => {
                println!("Unrecognized operation:\n\t{}", str);
                process::exit(1);
            }
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
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
        }
    }

    pub fn get_type(&self) -> PrimitiveType {
        match self {
            Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Mod => PrimitiveType::Int,
            Op::And | Op::Or | Op::Eq | Op::Neq | Op::Lt | Op::Leq | Op::Gt | Op::Geq => {
                PrimitiveType::Bool
            }
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}
