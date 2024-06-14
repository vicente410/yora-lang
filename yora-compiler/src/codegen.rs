use crate::parser::*;
use std::collections::HashMap;

pub fn codegen(ast: Vec<Statement>) -> String {
    let mut assembly = String::new();
    let mut symbol_table: HashMap<String, String> = HashMap::new();

    assembly.push_str("global _start\n_start:\n");
    for statement in ast {
        match statement {
            Statement::Exit(arg) => {
                assembly.push_str(&format!(
                    "    mov rax, 60\n    mov rdi, {}\n    syscall\n",
                    match arg {
                        Expression::Literal(int) => int,
                        Expression::Identifier(id) => symbol_table[&id].clone(),
                    }
                ));
            }
            Statement::Assignment(id, int) => match (id, int) {
                (Expression::Identifier(id), Expression::Literal(int)) => {
                    symbol_table.insert(id, int);
                }
                _ => {}
            },
        }
    }

    assembly
}
