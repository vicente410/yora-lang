use std::fmt;
use std::process;

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
    Idx,
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
            Op::Idx => "[]",
        }
    }

    pub fn get_type(&self) -> &str {
        match self {
            Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Mod => "int",
            Op::And | Op::Or | Op::Eq | Op::Neq | Op::Lt | Op::Leq | Op::Gt | Op::Geq => "bool",
            Op::Idx => "int", // later must return type of the array, but arrays are only of ints
                              // for now
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}
