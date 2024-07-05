use crate::op::Op;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct Ir {
    pub data: Vec<(String, String, usize)>,
    pub code: Vec<IrInstruction>,
}

impl Ir {
    pub fn new() -> Ir {
        Ir {
            data: Vec::new(),
            code: Vec::new(),
        }
    }

    pub fn add_instruction(&mut self, instruction: IrInstruction) {
        self.code.push(instruction)
    }

    pub fn add_data(&mut self, label: String, data: String, size: usize) {
        self.data.push((label, data, size))
    }
}

impl fmt::Display for Ir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let mut string = String::new();
        string.push_str("section: data\n");
        for (label, data, ..) in &self.data {
            string.push_str(&format!("    {} {}\n", label, data));
        }
        string.push_str("\nsection: code\n");
        for instruction in &self.code {
            string.push_str(&format!("{}\n", instruction));
        }
        write!(f, "{}", string)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum IrInstruction {
    // Operations
    Ass {
        dest: String,
        src: String,
    },
    Not {
        dest: String,
        src: String,
    },
    Op {
        dest: String,
        src1: String,
        op: Op,
        src2: String,
    },

    // Control-flow
    Label(String),
    Goto {
        label: String,
    },
    IfGoto {
        src1: String,
        src2: String,
        cond: Op,
        label: String,
    },

    // Funcion calls
    Param {
        src: String,
    },
    Call {
        label: String,
    },
    Ret {
        src: String,
    },
}

impl fmt::Display for IrInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrInstruction::Ass { dest, src } => write!(f, "    {} = {}", dest, src),
            IrInstruction::Not { dest, src } => write!(f, "    {} = !{}", dest, src),
            IrInstruction::Op {
                dest,
                src1,
                op,
                src2,
            } => write!(f, "    {} = {} {} {}", dest, src1, op, src2),
            IrInstruction::Label(str) => write!(f, "{}:", str),
            IrInstruction::Goto { label } => write!(f, "    goto {}", label),
            IrInstruction::IfGoto {
                src1,
                src2,
                cond,
                label,
            } => write!(f, "    if {} {} {} goto {}", src1, cond, src2, label),
            IrInstruction::Param { src } => write!(f, "    param {}", src),
            IrInstruction::Call { label } => write!(f, "    call {}", label),
            IrInstruction::Ret { src } => write!(f, "    ret {}", src),
        }
    }
}
