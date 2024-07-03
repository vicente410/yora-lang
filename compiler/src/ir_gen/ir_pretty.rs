use std::fmt;

use crate::Ir;
use crate::Op1;
use crate::Op2;

impl fmt::Display for Ir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ir::Op1 { dest, op, src } => write!(f, "    {} = {}{}", dest, op, src),
            Ir::Op2 {
                dest,
                src1,
                op,
                src2,
            } => write!(f, "    {} = {} {} {}", dest, src1, op, src2),
            Ir::Label(str) => write!(f, "{}:", str),
            Ir::Goto { label } => write!(f, "    goto {}", label),
            Ir::IfGoto { src, label } => write!(f, "    if {} goto {}", src, label),
            Ir::Param { src } => write!(f, "    param {}", src),
            Ir::Call { label } => write!(f, "    call {}", label),
            Ir::Ret { src } => write!(f, "    ret {}", src),
        }
    }
}

impl fmt::Display for Op2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Op2::Add => "+",
                Op2::Sub => "-",
                Op2::Mul => "*",
                Op2::Div => "/",
                Op2::Mod => "%",
                Op2::And => "&",
                Op2::Or => "|",
                Op2::Eq => "==",
                Op2::Neq => "!=",
                Op2::Lt => "<",
                Op2::Leq => "<=",
                Op2::Gt => ">",
                Op2::Geq => ">=",
            }
        )
    }
}

impl fmt::Display for Op1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Op1::Ass => "",
                Op1::Neg => "-",
                Op1::Not => "!",
            }
        )
    }
}
