use crate::op::Op;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct Ir {
    pub data: Vec<Buffer>,
    pub code: Vec<IrInstruction>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Buffer {
    pub label: String,
    pub contents: String,
    pub size: usize,
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

    pub fn add_data(&mut self, label: String, contents: String, size: usize) {
        self.data.push(Buffer {
            label,
            contents,
            size,
        })
    }
}

impl fmt::Display for Ir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let mut string = String::new();
        string.push_str("section: data\n");
        for buffer in &self.data {
            string.push_str(&format!("    {} {}\n", buffer.label, buffer.contents));
        }
        string.push_str("\nsection: code\n");
        for instruction in &self.code {
            string.push_str(&format!("{}\n", instruction));
        }
        write!(f, "{}", string)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Value {
    Identifier { id: String },
    Constant { value: String },
    MemPos { id: String, offset: String },
}

#[derive(Debug, PartialEq, Clone)]
pub enum IrInstruction {
    // Operations
    Ass {
        dest: Value,
        src: Value,
    },
    Not {
        dest: Value,
        src: Value,
    },
    Op {
        dest: Value,
        src1: Value,
        op: Op,
        src2: Value,
    },

    // Control-flow
    Label(String),
    Goto {
        label: String,
    },
    IfGoto {
        src1: Value,
        src2: Value,
        cond: Op,
        label: String,
    },

    // Funcion calls
    Param {
        src: Value,
    },
    Call {
        label: String,
    },
    Ret {
        src: Value,
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
            } => {
                write!(f, "    {} = {} {} {}", dest, src1, op, src2)
            }
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

impl fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Identifier { id } => write!(f, "{}", id),
            Value::Constant { value } => write!(f, "{}", value),
            Value::MemPos { id, offset } => write!(f, "[{} + {}]", id, offset),
        }
    }
}
