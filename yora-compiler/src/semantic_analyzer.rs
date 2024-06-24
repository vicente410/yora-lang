use crate::parser::Expression;
use std::{collections::HashMap, ops::Deref, process};

enum ErrorTypes {
    UndeclaredVariable(String, u32),
    NotAnIdentifier(String, u32),
    CannotMakeOperation(String, String, u32),
    NotACondition(u32),
    InvalidComparison(String, String, u32),
    InvalidExitCode(u32),
}

pub fn analyze(ast: &Vec<Expression>) {
    let mut variables: HashMap<String, String> = HashMap::new();
    let mut errors: Vec<ErrorTypes> = Vec::new();

    for expr in ast {
        analyze_expression(expr, &mut variables, &mut errors);
    }

    for error in &errors {
        print_error(error);
    }

    if !errors.is_empty() {
        process::exit(1);
    }
}

fn analyze_expression(
    expr: &Expression,
    vars: &mut HashMap<String, String>,
    errors: &mut Vec<ErrorTypes>,
) {
    match expr {
        Expression::Declare(ref dest, ref src) => {
            analyze_expression(src, vars, errors);

            let src_val = get_value(src);
            let dest_val = get_value(dest);

            if !vars.contains_key(&src_val) && get_type(src) == "id" {
                errors.push(ErrorTypes::UndeclaredVariable(src_val, 69));
            }

            if !vars.contains_key(&dest_val) && get_type(dest) == "id" {
                vars.insert(dest_val, get_type(src));
            } else {
                errors.push(ErrorTypes::NotAnIdentifier(dest_val, 69));
            }
        }
        Expression::Assign(ref dest, ref src) => {
            analyze_expression(src, vars, errors);

            let src_val = get_value(src);
            let dest_val = get_value(dest);

            if !vars.contains_key(&src_val) && get_type(src) == "id" {
                errors.push(ErrorTypes::UndeclaredVariable(src_val, 69));
            }

            if !vars.contains_key(&dest_val) && get_type(dest) == "id" {
                errors.push(ErrorTypes::UndeclaredVariable(dest_val, 69));
            } else if !vars.contains_key(&dest_val) {
                errors.push(ErrorTypes::NotAnIdentifier(dest_val, 69));
            }
        }

        Expression::Add(ref dest, ref src)
        | Expression::Sub(ref dest, ref src)
        | Expression::Mul(ref dest, ref src)
        | Expression::Div(ref dest, ref src)
        | Expression::Mod(ref dest, ref src) => {
            analyze_expression(src, vars, errors);
            analyze_expression(dest, vars, errors);

            let src_val = get_value(src);
            let dest_val = get_value(dest);

            if !vars.contains_key(&src_val) && get_type(src) == "id" {
                errors.push(ErrorTypes::UndeclaredVariable(src_val.clone(), 69));
            }

            if !vars.contains_key(&dest_val) && get_type(dest) == "id" {
                errors.push(ErrorTypes::UndeclaredVariable(dest_val.clone(), 69));
            }

            if vars.contains_key(&src_val) && vars[&src_val] != "int" {
                errors.push(ErrorTypes::CannotMakeOperation(
                    vars[&src_val].clone(),
                    "int".to_string(),
                    69,
                ));
            } else if get_type(src) != "id" && get_type(src) != "int" {
                errors.push(ErrorTypes::CannotMakeOperation(
                    get_type(src),
                    "int".to_string(),
                    69,
                ));
            }

            if vars.contains_key(&dest_val) && vars[&dest_val] != "int" {
                errors.push(ErrorTypes::CannotMakeOperation(
                    vars[&dest_val].clone(),
                    "int".to_string(),
                    69,
                ));
            } else if get_type(dest) != "id" && get_type(dest) != "int" {
                errors.push(ErrorTypes::CannotMakeOperation(
                    get_type(dest),
                    "int".to_string(),
                    69,
                ));
            }
        }
        Expression::Sequence(seq) => {
            for expr in &seq[0..seq.len()] {
                analyze_expression(expr, vars, errors);
            }
        }
        Expression::If(cond, seq) => {
            if get_type(cond) != "bool" {
                errors.push(ErrorTypes::NotACondition(69));
            }
            analyze_expression(cond, vars, errors);
            analyze_expression(seq, vars, errors);
        }

        Expression::Eq(cmp1, cmp2)
        | Expression::NotEq(cmp1, cmp2)
        | Expression::Less(cmp1, cmp2)
        | Expression::LessEq(cmp1, cmp2)
        | Expression::Greater(cmp1, cmp2)
        | Expression::GreaterEq(cmp1, cmp2) => {
            analyze_expression(cmp1, vars, errors);
            analyze_expression(cmp2, vars, errors);

            let val1 = get_value(cmp1);
            let val2 = get_value(cmp2);

            if vars.contains_key(&val1) && vars.contains_key(&val2) && vars[&val1] != vars[&val2] {
                errors.push(ErrorTypes::InvalidComparison(
                    vars[&val1].clone(),
                    vars[&val2].clone(),
                    69,
                ));
            } else if vars.contains_key(&val1)
                && !vars.contains_key(&val2)
                && vars[&val1] != get_type(cmp2)
            {
                errors.push(ErrorTypes::InvalidComparison(
                    vars[&val1].clone(),
                    get_type(cmp2),
                    69,
                ));
            } else if !vars.contains_key(&val1)
                && vars.contains_key(&val2)
                && get_type(cmp1) != vars[&val2]
            {
                errors.push(ErrorTypes::InvalidComparison(
                    get_type(cmp1),
                    vars[&val2].clone(),
                    69,
                ));
            } else if !vars.contains_key(&val1)
                && !vars.contains_key(&val2)
                && get_type(cmp1) != get_type(cmp2)
            {
                errors.push(ErrorTypes::InvalidComparison(
                    get_type(cmp1),
                    get_type(cmp2),
                    69,
                ));
            }
        }
        Expression::Loop(seq) => analyze_expression(seq, vars, errors),
        Expression::Exit(val) => {
            analyze_expression(val, vars, errors);
            if (get_type(val) == "id"
                && vars.contains_key(&get_value(val))
                && vars[&get_value(val)] != "int")
                || get_type(val) != "id" && get_type(val) != "int"
            {
                errors.push(ErrorTypes::InvalidExitCode(69));
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

fn print_error(error: &ErrorTypes) {
    match error {
        ErrorTypes::UndeclaredVariable(var, line) => {
            println!("Variable with name {var} undeclared [line {line}]")
        }
        ErrorTypes::NotAnIdentifier(id, line) => {
            println!("{id} is not an identifier [line {line}]")
        }
        ErrorTypes::CannotMakeOperation(type1, type2, line) => {
            println!("Can't make an operation between type {type1} and {type2} [line {line}]")
        }
        ErrorTypes::NotACondition(line) => {
            println!("Invalid condition for if statement [line {line}]")
        }
        ErrorTypes::InvalidComparison(type1, type2, line) => {
            println!("Can't compare type {type1} with type {type2} [line {line}]")
        }
        ErrorTypes::InvalidExitCode(line) => {
            println!("Exit codes can only be ints [line {line}]")
        }
    }
}
