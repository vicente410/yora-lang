use std::collections::HashMap;

use crate::ir_gen::*;

struct Generator {
    asm: String,
    symbol_table: HashMap<String, String>,
    type_table: HashMap<String, String>,
    regs: [&'static str; 7],
    current_stack: usize,
}

pub fn generate_asm(ir: Vec<Ir>, type_table: &mut HashMap<String, String>) -> String {
    let mut generator = Generator {
        asm: String::from("global _start\n_start:\n\tmov rbp, rsp\n"),
        symbol_table: HashMap::new(),
        type_table: type_table.clone(),
        regs: ["rbx", "r10", "r11", "r12", "r13", "r14", "r15"],
        current_stack: 0,
    };

    generator.generate_asm(ir);
    generator.asm
}

fn get_cond(cond: &Cond) -> &str {
    match cond {
        Cond::Eq => "e",
        Cond::Neq => "ne",
        Cond::Lt => "l",
        Cond::Leq => "le",
        Cond::Gt => "g",
        Cond::Geq => "ge",
    }
}

fn get_operation(operation: &Op) -> &str {
    match operation {
        Op::Assign => "mov",
        Op::Add => "add",
        Op::Sub => "sub",
        Op::Mul => "mul",
        Op::Div => "div",
        Op::Mod => "div",
        Op::Not => "not",
        Op::And => "and",
        Op::Or => "or",
        Op::Cmp => "cmp",
    }
}

fn get_instruction(src_val: String, dest_val: String, instruction: &Ir) -> String {
    if let Ir::Op { op, .. } = instruction {
        if src_val.contains('[') && dest_val.contains('[') {
            format!(
                "\tpush rax\n\
                \tmov rax, {}\n\
                \t{} {}, rax\n\
                \tpop rax\n",
                src_val,
                get_operation(op),
                dest_val
            )
        } else {
            format!("\t{} {}, {}\n", get_operation(op), dest_val, src_val)
        }
    } else {
        panic!();
    }
}

impl Generator {
    fn generate_asm(&mut self, ir: Vec<Ir>) {
        for instruction in ir {
            let string = match instruction {
                Ir::Op {
                    ref dest,
                    ref src,
                    ref op,
                } => match op {
                    Op::Assign => {
                        if !self.symbol_table.contains_key(&dest.clone()) {
                            self.insert_reg(
                                dest.clone(),
                                get_size_for_type(self.type_table[dest].clone()),
                            );
                        }
                        &get_instruction(self.get_value(src), self.get_value(dest), &instruction)
                    }
                    Op::Mul | Op::Div => &format!(
                        "\tmov rax, {}\n\
                    \t{} {}\n\
                    \tmov {}, rax\n",
                        self.get_value(dest),
                        get_operation(op),
                        self.get_value(src),
                        self.get_value(dest),
                    ),
                    Op::Mod => &format!(
                        "\tmov rax, {}\n\
                    \t{} {}\n\
                    \tmov {}, rdx\n",
                        self.get_value(dest),
                        get_operation(op),
                        self.get_value(src),
                        self.get_value(dest),
                    ),
                    Op::Add | Op::Sub | Op::Not | Op::Cmp | Op::And | Op::Or => {
                        &get_instruction(self.get_value(src), self.get_value(dest), &instruction)
                    }
                },
                Ir::Label(label) => &format!("{}:\n", label),
                Ir::Jmp { label } => &format!("\tjmp {}\n", label),
                Ir::JmpCond { src, label } => &format!(
                    "\tcmp {}, 0\n\
                    \tje {}\n",
                    self.get_value(&src),
                    label
                ),
                Ir::Exit { src } => &format!(
                    "\tmov rdi, {}\n\
                    \tmov rax, 60\n\
                    \tsyscall\n",
                    self.get_value(&src),
                ),
                Ir::Set { dest, cond } => {
                    if !self.symbol_table.contains_key(&dest) {
                        self.insert_reg(
                            dest.clone(),
                            get_size_for_type(self.type_table[&dest].clone()),
                        );
                    };
                    &format!("\tset{} {}b\n", get_cond(&cond), self.get_value(&dest))
                }
            };
            self.asm.push_str(string);
        }
    }

    fn get_value(&self, value: &String) -> String {
        if self.symbol_table.contains_key(value) {
            self.symbol_table[value].clone()
        } else {
            value.to_string()
        }
    }

    fn insert_reg(&mut self, dest: String, size: usize) {
        let num_regs = self.symbol_table.len();

        if num_regs < self.regs.len() {
            self.symbol_table
                .insert(dest.to_string(), self.regs[num_regs].to_string());
        } else {
            self.asm.push_str(format!("\tsub rsp, {}\n", size).as_str());
            self.symbol_table.insert(
                dest.to_string(),
                format!(
                    "{} [rbp-{}]",
                    get_word_for_size(size),
                    self.current_stack + size
                ),
            );
            self.current_stack += size;
        }
    }
}

fn get_word_for_size(size: usize) -> String {
    match size {
        1 => "byte".to_string(),
        2 => "word".to_string(),
        4 => "dword".to_string(),
        8 => "qword".to_string(),
        _ => panic!("Invalid size."),
    }
}

fn get_size_for_type(type_to_check: String) -> usize {
    match type_to_check.as_str() {
        "int" => 8,
        "bool" => 1,
        _ => panic!("Invalid type."),
    }
}
