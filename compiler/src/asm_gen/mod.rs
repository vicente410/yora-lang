use std::collections::HashMap;

use crate::ir_gen::*;
use crate::parser::Op;

struct AsmGenerator {
    asm: String,
    symbol_table: HashMap<String, String>,
    type_table: HashMap<String, String>,
    current_stack: usize,
}

pub fn generate_asm(ir: Vec<Ir>, type_table: &mut HashMap<String, String>) -> String {
    let mut generator = AsmGenerator {
        asm: String::from("global _start\n_start:\n\tmov rbp, rsp\n"),
        symbol_table: HashMap::new(),
        type_table: type_table.clone(),
        current_stack: 0,
    };

    generator.generate_asm(ir);
    generator.asm
}

impl AsmGenerator {
    fn generate_asm(&mut self, ir: Vec<Ir>) {
        for instruction in ir {
            let string = match instruction {
                Ir::Ass { ref dest, ref src } => self.get_assign(dest, src),
                Ir::Not { ref dest, ref src } => self.get_not(dest, src),
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
                        Op::Add | Op::Sub => self.get_add_sub(dest, src1, src2, op),
                        Op::And | Op::Or => self.get_and_or(dest, src1, src2, op),
                        Op::Mul => self.get_mul(dest, src1, src2),
                        Op::Div => self.get_div(dest, src1, src2),
                        Op::Mod => self.get_mod(dest, src1, src2),
                        Op::Eq | Op::Neq | Op::Lt | Op::Leq | Op::Gt | Op::Geq => {
                            self.get_cmp(dest, src1, src2, op)
                        }
                    }
                }
                Ir::Label(label) => format!("{}:\n", label),
                Ir::Goto { label } => format!("\tjmp {}\n", label),
                Ir::IfGoto {
                    src1,
                    src2,
                    cond,
                    label,
                } => self.get_if_goto(&src1, &src2, cond, &label),
                Ir::Param { src } => format!("\tmov rdi, {}\n", self.get_value(&src)),
                Ir::Call { label } => format!("\tcall {}\n", label),
                Ir::Ret { .. } => "".to_string(), // todo
            };
            self.asm.push_str(&string);
        }

        //dbg!(&self.symbol_table);
        //dbg!(&self.type_table);
        self.asm.push_str(
            "\n\tmov rdi, 0\n\
            exit:\n\
            \tmov rax, 60\n\
            \tsyscall\n",
        );
    }

    fn get_assign(&mut self, dest: &String, src: &String) -> String {
        if !self.symbol_table.contains_key(&dest.clone()) {
            self.insert_reg(
                dest.clone(),
                get_size_for_type(self.type_table[dest].clone()),
            );
        }

        let src_val = self.get_value(src);
        let dest_val = self.get_value(dest);

        if src_val.contains('[') && dest_val.contains('[') {
            format!(
                "\tpush rax\n\
                \tmov rax, {}\n\
                \tmov {}, rax\n\
                \tpop rax\n",
                src_val, dest_val,
            )
        } else {
            format!("\tmov {}, {}\n", dest_val, src_val)
        }
    }

    fn get_not(&mut self, dest: &String, src: &String) -> String {
        if !self.symbol_table.contains_key(&dest.clone()) {
            self.insert_reg(
                dest.clone(),
                get_size_for_type(self.type_table[dest].clone()),
            );
        }

        format!(
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

    fn get_add_sub(&mut self, dest: &String, src1: &String, src2: &String, op: &Op) -> String {
        let src_val1 = self.get_value(&src1);
        let src_val2 = self.get_value(&src2);
        let dest_val = self.get_value(&dest);

        format!(
            "\tmov {}, {}\n\
             \t{} {}, {}\n",
            dest_val,
            src_val1,
            match op {
                Op::Add => "add",
                Op::Sub => "sub",
                _ => panic!("Must be add or subtract operation"),
            },
            dest_val,
            src_val2
        )
    }

    fn get_and_or(&mut self, dest: &String, src1: &String, src2: &String, op: &Op) -> String {
        let src_val1 = self.get_value(&src1);
        let src_val2 = self.get_value(&src2);
        let dest_val = self.get_value(&dest);

        format!(
            "\tmov {}, {}\n\
             \t{} {}, {}\n",
            dest_val,
            src_val1,
            match op {
                Op::And => "and",
                Op::Or => "or",
                _ => panic!("Must be add or subtract operation"),
            },
            dest_val,
            src_val2
        )
    }

    fn get_mul(&mut self, dest: &String, src1: &String, src2: &String) -> String {
        if src2 == dest {
            format!(
                "\tmov {}, {}\n\
                \tmov rax, {}\n\
                \tmul {}\n\
                \tmov {}, rax\n",
                self.get_value(dest),
                self.get_value(src2),
                self.get_value(src1),
                self.get_value(dest),
                self.get_value(dest),
            )
        } else {
            format!(
                "\tmov {}, {}\n\
                \tmov rax, {}\n\
                \tmul {}\n\
                \tmov {}, rax\n",
                self.get_value(dest),
                self.get_value(src1),
                self.get_value(src2),
                self.get_value(dest),
                self.get_value(dest),
            )
        }
    }

    fn get_div(&mut self, dest: &String, src1: &String, src2: &String) -> String {
        format!(
            "\tmov {}, {}\n\
            \tmov rax, {}\n\
            \tdiv {}\n\
            \tmov {}, rax\n",
            self.get_value(dest),
            self.get_value(src2),
            self.get_value(src1),
            self.get_value(dest),
            self.get_value(dest),
        )
    }

    fn get_mod(&mut self, dest: &String, src1: &String, src2: &String) -> String {
        format!(
            "\tmov {}, {}\n\
            \tmov rax, {}\n\
            \tmov rdx, 0\n\
            \tdiv {}\n\
            \tmov {}, rdx\n",
            self.get_value(dest),
            self.get_value(src2),
            self.get_value(src1),
            self.get_value(dest),
            self.get_value(dest),
        )
    }

    fn get_cmp(&mut self, dest: &String, src1: &String, src2: &String, op: &Op) -> String {
        if !self.symbol_table.contains_key(dest) {
            self.insert_reg(
                dest.clone(),
                get_size_for_type(self.type_table[dest].clone()),
            );
        };

        let dest = self.get_value(dest);

        self.asm.push_str(&format!(
            "\tcmp {}, {}\n",
            self.get_value(src1),
            self.get_value(src2),
        ));

        if self.get_value(&dest).contains("rbp") {
            format!("\tset{} {}\n", get_rel_op(op), dest)
        //} else if self.get_value(&dest).contains("rbx") {
        //    format!("\tset{} bl\n", get_rel_op(op))
        } else {
            format!("\tset{} {}\n", get_rel_op(op), dest)
        }
    }

    fn get_if_goto(&self, src1: &String, src2: &String, cond: Cond, label: &String) -> String {
        format!(
            "\tcmp {}, {}\n\
            \tj{} {}\n",
            self.get_value(&src1),
            self.get_value(&src2),
            get_cond_str(cond),
            label
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

        // number of work registers
        if num_regs < 7 {
            self.symbol_table
                .insert(dest.to_string(), self.get_reg_with_size(num_regs, size));
        } else {
            //self.asm.push_str(format!("\tsub rsp, {}\n", size).as_str());
            self.current_stack += size;
            self.symbol_table.insert(
                dest.to_string(),
                format!("{} [rbp-{}]", get_word_for_size(size), self.current_stack),
            );
        }
    }

    fn get_reg_with_size(&self, num_regs: usize, size: usize) -> String {
        if num_regs == 0 {
            match size {
                8 => "rbx",
                4 => "ebx",
                2 => "bx",
                1 => "bl",
                _ => panic!("Invalid register size"),
            }
            .to_string()
        } else {
            format!(
                "r{}{}",
                num_regs + 9,
                match size {
                    8 => "",
                    4 => "d",
                    2 => "w",
                    1 => "b",
                    _ => panic!("Invalid register size"),
                },
            )
        }
    }

    /*fn get_instruction_op(
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
    }*/
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

fn get_cond_str(cond: Cond) -> String {
    match cond {
        Cond::Eq => "e",
        Cond::Neq => "ne",
        Cond::Lt => "l",
        Cond::Leq => "le",
        Cond::Gt => "g",
        Cond::Geq => "ge",
    }
    .to_string()
}
