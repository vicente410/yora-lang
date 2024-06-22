use crate::parser::Expression;
use std::{collections::HashMap, ops::Deref, process};

pub fn analyze(ast: &Vec<Expression>) {
    let mut variables: HashMap<String, String> = HashMap::new();
    for expr in ast {
        analyze_expression(expr, &mut variables);
    }
}

fn analyze_expression(expr: &Expression, vars: &mut HashMap<String, String>) {
    match expr {
        Expression::Declare(ref dest, ref src) => {
            analyze_expression(src, vars);

            let src_val = get_value(src);
            let dest_val = get_value(dest);
            if !vars.contains_key(&src_val) && get_type(&src) == "id".to_string() {
                println!("Variable with name {src_val} undeclared");
                process::exit(1);
            }
            if !vars.contains_key(&dest_val) && get_type(&dest) == "id".to_string() {
                vars.insert(dest_val, get_type(src));
            } else {
                println!("{dest_val} it not an identifier 1");
                process::exit(1);
            }
        }
        Expression::Assign(ref dest, ref src) => {
            analyze_expression(src, vars);

            let src_val = get_value(src);
            let dest_val = get_value(dest);
            if !vars.contains_key(&src_val) && get_type(&src) == "id".to_string() {
                println!("Variable with name {src_val} undeclared");
                process::exit(1);
            }
            if !vars.contains_key(&dest_val) && get_type(&dest) == "id".to_string() {
                println!("Variable with name {dest_val} undeclared");
                process::exit(1);
            } else if !vars.contains_key(&dest_val) {
                println!("{dest_val} is not an identifier 2");
                dbg!(dest);
                process::exit(1);
            }
        }

        Expression::Add(ref dest, ref src)
        | Expression::Sub(ref dest, ref src)
        | Expression::Mul(ref dest, ref src)
        | Expression::Div(ref dest, ref src)
        | Expression::Mod(ref dest, ref src) => {
            analyze_expression(src, vars);
            analyze_expression(dest, vars);

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
            if vars.contains_key(&src_val) && vars[&src_val] != "int" {
                println!(
                    "Variable with name {src_val} is not an int. Can't make an operation with it"
                );
                process::exit(1);
            } else if get_type(&src) != "id".to_string() && get_type(&src) != "int".to_string() {
                println!(
                    "Can't make an operation between type int and {}",
                    get_type(&src)
                );
                process::exit(1);
            }
            if vars.contains_key(&dest_val) && vars[&dest_val] != "int" {
                println!(
                    "Variable with name {dest_val} is not an int. Can't make an operation with it"
                );
                process::exit(1);
            } else if get_type(&dest) != "id".to_string() && get_type(&dest) != "int".to_string() {
                println!(
                    "Can't make an operation between type int and {}",
                    get_type(&dest)
                );
                process::exit(1);
            }
        }
        Expression::Sequence(seq) => {
            for expr in &seq[0..seq.len()] {
                analyze_expression(expr, vars);
            }
        }
        Expression::If(cond, seq) => {
            if get_type(cond) != "bool" {
                println!("Given if does not have a boolean type inside it");
                process::exit(1);
            }
            analyze_expression(cond, vars);
            analyze_expression(seq, vars);
        }

        Expression::Eq(cmp1, cmp2)
        | Expression::NotEq(cmp1, cmp2)
        | Expression::Less(cmp1, cmp2)
        | Expression::LessEq(cmp1, cmp2)
        | Expression::Greater(cmp1, cmp2)
        | Expression::GreaterEq(cmp1, cmp2) => {
            analyze_expression(cmp1, vars);
            analyze_expression(cmp2, vars);

            let val1 = get_value(cmp1);
            let val2 = get_value(cmp2);

            if vars.contains_key(&val1) && vars.contains_key(&val2) && vars[&val1] != vars[&val2] {
                println!(
                    "Can't compare type {} with type {}",
                    vars[&val1], vars[&val2]
                );
                process::exit(1);
            } else if vars.contains_key(&val1)
                && !vars.contains_key(&val2)
                && vars[&val1] != get_type(cmp2)
            {
                println!(
                    "Can't compare type {} with type {}",
                    vars[&val1],
                    get_type(cmp2)
                );
                process::exit(1);
            } else if !vars.contains_key(&val1)
                && vars.contains_key(&val2)
                && get_type(cmp1) != vars[&val2]
            {
                println!(
                    "Can't compare type {} with type {}",
                    get_type(cmp1),
                    vars[&val2]
                );
                process::exit(1);
            } else if !vars.contains_key(&val1)
                && !vars.contains_key(&val2)
                && get_type(cmp1) != get_type(cmp2)
            {
                println!(
                    "Can't compare type {} with type {}",
                    get_type(cmp1),
                    get_type(cmp2)
                );
                process::exit(1);
            }
        }
        Expression::Loop(seq) => analyze_expression(seq, vars),
        Expression::Exit(val) => {
            analyze_expression(val, vars);
            if (get_type(val) == "id"
                && vars.contains_key(&get_value(val))
                && vars[&get_value(val)] != "int")
                || get_type(val) != "id" && get_type(val) != "int"
            {
                println!("Exit codes can only be ints");
                process::exit(1);
            }
        }
        Expression::Break
        | Expression::Identifier(..)
        | Expression::IntLit(..)
        | Expression::BoolLit(..) => {}
    }
}
fn get_value(expr: &Expression) -> String {
    match expr {
        Expression::Exit(..) => {
            println!("Can't evaluate exit");
            process::exit(1)
        }
        Expression::Assign(ref _dest, ref _src) | Expression::Declare(ref _dest, ref _src) => {
            println!("Can't evaluate assigns or declarations");
            process::exit(1)
        }
        Expression::Add(ref dest, _)
        | Expression::Sub(ref dest, _)
        | Expression::Mul(ref dest, _)
        | Expression::Div(ref dest, _)
        | Expression::Mod(ref dest, _) => get_value(dest),
        Expression::IntLit(int) => int.to_string(),
        Expression::BoolLit(bool) => bool.to_string(),
        Expression::Identifier(id) => id.to_string(),
        Expression::If(_, seq) | Expression::Loop(seq) => match seq.deref() {
            Expression::Sequence(sequence) => get_type(&sequence[sequence.len() - 1]),
            _ => panic!("Sequence in the if is not a sequence."),
        },
        Expression::Break => "".to_string(),
        Expression::Sequence(seq) => get_type(&seq[seq.len() - 1]),
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
        Expression::Add(ref dest, _)
        | Expression::Sub(ref dest, _)
        | Expression::Mul(ref dest, _)
        | Expression::Div(ref dest, _)
        | Expression::Mod(ref dest, _) => get_type(dest),
        _ => {
            dbg!(&expr);
            panic!("Not a valid type")
        }
    }
}
