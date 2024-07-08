use std::collections::HashMap;
use std::ops::Deref;

use crate::ir_gen::ir::*;
use crate::op::Op;

struct AsmGenerator {
    asm_data: String,
    asm_text: String,
    symbol_table: HashMap<String, String>,
    type_table: HashMap<String, String>,
    current_stack: usize,
    num_params: usize,
}

pub fn generate_asm(ir: Ir, type_table: &mut HashMap<String, String>) -> String {
    let mut generator = AsmGenerator {
        asm_data: String::from("section .data\n"),
        asm_text: String::from("section .text\nglobal _start\n_start:\n\tmov rbp, rsp\n"),
        symbol_table: HashMap::new(),
        type_table: type_table.clone(),
        current_stack: 0,
        num_params: 0,
    };

    dbg!(type_table);
    generator.generate_data(ir.data);
    generator.generate_code(ir.code);
    generator.asm_data + "\n" + &generator.asm_text
}

impl AsmGenerator {
    fn generate_data(&mut self, data: Vec<Buffer>) {
        for buffer in data {
            if buffer.contents.contains('\"') {
                self.asm_data
                    .push_str(&format!("{}:\tdb\t{}\n", buffer.label, buffer.contents));
            } else {
                self.asm_data
                    .push_str(&format!("{}:\tdb\t{}\n", buffer.label, buffer.contents));
            }
        }
    }
    fn generate_code(&mut self, ir: Vec<IrInstruction>) {
        for instruction in ir {
            if let IrInstruction::Param { .. } = instruction {
                self.num_params += 1;
            } else {
                self.num_params = 0;
            }

            let string = match instruction {
                IrInstruction::Ass { ref dest, ref src } => self.get_assign(dest, src),
                IrInstruction::Not { ref dest, ref src } => self.get_not(dest, src),
                IrInstruction::Op {
                    ref dest,
                    ref src1,
                    ref op,
                    ref src2,
                } => {
                    if let Value::Identifier { id } = dest {
                        if !self.symbol_table.contains_key(id) {
                            self.insert_reg(
                                id.clone(),
                                get_size_for_type(self.type_table[id].clone()),
                            );
                        }
                    }
                    match op {
                        Op::Add | Op::Sub | Op::And | Op::Or => {
                            self.get_simple_op(dest, src1, src2, op)
                        }
                        Op::Mul => self.get_mul(dest, src1, src2),
                        Op::Div => self.get_div(dest, src1, src2),
                        Op::Mod => self.get_mod(dest, src1, src2),
                        Op::Eq | Op::Neq | Op::Lt | Op::Leq | Op::Gt | Op::Geq => {
                            self.get_cmp(dest, src1, src2, op)
                        }
                    }
                }
                IrInstruction::Label(label) => format!("{}:\n", label),
                IrInstruction::Goto { label } => format!("\tjmp {}\n", label),
                IrInstruction::IfGoto {
                    src1,
                    src2,
                    cond,
                    label,
                } => self.get_if_goto(&src1, &src2, cond, &label),
                IrInstruction::Param { src } => self.get_param(&src),
                IrInstruction::Call { label } => format!("\tcall {}\n", label),
                IrInstruction::Ret { .. } => "".to_string(), // todo
            };
            self.asm_text.push_str(&string);
        }

        self.asm_text.push_str(
            "\n\tmov rdi, 0\n\
            exit:\n\
            \tmov rax, 60\n\
            \tsyscall\n",
        );

        self.asm_text.push_str(
            "\nprint:\n\
            \tmov rax, 1\n\
            \tsyscall\n\
            \tret\n",
        );
    }

    fn get_assign(&mut self, dest: &Value, src: &Value) -> String {
        if let Value::Identifier { id } = dest {
            if !self.symbol_table.contains_key(id) {
                self.insert_reg(id.clone(), get_size_for_type(self.type_table[id].clone()));
            }
        }

        let src_val = self.get_value(src);
        let dest_val = self.get_value(dest);
        let acc = Self::get_reg_with_size("rax".to_string(), 1);

        if src_val.contains('[') && dest_val.contains('[') {
            format!(
                "\tpush rax\n\
                \tmov {acc}, {src_val}\n\
                \tmov {dest_val}, {acc}\n\
                \tpop rax\n",
            )
        } else {
            format!("\tmov {dest_val}, {src_val}\n")
        }
    }

    fn get_not(&mut self, dest: &Value, src: &Value) -> String {
        if let Value::Identifier { id } = dest {
            if !self.symbol_table.contains_key(&id.clone()) {
                self.insert_reg(id.clone(), get_size_for_type(self.type_table[id].clone()));
            }
        }

        format!(
            // todo: does not account yet for stack, find generic way of dealing
            // with stack operations instead of this mess
            "\tmov {0}, {1}\n\
            \tnot {0}\n\
            \tand {0}, 1\n",
            self.get_value(dest),
            self.get_value(src),
        )
    }

    fn get_simple_op(&mut self, dest: &Value, src1: &Value, src2: &Value, op: &Op) -> String {
        format!(
            "\tmov {0}, {1}\n\
             \t{2} {0}, {3}\n",
            self.get_value(&dest),
            self.get_value(&src1),
            match op {
                Op::Add => "add",
                Op::Sub => "sub",
                Op::And => "and",
                Op::Or => "or",
                _ => panic!("Must be 'add', 'sub', 'and' or 'or' operation"),
            },
            self.get_value(&src2),
        )
    }

    fn get_mul(&mut self, dest: &Value, src1: &Value, src2: &Value) -> String {
        if src2 == dest {
            format!(
                "\tmov {0}, {1}\n\
                \tmov {2}, {3}\n\
                \tmul {0}\n\
                \tmov {0}, {2}\n",
                self.get_value(dest),
                self.get_value(src2),
                Self::get_reg_with_size("rax".to_string(), 1),
                self.get_value(src1),
            )
        } else {
            format!(
                "\tmov {0}, {1}\n\
                \tmov {2}, {3}\n\
                \tmul {0}\n\
                \tmov {0}, {2}\n",
                self.get_value(dest),
                self.get_value(src1),
                Self::get_reg_with_size("rax".to_string(), 1),
                self.get_value(src2),
            )
        }
    }

    fn get_div(&mut self, dest: &Value, src1: &Value, src2: &Value) -> String {
        if src1 == dest {
            format!(
                "\tmov rax, 0\n\
                \tmov rdx, 0\n\
                \tmov {3}, {2}\n\
                \tmov {0}, {1}\n\
                \tdiv {4}\n\
                \tmov {0}, {3}\n",
                self.get_value(dest),
                self.get_value(src2),
                self.get_value(src1),
                Self::get_reg_with_size("rax".to_string(), 1),
                Self::get_reg_with_size(self.get_value(dest), 8),
            )
        } else {
            format!(
                "\tmov rax, 0\n\
                \tmov rdx, 0\n\
                \tmov {0}, {1}\n\
                \tmov {3}, {2}\n\
                \tdiv {4}\n\
                \tmov {0}, {3}\n",
                self.get_value(dest),
                self.get_value(src2),
                self.get_value(src1),
                Self::get_reg_with_size("rax".to_string(), 1),
                Self::get_reg_with_size(self.get_value(dest), 8),
            )
        }
    }

    fn get_mod(&mut self, dest: &Value, src1: &Value, src2: &Value) -> String {
        if src1 == dest {
            format!(
                "\tmov rax, 0\n\
                \tmov rdx, 0\n\
                \tmov {3}, {2}\n\
                \tmov {0}, {1}\n\
                \tdiv {4}\n\
                \tmov {0}, {5}\n",
                self.get_value(dest),
                self.get_value(src2),
                self.get_value(src1),
                Self::get_reg_with_size("rax".to_string(), 1),
                Self::get_reg_with_size(self.get_value(dest), 8),
                Self::get_reg_with_size("rdx".to_string(), 1),
            )
        } else {
            format!(
                "\tmov rax, 0\n\
                \tmov rdx, 0\n\
                \tmov {0}, {1}\n\
                \tmov {3}, {2}\n\
                \tdiv {4}\n\
                \tmov {0}, {5}\n",
                self.get_value(dest),
                self.get_value(src2),
                self.get_value(src1),
                Self::get_reg_with_size("rax".to_string(), 1),
                Self::get_reg_with_size(self.get_value(dest), 8),
                Self::get_reg_with_size("rdx".to_string(), 1),
            )
        }
    }

    fn get_cmp(&mut self, dest: &Value, src1: &Value, src2: &Value, op: &Op) -> String {
        let dest_val = self.get_value(dest);
        let src1_val = self.get_value(src1);
        let src2_val = self.get_value(src2);

        if let Value::Identifier { id } = dest {
            if !self.symbol_table.contains_key(id) {
                self.insert_reg(
                    dest_val.clone(),
                    get_size_for_type(self.type_table[id].clone()),
                );
            };
        }

        self.asm_text
            .push_str(&format!("\tcmp {}, {}\n", src1_val, src2_val,));

        format!("\tset{} {}\n", get_relation_str(op), dest_val)
    }

    fn get_if_goto(&mut self, src1: &Value, src2: &Value, cond: Op, label: &String) -> String {
        format!(
            "\tcmp {}, {}\n\
            \tj{} {}\n",
            self.get_value(&src1),
            self.get_value(&src2),
            get_relation_str(&cond),
            label
        )
    }

    fn get_param(&mut self, src: &Value) -> String {
        let mut type_size = 8;

        let reg = match src {
            Value::Constant { value } => {
                if value.parse::<i32>().is_ok() {
                    value.clone()
                } else {
                    panic!("Invalid param")
                }
            }
            Value::Identifier { id } => {
                if !self.symbol_table.contains_key(id) {
                    self.get_value(&src)
                } else {
                    type_size = get_size_for_type(self.type_table[id].clone());
                    self.get_value(&src)
                }
            }
            Value::MemPos { id, .. } => {
                type_size = get_size_for_type(self.type_table[id].clone());
                self.get_value(&src)
            }
        };

        format!(
            "\tmov {}, {}\n",
            match self.num_params {
                1 => Self::get_reg_with_size("rdi".to_string(), type_size),
                2 => Self::get_reg_with_size("rsi".to_string(), type_size),
                3 => Self::get_reg_with_size("rdx".to_string(), type_size),
                4 => Self::get_reg_with_size("rcx".to_string(), type_size),
                5 => Self::get_reg_with_size("r8".to_string(), type_size),
                6 => Self::get_reg_with_size("r9".to_string(), type_size),
                _ => panic!("Too many params"), // todo: implement stack for parameters
            },
            reg
        )
    }

    fn get_value(&mut self, value: &Value) -> String {
        match value {
            Value::MemPos { id, offset } => {
                let offset_val = self.get_value(offset); // register where it is

                if offset_val.contains('[') {
                    // name of ir's variable
                    let offset_var = if let Value::Identifier { id } = offset.deref() {
                        id
                    } else {
                        panic!("Insert useful panic text")
                    };

                    let acc = Self::get_reg_with_size(
                        "rax".to_string(),
                        get_size_for_type(self.type_table[offset_var].clone()),
                    );
                    self.asm_text
                        .push_str(&format!("\tmov {}, {}\n", acc, offset_val));
                    acc
                } else {
                    format!("byte [{} + {}]", id, Self::get_reg_with_size(offset_val, 4))
                }
            }
            Value::Identifier { id } => {
                if self.symbol_table.contains_key(id) {
                    self.symbol_table[id].clone()
                } else {
                    value.to_string()
                }
            }
            Value::Constant { value } => value.to_string(),
        }
    }

    fn insert_reg(&mut self, dest: String, size: usize) {
        let num_regs = self.symbol_table.len();

        // number of work registers
        if num_regs < 7 {
            self.symbol_table
                .insert(dest.to_string(), self.get_next_reg(num_regs, size));
        } else {
            //self.asm.push_str(format!("\tsub rsp, {}\n", size).as_str());
            self.current_stack += size;
            self.symbol_table.insert(
                dest.to_string(),
                format!("{} [rbp-{}]", get_word_for_size(size), self.current_stack),
            );
        }
    }

    fn get_next_reg(&self, num_regs: usize, size: usize) -> String {
        Self::get_reg_with_size(
            if num_regs == 0 {
                "rbx".to_string()
            } else {
                format!("r{}", num_regs + 9)
            },
            size,
        )
    }

    fn get_reg_with_size(reg: String, size: usize) -> String {
        if reg.parse::<i32>().is_ok() {
            return reg;
        } else if reg.contains("[") {
            let split_reg: Vec<&str> = reg.split(' ').collect();

            return reg.replace(
                split_reg[0],
                match size {
                    8 => "qword",
                    4 => "dword",
                    2 => "word",
                    1 => "byte",
                    _ => panic!("Invalid register size"),
                },
            );
        }

        let mut reg_letter = &reg[reg.len() - 2..reg.len() - 1];
        let reg_last = &reg[reg.len() - 1..reg.len()];

        if reg_letter == "i" || reg_last == "i" {
            if reg_last == "l" {
                reg_letter = &reg[reg.len() - 3..reg.len() - 2]
            }
            match size {
                8 => format!("r{}i", reg_letter),
                4 => format!("e{}i", reg_letter),
                2 => format!("{}i", reg_letter),
                1 => format!("{}il", reg_letter),
                _ => panic!("Invalid register size"),
            }
        } else if reg_last.parse::<i32>().is_err() && reg_letter.parse::<i32>().is_err() {
            match size {
                8 => format!("r{}x", reg_letter),
                4 => format!("e{}x", reg_letter),
                2 => format!("{}x", reg_letter),
                1 => format!("{}l", reg_letter),
                _ => panic!("Invalid register size"),
            }
        } else {
            let mut new_reg = reg.clone();

            if let Some(ch) = reg.chars().last() {
                if ch.is_alphabetic() {
                    new_reg.pop();
                }
            }

            format!(
                "{}{}",
                new_reg,
                match size {
                    8 => "",
                    4 => "d",
                    2 => "w",
                    1 => "b",
                    _ => panic!("Invalid register size"),
                }
            )
        }
    }
}

fn get_relation_str(op: &Op) -> &str {
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
        "ptr" => 1,
        "i8" => 1,
        "u8" => 1,
        "i16" => 2,
        "u16" => 2,
        "i32" => 4,
        "u32" => 4,
        "i64" => 8,
        "u64" => 8,
        "int" => 1,
        "bool" => 1,
        _ => panic!("Invalid type."),
    }
}
