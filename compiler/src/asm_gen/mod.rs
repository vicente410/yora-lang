use std::collections::HashMap;

use crate::ir_gen::*;
use crate::parser::Op;

struct AsmGenerator {
    asm: String,
    symbol_table: HashMap<String, String>,
    type_table: HashMap<String, String>,
    regs: [&'static str; 7],
    current_stack: usize,
}

pub fn generate_asm(ir: Vec<Ir>, type_table: &mut HashMap<String, String>) -> String {
    let mut generator = AsmGenerator {
        asm: String::from("global _start\n_start:\n\tmov rbp, rsp\n"),
        symbol_table: HashMap::new(),
        type_table: type_table.clone(),
        regs: ["rbx", "r10", "r11", "r12", "r13", "r14", "r15"],
        current_stack: 0,
    };

    generator.generate_asm(ir);
    generator.asm
}

fn get_rel_op(op: &Op) -> &str {
    match op {
        Op::Eq => "e",
        Op::Neq => "ne",
        Op::Lt => "l",
        Op::Leq => "le",
        Op::Gt => "g",
        Op::Geq => "ge",
        _ => panic!("Not a relational operator"),
    }
}

fn get_arit_op(op: &Op) -> &str {
    match op {
        Op::Add => "add",
        Op::Sub => "sub",
        Op::Mul => "mul",
        Op::Div => "div",
        Op::Mod => "div",
        Op::And => "and",
        Op::Or => "or",
        _ => panic!("Not an arithmetric operator"),
    }
}

impl AsmGenerator {
    fn generate_asm(&mut self, ir: Vec<Ir>) {
        for instruction in ir {
            let string = match instruction {
                Ir::Ass { ref dest, ref src } => {
                    if !self.symbol_table.contains_key(&dest.clone()) {
                        self.insert_reg(
                            dest.clone(),
                            get_size_for_type(self.type_table[dest].clone()),
                        );
                    }

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
                Ir::Not { ref dest, ref src } => {
                    if !self.symbol_table.contains_key(&dest.clone()) {
                        self.insert_reg(
                            dest.clone(),
                            get_size_for_type(self.type_table[dest].clone()),
                        );
                    }

                    &format!(
                        // todo: does not account yet for stack, find generic way of dealing
                        // with stack operations instead of this mess
                        "\tmov {}, {}\n\
                            \tnot {}\n\
                            \tand {}, 1\n",
                        self.get_value(dest),
                        self.get_value(src),
                        self.get_value(dest),
                        self.get_value(dest),
                    )
                }
                Ir::Op {
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
                        Op::Add | Op::Sub | Op::And | Op::Or => &self.get_instruction_op(
                            src1.to_string(),
                            src2.to_string(),
                            dest.to_string(),
                            &instruction,
                        ),
                        Op::Mul | Op::Div => &format!(
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
                        Op::Mod => &format!(
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
                        Op::Eq | Op::Neq | Op::Lt | Op::Leq | Op::Gt | Op::Geq => {
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

        if let Ir::Op { op, .. } = instruction {
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
