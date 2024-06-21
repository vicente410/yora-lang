use crate::parser::Expression;
use core::panic;
use std::{collections::HashMap, process};

pub fn analyse(ast: &Vec<Expression>) {
    let mut variables: HashMap<String, String> = HashMap::new();
    for expr in ast {
        analyze_expression(expr, &mut variables);
    }
}

fn analyze_expression(expr: &Expression, vars: &mut HashMap<String, String>) {
    match expr {
        Expression::Declare(ref dest, ref src) => {
            let src_val = get_value(src);
            let dest_val = get_value(dest);
            if !vars.contains_key(&src_val) && get_type(&src) == "id".to_string() {
                println!("Variable with name {src_val} undeclared");
                process::exit(1);
            }
            if !vars.contains_key(&dest_val) && get_type(&dest) == "id".to_string() {
                vars.insert(dest_val, get_type(src));
            } else {
                println!("{dest_val} it not an identifier");
                process::exit(1);
            }
        }
        Expression::Assign(ref dest, ref src) => {
            let src_val = get_value(src);
            let dest_val = get_value(dest);
            if !vars.contains_key(&src_val) && get_type(&src) == "id".to_string() {
                println!("Variable with name {src_val} undeclared");
                process::exit(1);
            }
            if !vars.contains_key(&dest_val) && get_type(&dest) == "id".to_string() {
                println!("Variable with name {dest_val} undeclared");
                process::exit(1);
            } else {
                println!("{dest_val} is not an identifier");
                process::exit(1);
            }
        }

        Expression::Add(ref dest, ref src)
        | Expression::Sub(ref dest, ref src)
        | Expression::Mul(ref dest, ref src)
        | Expression::Div(ref dest, ref src)
        | Expression::Mod(ref dest, ref src) => {
            let src_val = get_value(src);
            let dest_val = get_value(dest);
            if !vars.contains_key(&src_val) && get_type(&src) == "id".to_string() {
                println!("Variable with name {src_val} undeclared");
                process::exit(1);
            }
            if !vars.contains_key(&dest_val) && get_type(&dest) == "id".to_string() {
                println!("Variable with name {dest_val} undeclared");
                process::exit(1);
            }
            if vars[&src_val] != "int" {
                println!(
                    "Variable with name {src_val} is not an int. Can't make an operation with it"
                );
                process::exit(1);
            }
            if vars[&dest_val] != "int" {
                println!(
                    "Variable with name {dest_val} is not an int. Can't make an operation with it"
                );
                process::exit(1);
            }
        }
        Expression::Sequence(seq) => {
            for expr in &seq[0..seq.len() - 1] {
                analyze_expression(expr, vars);
            }
        }
        Expression::Identifier(..)
        | Expression::IntLit(..)
        | Expression::BoolLit(..)
        | Expression::Eq(..)
        | Expression::NotEq(..)
        | Expression::Less(..)
        | Expression::LessEq(..)
        | Expression::Greater(..)
        | Expression::GreaterEq(..) => {}
        _ => todo!(),
    }
}
fn get_value(expr: &Expression) -> String {
    match expr {
        Expression::Exit(_val) => {
            todo!()
        }
        Expression::Assign(ref _dest, ref _src) | Expression::Declare(ref _dest, ref _src) => {
            todo!()
        }
        Expression::Add(ref _dest, ref _src)
        | Expression::Sub(ref _dest, ref _src)
        | Expression::Mul(ref _dest, ref _src)
        | Expression::Div(ref _dest, ref _src)
        | Expression::Mod(ref _dest, ref _src) => {
            todo!()
        }
        Expression::IntLit(int) => int.to_string(),
        Expression::BoolLit(bool) => bool.to_string(),
        Expression::Identifier(id) => id.to_string(),
        Expression::If(cond, _seq) => match **cond {
            Expression::BoolLit(..) => {
                todo!()
            }
            Expression::Eq(ref _cmp1, ref _cmp2)
            | Expression::NotEq(ref _cmp1, ref _cmp2)
            | Expression::Less(ref _cmp1, ref _cmp2)
            | Expression::LessEq(ref _cmp1, ref _cmp2)
            | Expression::Greater(ref _cmp1, ref _cmp2)
            | Expression::GreaterEq(ref _cmp1, ref _cmp2) => {
                todo!()
            }
            _ => {
                dbg!(cond);
                panic!("Unrecognized boolean expression.")
            }
        },
        Expression::Loop(_seq) => {
            todo!()
        }
        Expression::Break => "".to_string(),
        Expression::Sequence(_seq) => {
            todo!()
        }
        _ => panic!("Invalid expression"),
    }
}

fn get_type(expr: &Expression) -> String {
    match expr {
        Expression::Identifier(..) => "id".to_string(),
        Expression::IntLit(..) => "int".to_string(),
        Expression::BoolLit(..)
        | Expression::Eq(..)
        | Expression::NotEq(..)
        | Expression::Less(..)
        | Expression::LessEq(..)
        | Expression::Greater(..)
        | Expression::GreaterEq(..) => "bool".to_string(),
        _ => {
            dbg!(&expr);
            panic!("Not a valid type")
        }
    }
}
