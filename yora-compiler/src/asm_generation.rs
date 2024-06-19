use crate::ir_generation::Ir;
use std::collections::HashMap;

pub fn generate_asm(ir: Vec<Ir>) -> String {
    let mut asm = String::from("global _start\n_start:\n");
    let mut symbol_table: HashMap<String, String> = HashMap::new();
    let mut num_regs = 0;
    let regs = ["rax", "rbx", "rcx", "rdx"];

    for instruction in ir {
        let string = match instruction {
            Ir::Assign(ref dest, src) => {
                symbol_table.insert(dest.to_string(), regs[num_regs].to_string());
                num_regs += 1;
                &format!(
                    "\tmov {}, {}\n",
                    get_value(dest, &symbol_table),
                    get_value(&src, &symbol_table)
                )
            }
            Ir::Add(ref dest, ref src)
            | Ir::Sub(ref dest, ref src)
            | Ir::Mul(ref dest, ref src)
            | Ir::Div(ref dest, ref src)
            | Ir::Mod(ref dest, ref src) => &format!(
                "\t{} {}, {}\n",
                get_operation(&instruction),
                get_value(dest, &symbol_table),
                get_value(src, &symbol_table)
            ),
            Ir::Exit(src) => &format!(
                "\n\tmov rdi, {}\n\
                    \tmov rax, 60\n\
                    \tsyscall",
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

fn get_operation(operation: &Ir) -> &str {
    match operation {
        Ir::Add(..) => "add",
        Ir::Sub(..) => "sub",
        Ir::Mul(..) => "mul",
        Ir::Div(..) => "div",
        Ir::Mod(..) => "mod",
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
                        \tmov rax, 2\n\
                        \tmov rbx, 3\n\
                        \tadd rax, rbx\n\n\
                        \tmov rdi, rax\n\
                        \tmov rax, 60\n\
                        \tsyscall";
        assert_eq!(generate_asm(input), output);
    }
}
