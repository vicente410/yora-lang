use std::fmt;

use crate::Ir;
use crate::Op;

impl fmt::Display for Ir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ir::Ass { dest, src } => write!(f, "    {} = {}", dest, src),
            Ir::Not { dest, src } => write!(f, "    {} = !{}", dest, src),
            Ir::Op {
                dest,
                src1,
                op,
                src2,
            } => write!(f, "    {} = {} {} {}", dest, src1, op, src2),
            Ir::Label(str) => write!(f, "{}:", str),
            Ir::Goto { label } => write!(f, "    goto {}", label),
            Ir::IfGoto {
                src1,
                src2,
                cond,
                label,
            } => write!(f, "    if {} {} {} goto {}", src1, cond, src2, label),
            Ir::Param { src } => write!(f, "    param {}", src),
            Ir::Call { label } => write!(f, "    call {}", label),
            Ir::Ret { src } => write!(f, "    ret {}", src),
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Op::Add => "+",
                Op::Sub => "-",
                Op::Mul => "*",
                Op::Div => "/",
                Op::Mod => "%",
                Op::And => "&",
                Op::Or => "|",
                Op::Eq => "==",
                Op::Neq => "!=",
                Op::Lt => "<",
                Op::Leq => "<=",
                Op::Gt => ">",
                Op::Geq => ">=",
            }
        )
    }
}
