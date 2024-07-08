use std::collections::HashMap;

use crate::core::PrimitiveType;
use crate::ir_gen::ir::*;
use crate::op::Op;

struct AsmGenerator {
    asm_data: String,
    asm_text: String,
    symbol_table: HashMap<String, String>,
    type_table: HashMap<String, PrimitiveType>,
    current_stack: usize,
    num_params: usize,
    current_param_stack: usize,
}

pub fn generate_asm(ir: Ir, type_table: &mut HashMap<String, PrimitiveType>) -> String {
    let mut generator = AsmGenerator {
        asm_data: String::from("section .data\n"),
        asm_text: String::from("section .text\nglobal _start\n_start:\n\tmov rbp, rsp\n"),
        symbol_table: HashMap::new(),
        type_table: type_table.clone(),
        current_stack: 0,
        num_params: 0,
        current_param_stack: 0,
    };

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
                    let id = self.get_var(dest);
                    if !self.symbol_table.contains_key(&id) {
                        self.insert_reg(id.clone(), self.get_var_size(dest));
                    }
                    match op {
                        Op::Add | Op::Sub | Op::And | Op::Or => {
                            self.get_simple_op(dest, src1, src2, op)
                        }
                        Op::Mul => self.get_mul(dest, src1, src2),
                        Op::Div | Op::Mod => self.get_div_or_mod(dest, src1, src2, op),
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
                IrInstruction::Ret { src } => self.get_ret(&src),
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
        let id = self.get_var(dest);
        if !self.symbol_table.contains_key(&id) {
            self.insert_reg(id.clone(), self.get_var_size(dest));
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
        let id = self.get_var(dest);
        if !self.symbol_table.contains_key(&id) {
            self.insert_reg(id.clone(), self.get_var_size(dest));
        }

        format!(
            "\tmov {2}, {1}\n\
            \tmov {0}, {1}\n\
            \tnot {0}\n\
            \tand {0}, 1\n",
            self.get_value(dest),
            self.get_value(src),
            Self::get_reg_with_size("rax".to_string(), self.get_var_size(dest)),
        )
    }

    fn get_simple_op(&mut self, dest: &Value, src1: &Value, src2: &Value, op: &Op) -> String {
        let op_str = match op {
            Op::Add => "add",
            Op::Sub => "sub",
            Op::And => "and",
            Op::Or => "or",
            _ => panic!("Must be 'add', 'sub', 'and' or 'or' operation"),
        };

        let dest_val = self.get_value(dest);
        let src1_val = self.get_value(src1);
        let src2_val = self.get_value(src2);
        let acc = Self::get_reg_with_size("rax".to_string(), self.get_var_size(dest));

        if dest == src1 {
            format!("\t{op_str} {dest_val}, {src2_val}\n")
        } else {
            format!(
                "\tmov {acc}, {src1_val}\n\
                \t{op_str} {acc}, {src2_val}\n\
                \tmov {dest_val}, {acc}\n",
            )
        }
    }

    fn get_mul(&mut self, dest: &Value, src1: &Value, src2: &Value) -> String {
        let dest_val = self.get_value(dest);
        let src1_val = self.get_value(src1);
        let src2_val = self.get_value(src2);
        let acc = Self::get_reg_with_size("rax".to_string(), self.get_var_size(dest));

        if dest == src1 {
            format!(
                "\tmov {acc}, {src2_val}\n\
                \tmul {dest_val}\n\
                \tmov {dest_val}, {acc}\n",
            )
        } else if dest == src2 {
            format!(
                "\tmov {acc}, {src1_val}\n\
                \tmul {dest_val}\n\
                \tmov {dest_val}, {acc}\n",
            )
        } else {
            format!(
                "\tmov {acc}, {src1_val}\n\
                \tmov {dest_val}, {acc}\n\
                \tmov {acc}, {src2_val}\n\
                \tmul {dest_val}\n\
                \tmov {dest_val}, {acc}\n",
            )
        }
    }

    fn get_div_or_mod(&mut self, dest: &Value, src1: &Value, src2: &Value, op: &Op) -> String {
        let dest_val = self.get_value(dest);
        let src1_val = self.get_value(src1);
        let acc = Self::get_reg_with_size("rax".to_string(), self.get_var_size(dest));

        let result_reg = if *op == Op::Div {
            acc.clone()
        } else {
            Self::get_reg_with_size("rdx".to_string(), self.get_var_size(dest))
        };

        format!(
            "\tmov rax, 0\n\
                \tmov rdx, 0\n\
                \tmov {acc}, {src1_val}\n\
                \tdiv {0}\n\
                \tmov {dest_val}, {result_reg}\n",
            Self::get_reg_with_size(self.get_value(src2), 8),
        )
    }

    fn get_cmp(&mut self, dest: &Value, src1: &Value, src2: &Value, op: &Op) -> String {
        let dest_val = self.get_value(dest);
        let src1_val = self.get_value(src1);
        let src2_val = self.get_value(src2);

        let id = self.get_var(dest);
        if !self.symbol_table.contains_key(&id) {
            self.insert_reg(id.clone(), self.get_var_size(dest));
        }

        self.asm_text
            .push_str(&format!("\tcmp {}, {}\n", src1_val, src2_val,));

        format!("\tset{} {}\n", get_relation_str(op), dest_val)
    }

    fn get_if_goto(&mut self, src1: &Value, src2: &Value, cond: Op, label: &String) -> String {
        format!(
            "\tcmp {}, {}\n\
            \tj{} {}\n",
            self.get_value(src1),
            self.get_value(src2),
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
                    self.get_value(src)
                } else {
                    type_size = self.get_var_size(src);
                    self.get_value(src)
                }
            }
            Value::MemPos { .. } => {
                type_size = self.get_var_size(src);
                self.get_value(src)
            }
        };

        if self.num_params < 7 {
            format!(
                "\tmov {}, {}\n",
                match self.num_params {
                    1 => Self::get_reg_with_size("rdi".to_string(), type_size),
                    2 => Self::get_reg_with_size("rsi".to_string(), type_size),
                    3 => Self::get_reg_with_size("rdx".to_string(), type_size),
                    4 => Self::get_reg_with_size("rcx".to_string(), type_size),
                    5 => Self::get_reg_with_size("r8".to_string(), type_size),
                    6 => Self::get_reg_with_size("r9".to_string(), type_size),
                    _ => unreachable!(),
                },
                reg
            )
        } else {
            self.current_param_stack += type_size;
            format!("\tpush {}\n", reg)
        }
    }

    fn get_ret(&mut self, src: &Value) -> String {
        let ret = format!(
            "\tmov {}, {}\n\
                \tret {}",
            Self::get_reg_with_size("rax".to_string(), self.get_var_size(src)),
            self.get_value(src),
            self.current_param_stack
        );
        self.current_param_stack = 0;
        ret
    }

    fn get_value(&mut self, value: &Value) -> String {
        match value {
            Value::MemPos { id, offset } => {
                let offset_val = self.get_value(offset); // register where it is

                if offset_val.contains('[') {
                    let acc = Self::get_reg_with_size("rax".to_string(), self.get_var_size(offset));
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

    fn get_var(&self, value: &Value) -> String {
        match value {
            Value::MemPos { id, .. } => id.to_string(),
            Value::Identifier { id } => id.to_string(),
            Value::Constant { .. } => panic!("Invalid variable"),
        }
    }

    fn insert_reg(&mut self, dest: String, size: usize) {
        let num_regs = self.symbol_table.len();
        let next_reg = self.get_next_reg(num_regs, size);

        // number of work registers
        if num_regs >= 7 {
            self.current_stack += size;
        }

        self.symbol_table.insert(dest.to_string(), next_reg);
    }

    fn get_next_reg(&mut self, num_regs: usize, size: usize) -> String {
        Self::get_reg_with_size(
            if num_regs == 0 {
                "rbx".to_string()
            } else if num_regs < 7 {
                format!("r{}", num_regs + 9)
            } else {
                format!("{} [rbp-{}]", get_word_for_size(size), self.current_stack)
            },
            size,
        )
    }

    fn get_reg_with_size(reg: String, size: usize) -> String {
        if reg.parse::<i32>().is_ok() {
            return reg;
        } else if reg.contains("[") {
            let split_reg: Vec<&str> = reg.split(' ').collect();

            return reg.replace(split_reg[0], &get_word_for_size(size));
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

    fn get_var_size(&self, var: &Value) -> usize {
        self.type_table[&self.get_var(var)].clone().get_size()
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
        1 => "byte",
        2 => "word",
        4 => "dword",
        8 => "qword",
        _ => panic!("Invalid size."),
    }
    .to_string()
}
