use crate::{ir_generation::Ir, JmpType};
use std::collections::HashMap;

pub fn generate_asm(ir: Vec<Ir>) -> String {
    let mut asm = String::from("global _start\n_start:\n\tmov rbp, rsp\n");
    let mut symbol_table: HashMap<String, String> = HashMap::new();
    let mut num_regs = 0;
    let regs = ["rbx", "r10", "r11", "r12", "r13", "r14", "r15"];

    for instruction in ir {
        let string = match instruction {
            Ir::Assign(ref dest, src) => {
                if !symbol_table.contains_key(dest) {
                    if num_regs < regs.len() {
                        symbol_table.insert(dest.to_string(), regs[num_regs].to_string());
                    } else {
                        asm.push_str("\tsub rsp, 8\n");
                        symbol_table.insert(
                            dest.to_string(),
                            format!("qword [rbp-{}]", (num_regs - regs.len() + 1) * 8),
                        );
                    }
                    num_regs += 1;
                }
                &format!(
                    "\tmov {}, {}\n",
                    get_value(dest, &symbol_table),
                    get_value(&src, &symbol_table)
                )
            }
            Ir::Mul(ref dest, ref src) | Ir::Div(ref dest, ref src) => &format!(
                "\tmov rax, {}\n\
                \t{} {}\n\
                \tmov {}, rax",
                get_value(dest, &symbol_table),
                get_operation(&instruction),
                get_value(src, &symbol_table),
                get_value(dest, &symbol_table),
            ),
            Ir::Mod(ref dest, ref src) => &format!(
                "\tmov rax, {}\n\
                \t{} {}\n\
                \tmov {}, rdx",
                get_value(dest, &symbol_table),
                get_operation(&instruction),
                get_value(src, &symbol_table),
                get_value(dest, &symbol_table),
            ),
            Ir::Add(ref dest, ref src) | Ir::Sub(ref dest, ref src) => &format!(
                "\t{} {}, {}\n",
                get_operation(&instruction),
                get_value(dest, &symbol_table),
                get_value(src, &symbol_table)
            ),
            Ir::Label(label) => &format!("{}:\n", label),
            Ir::Jmp(cmp1, cmp2, jmp_label, jmp_type) => {
                if jmp_type == JmpType::Jmp {
                    &format!("\tjmp {}", jmp_label)
                } else {
                    &format!(
                        "\tcmp {}, {}\n\
                        \t{} {}\n",
                        get_value(&cmp1, &symbol_table),
                        get_value(&cmp2, &symbol_table),
                        get_jump(jmp_type),
                        jmp_label
                    )
                }
            }
            Ir::Exit(src) => &format!(
                "\n\tmov rdi, {}\n\
                    \tmov rax, 60\n\
                    \tsyscall\n",
                get_value(&src, &symbol_table),
            ),
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

fn get_jump(jmp_type: JmpType) -> String {
    match jmp_type {
        JmpType::Jmp => "jmp".to_string(),
        JmpType::Je => "je".to_string(),
        JmpType::Jne => "jne".to_string(),
        JmpType::Jl => "jl".to_string(),
        JmpType::Jle => "jle".to_string(),
        JmpType::Jg => "jg".to_string(),
        JmpType::Jge => "jge".to_string(),
    }
}

fn get_operation(operation: &Ir) -> &str {
    match operation {
        Ir::Add(..) => "add",
        Ir::Sub(..) => "sub",
        Ir::Mul(..) => "mul",
        Ir::Div(..) => "div",
        Ir::Mod(..) => "div",
        _ => panic!("Unexpected operation."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asm_generation() {
        let input = vec![
            Ir::Assign("t1".to_string(), "2".to_string()),
            Ir::Assign("t2".to_string(), "3".to_string()),
            Ir::Add("t1".to_string(), "t2".to_string()),
            Ir::Exit("t1".to_string()),
        ];
        let output = "global _start\n\
                    _start:\n\
                        \tmov rbp, rsp\n\
                        \tmov rbx, 2\n\
                        \tmov r10, 3\n\
                        \tadd rbx, r10\n\n\
                        \tmov rdi, rbx\n\
                        \tmov rax, 60\n\
                        \tsyscall\n";
        assert_eq!(generate_asm(input), output);
    }
}
