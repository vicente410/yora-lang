use std::collections::HashMap;

use crate::ir_generation::*;

pub fn generate_asm(ir: Vec<Ir>) -> String {
    let mut asm = String::from("global _start\n_start:\n\tmov rbp, rsp\n");
    let mut symbol_table: HashMap<String, String> = HashMap::new();
    let regs = ["rbx", "r10", "r11", "r12", "r13", "r14", "r15"];

    for instruction in ir {
        let string = match instruction {
            Ir::Op {
                ref dest,
                ref src,
                ref op,
            } => match op {
                Op::Assign => {
                    if !symbol_table.contains_key(dest) {
                        insert_reg(dest.clone(), regs, &mut symbol_table, &mut asm);
                    }
                    &get_instruction(
                        get_value(src, &symbol_table),
                        get_value(dest, &symbol_table),
                        &instruction,
                    )
                }
                Op::Mul | Op::Div => &format!(
                    "\tmov rax, {}\n\
                    \t{} {}\n\
                    \tmov {}, rax\n",
                    get_value(dest, &symbol_table),
                    get_operation(op),
                    get_value(src, &symbol_table),
                    get_value(dest, &symbol_table),
                ),
                Op::Mod => &format!(
                    "\tmov rax, {}\n\
                    \t{} {}\n\
                    \tmov {}, rdx\n",
                    get_value(dest, &symbol_table),
                    get_operation(op),
                    get_value(src, &symbol_table),
                    get_value(dest, &symbol_table),
                ),
                Op::Add | Op::Sub | Op::Not | Op::Cmp | Op::And | Op::Or => &get_instruction(
                    get_value(src, &symbol_table),
                    get_value(dest, &symbol_table),
                    &instruction,
                ),
            },
            Ir::Label(label) => &format!("{}:\n", label),
            Ir::Jmp { label } => &format!("\tjmp {}\n", label),
            Ir::JmpCond { src, label } => &format!(
                "\tcmp {}, 0\n\
                    \tje {}\n",
                get_value(&src, &symbol_table),
                label
            ),
            Ir::Exit { src } => &format!(
                "\tmov rdi, {}\n\
                    \tmov rax, 60\n\
                    \tsyscall\n",
                get_value(&src, &symbol_table),
            ),
            Ir::Set { dest, cond } => {
                if !symbol_table.contains_key(&dest) {
                    insert_reg(dest.clone(), regs, &mut symbol_table, &mut asm);
                };
                &format!(
                    "\tset{} {}b\n",
                    get_cond(&cond),
                    get_value(&dest, &symbol_table)
                )
            }
        };
        asm.push_str(string);
    }

    asm
}

fn get_value(value: &String, symbol_table: &HashMap<String, String>) -> String {
    if symbol_table.contains_key(value) {
        symbol_table[value].clone()
    } else {
        value.to_string()
    }
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

fn insert_reg(
    dest: String,
    regs: [&str; 7],
    symbol_table: &mut HashMap<String, String>,
    asm: &mut String,
) {
    let num_regs = symbol_table.len();

    if num_regs < regs.len() {
        symbol_table.insert(dest.to_string(), regs[num_regs].to_string());
    } else {
        asm.push_str("\tsub rsp, 8\n");
        symbol_table.insert(
            dest.to_string(),
            format!("qword [rbp-{}]", (num_regs - regs.len()) * 8),
        );
    }
}
