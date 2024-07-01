use std::fmt;

use crate::Cond;
use crate::Ir;
use crate::Op;

impl fmt::Display for Ir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ir::Op { dest, src, op } => write!(f, "    {} {} {}", dest, op, src),
            Ir::Exit { src } => write!(f, "exit {}", src),
            Ir::Set { dest, cond } => write!(f, "set{} {}", cond, dest),
            Ir::Label(str) => write!(f, "{}:", str),
            Ir::Jmp { label } => write!(f, "jmp {}", label),
            Ir::JmpCond { src, label } => write!(f, "if {} jmp {}", src, label),
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Op::Assign => ":=",
                Op::Add => "+=",
                Op::Sub => "-=",
                Op::Mul => "*=",
                Op::Div => "/=",
                Op::Mod => "%=",
                Op::Not => "!=",
                Op::And => "&=",
                Op::Or => "|=",
                Op::Cmp => "cmp",
            }
        )
    }
}

impl fmt::Display for Cond {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Cond::Eq => "e",
                Cond::Neq => "ne",
                Cond::Lt => "l",
                Cond::Leq => "le",
                Cond::Gt => "g",
                Cond::Geq => "ge",
            }
        )
    }
}
