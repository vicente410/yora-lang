use crate::parser::*;
use std::collections::HashMap;

pub fn codegen(ast: Vec<Statement>) -> String {
    let mut data = String::new();
    let mut text = String::new();
    let mut symbol_table: HashMap<String, String> = HashMap::new();
    let mut num_buffers = 0;

    data.push_str("section .data\n");
    text.push_str("section .text\nglobal _start\n_start:\n");
    for statement in ast {
        match statement {
            Statement::Assignment(arg1, arg2) => match (arg1, arg2) {
                (Expression::Identifier(id), Expression::Literal(int)) => {
                    symbol_table.insert(id, int);
                }
                (Expression::Identifier(id1), Expression::Identifier(id2)) => {
                    symbol_table.insert(id1, symbol_table[&id2].clone());
                }
                _ => {}
            },
            Statement::Exit(arg) => {
                text.push_str(&format!(
                    "    mov rax, 60\n    mov rdi, {}\n    syscall\n\n",
                    match arg {
                        Expression::Literal(int) => int,
                        Expression::Identifier(id) => symbol_table[&id].clone(),
                    }
                ));
            }
            Statement::Print(arg) => {
                // TODO: change buffer size to fit string
                text.push_str(&format!(
                    "    mov rax, 1\n    mov rdi, 1\n    mov rsi, str{}\n    mov rdx, 3\n    syscall\n\n",
                    num_buffers,
                ));
                data.push_str(&format!(
                    "str{}: db \"{}\", 0x0A\n",
                    num_buffers,
                    match arg {
                        Expression::Literal(int) => int,
                        Expression::Identifier(id) => symbol_table[&id].clone(),
                    }
                ));
                num_buffers += 1;
            }
        }
    }

    data + "\n" + &text
}
