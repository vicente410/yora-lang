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
            Statement::Assignment(arg1, expr) => {
                if let Expression::Value(Value::Identifier(id)) = arg1 {
                    symbol_table.insert(id, eval_expr(expr, &symbol_table));
                } else {
                    panic!("Cannot assign to this expression.");
                }
            }
            Statement::Exit(expr) => {
                text.push_str(&format!(
                    "\tmov rax, 60\n\
                     \tmov rdi, {}\n\
                     \tsyscall\n\n",
                    eval_expr(expr, &symbol_table)
                ));
            }
            Statement::Print(expr) => {
                // TODO: change buffer size to fit string
                text.push_str(&format!(
                    "\tpush rax\n\
                     \tmov rax, 1\n\
                     \tmov rdi, 1\n\
                     \tmov rsi, str{}\n\
                     \tmov rdx, 3\n\
                     \tsyscall\n\
                     \tpop rax\n\n",
                    num_buffers,
                ));
                data.push_str(&format!(
                    "str{}: db \"{}\", 0x0A\n",
                    num_buffers,
                    eval_expr(expr, &symbol_table)
                ));
                num_buffers += 1;
            }
        }
    }

    data + "\n" + &text
}

fn eval_expr(expr: Expression, symbol_table: &HashMap<String, String>) -> String {
    match expr {
        Expression::Value(val) => match val {
            Value::Identifier(id) => symbol_table[&id].clone(),
            Value::Integer(int) => int,
        },
        Expression::Add(expr1, expr2) => {
            let val1: i32 = eval_expr(*expr1, symbol_table).parse().unwrap();
            let val2: i32 = eval_expr(*expr2, symbol_table).parse().unwrap();
            (val1 + val2).to_string()
        }
        Expression::Sub(expr1, expr2) => {
            let val1: i32 = eval_expr(*expr1, symbol_table).parse().unwrap();
            let val2: i32 = eval_expr(*expr2, symbol_table).parse().unwrap();
            (val1 - val2).to_string()
        }
        Expression::Mul(expr1, expr2) => {
            let val1: i32 = eval_expr(*expr1, symbol_table).parse().unwrap();
            let val2: i32 = eval_expr(*expr2, symbol_table).parse().unwrap();
            (val1 * val2).to_string()
        }
        Expression::Div(expr1, expr2) => {
            let val1: i32 = eval_expr(*expr1, symbol_table).parse().unwrap();
            let val2: i32 = eval_expr(*expr2, symbol_table).parse().unwrap();
            (val1 / val2).to_string()
        }
        Expression::Rem(expr1, expr2) => {
            let val1: i32 = eval_expr(*expr1, symbol_table).parse().unwrap();
            let val2: i32 = eval_expr(*expr2, symbol_table).parse().unwrap();
            (val1 % val2).to_string()
        }
    }
}
