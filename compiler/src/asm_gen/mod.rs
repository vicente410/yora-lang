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

fn get_rel_op(op: &Op2) -> &str {
    match op {
        Op2::Eq => "e",
        Op2::Neq => "ne",
        Op2::Lt => "l",
        Op2::Leq => "le",
        Op2::Gt => "g",
        Op2::Geq => "ge",
        _ => panic!("Not a relational operator"),
    }
}

fn get_arit_op(op: &Op2) -> &str {
    match op {
        Op2::Add => "add",
        Op2::Sub => "sub",
        Op2::Mul => "mul",
        Op2::Div => "div",
        Op2::Mod => "div",
        Op2::And => "and",
        Op2::Or => "or",
        _ => panic!("Not an arithmetric operator"),
    }
}

impl Generator {
    fn generate_asm(&mut self, ir: Vec<Ir>) {
        for instruction in ir {
            let string = match instruction {
                Ir::Op1 {
                    ref dest,
                    ref op,
                    ref src,
                } => {
                    if !self.symbol_table.contains_key(&dest.clone()) {
                        self.insert_reg(
                            dest.clone(),
                            get_size_for_type(self.type_table[dest].clone()),
                        );
                    }

                    match op {
                        Op1::Ass => {
                            let src_val = self.get_value(src);
                            let dest_val = self.get_value(dest);

                            if src_val.contains("[") && dest_val.contains("[") {
                                &format!(
                                    "\tpush rax\n\
                                    \tmov rax, {}\n\
                                    \tmov {}, rax\n\
                                    \tpop rax\n",
                                    src_val, dest_val,
                                )
                            } else {
                                &format!("\tmov {}, {}\n", dest_val, src_val)
                            }
                        }
                        Op1::Not => &format!(
                            // todo: does not account yet for stack, find generic way of dealing
                            // with stack operations instead of this mess
                            "\tmov {}, {}\n\
                            \tnot {}\n\
                            \tand {}, 1\n",
                            self.get_value(dest),
                            self.get_value(src),
                            self.get_value(dest),
                            self.get_value(dest),
                        ),
                    }
                }
                Ir::Op2 {
                    ref dest,
                    ref src1,
                    ref op,
                    ref src2,
                } => {
                    if !self.symbol_table.contains_key(dest) {
                        self.insert_reg(
                            dest.clone(),
                            get_size_for_type(self.type_table[dest].clone()),
                        );
                    }

                    match op {
                        Op2::Add | Op2::Sub | Op2::And | Op2::Or => &self.get_instruction_op(
                            src1.to_string(),
                            src2.to_string(),
                            dest.to_string(),
                            &instruction,
                        ),
                        Op2::Mul | Op2::Div => &format!(
                            "\tmov {}, {}\n\
                            \tmov rax, {}\n\
                            \t{} {}\n\
                            \tmov {}, rax\n",
                            self.get_value(dest),
                            self.get_value(src2),
                            self.get_value(src1),
                            get_arit_op(op),
                            self.get_value(dest),
                            self.get_value(dest),
                        ),
                        Op2::Mod => &format!(
                            "\tmov {}, {}\n\
                            \tmov rax, {}\n\
                            \t{} {}\n\
                            \tmov {}, rdx\n",
                            self.get_value(dest),
                            self.get_value(src1),
                            self.get_value(src2),
                            get_arit_op(op),
                            self.get_value(dest),
                            self.get_value(dest),
                        ),
                        Op2::Eq | Op2::Neq | Op2::Lt | Op2::Leq | Op2::Gt | Op2::Geq => {
                            if !self.symbol_table.contains_key(dest) {
                                self.insert_reg(
                                    dest.clone(),
                                    get_size_for_type(self.type_table[dest].clone()),
                                );
                            };

                            let dest = self.get_value(&dest);

                            self.asm.push_str(&format!(
                                "\tcmp {}, {}\n",
                                self.get_value(src1),
                                self.get_value(src2),
                            ));

                            if self.get_value(&dest).contains("rbp") {
                                &format!("\tset{} {}\n", get_rel_op(&op), dest)
                            } else if self.get_value(&dest).contains("rbx") {
                                &format!("\tset{} bl\n", get_rel_op(&op))
                            } else {
                                &format!("\tset{} {}b\n", get_rel_op(&op), dest)
                            }
                        }
                    }
                }
                Ir::Label(label) => &format!("{}:\n", label),
                Ir::Goto { label } => &format!("\tjmp {}\n", label),
                Ir::IfGoto { src, label } => &format!(
                    "\tcmp {}, 0\n\
                    \tje {}\n",
                    self.get_value(&src),
                    label
                ),
                Ir::Param { src } => &format!("\tmov rdi, {}\n", self.get_value(&src)),
                Ir::Call { label } => &format!("\tcall {}\n", label),
                Ir::Ret { .. } => "", // todo
            };
            self.asm.push_str(string);
        }
        self.asm.push_str(
            "\n\tmov rdi, 0\n\
        exit:\n\
            \tmov rax, 60\n\
            \tsyscall\n",
        )
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
            //self.asm.push_str(format!("\tsub rsp, {}\n", size).as_str());
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

    fn get_instruction_op(
        &mut self,
        src1: String,
        src2: String,
        dest: String,
        instruction: &Ir,
    ) -> String {
        let src_val1 = self.get_value(&src1);
        let src_val2 = self.get_value(&src2);
        let dest_val = self.get_value(&dest);

        if let Ir::Op2 { op, .. } = instruction {
            if src_val1.contains('[') && src_val2.contains('[') {
                let reg = if self.type_table[&src1] == "bool" {
                    "al".to_string()
                } else {
                    "rax".to_string()
                };

                format!(
                    "\tpush rax\n\
                    \tmov {}, {}\n\
                    \tmov {}, {}\n\
                    \t{} {}, {}\n\
                    \tpop rax\n",
                    reg,
                    src_val1,
                    dest_val,
                    src_val2,
                    get_arit_op(op),
                    dest_val,
                    reg
                )
            } else {
                format!(
                    "\tmov {}, {}\n\
                     \t{} {}, {}\n",
                    dest_val,
                    src_val1,
                    get_arit_op(op),
                    dest_val,
                    src_val2
                )
            }
        } else {
            panic!();
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
