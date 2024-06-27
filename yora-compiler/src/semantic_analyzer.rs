use crate::parser::*;
use std::{collections::HashMap, ops::Deref, process};

enum ErrorTypes {
    UndeclaredVariable(String, usize, usize),
    NotAnIdentifier(String, usize, usize),
    CannotMakeOperation(String, String, usize, usize),
    NotACondition(usize, usize),
    InvalidComparison(String, String, usize, usize),
    InvalidExitCode(usize, usize),
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
    match &expr.kind {
        ExpressionKind::Declare(ref dest, ref src) => {
            analyze_expression(src, vars, errors);

            let dest_val = get_value(dest);

            if get_type(src) != "bool" {
                let src_val = get_value(src);

                if !vars.contains_key(&src_val) && get_type(src) == "id" {
                    errors.push(ErrorTypes::UndeclaredVariable(
                        src_val, src.line, src.column,
                    ));
                }
            }

            if !vars.contains_key(&dest_val) && get_type(dest) == "id" {
                vars.insert(dest_val, get_type(src));
            } else {
                errors.push(ErrorTypes::NotAnIdentifier(
                    dest_val,
                    dest.line,
                    dest.column,
                ));
            }
        }
        ExpressionKind::Assign(ref dest, ref src) => {
            analyze_expression(src, vars, errors);

            let dest_val = get_value(dest);

            if get_type(src) != "bool" {
                let src_val = get_value(src);

                if !vars.contains_key(&src_val) && get_type(src) == "id" {
                    errors.push(ErrorTypes::UndeclaredVariable(
                        src_val, src.line, src.column,
                    ));
                }
            }
            if !vars.contains_key(&dest_val) && get_type(dest) == "id" {
                errors.push(ErrorTypes::UndeclaredVariable(
                    dest_val,
                    dest.line,
                    dest.column,
                ));
            } else if !vars.contains_key(&dest_val) {
                errors.push(ErrorTypes::NotAnIdentifier(
                    dest_val,
                    dest.line,
                    dest.column,
                ));
            }
        }

        ExpressionKind::Add(ref dest, ref src)
        | ExpressionKind::Sub(ref dest, ref src)
        | ExpressionKind::Mul(ref dest, ref src)
        | ExpressionKind::Div(ref dest, ref src)
        | ExpressionKind::Mod(ref dest, ref src) => {
            analyze_expression(src, vars, errors);
            analyze_expression(dest, vars, errors);

            let src_val = get_value(src);
            let dest_val = get_value(dest);

            if !vars.contains_key(&src_val) && get_type(src) == "id" {
                errors.push(ErrorTypes::UndeclaredVariable(
                    src_val.clone(),
                    src.line,
                    src.column,
                ));
            }

            if !vars.contains_key(&dest_val) && get_type(dest) == "id" {
                errors.push(ErrorTypes::UndeclaredVariable(
                    dest_val.clone(),
                    dest.line,
                    dest.column,
                ));
            }

            if vars.contains_key(&src_val) && vars[&src_val] != "int" {
                errors.push(ErrorTypes::CannotMakeOperation(
                    vars[&src_val].clone(),
                    "int".to_string(),
                    src.line,
                    src.column,
                ));
            } else if get_type(src) != "id" && get_type(src) != "int" {
                errors.push(ErrorTypes::CannotMakeOperation(
                    get_type(src),
                    "int".to_string(),
                    src.line,
                    src.column,
                ));
            }

            if vars.contains_key(&dest_val) && vars[&dest_val] != "int" {
                errors.push(ErrorTypes::CannotMakeOperation(
                    vars[&dest_val].clone(),
                    "int".to_string(),
                    dest.line,
                    dest.column,
                ));
            } else if get_type(dest) != "id" && get_type(dest) != "int" {
                errors.push(ErrorTypes::CannotMakeOperation(
                    get_type(dest),
                    "int".to_string(),
                    dest.line,
                    dest.column,
                ));
            }
        }
        ExpressionKind::Sequence(seq) => {
            for expr in &seq[0..seq.len()] {
                analyze_expression(expr, vars, errors);
            }
        }
        ExpressionKind::If(ref cond, ref seq) => {
            if get_type(cond) != "bool" && get_type(cond) != "id"
                || (get_type(cond) == "id" && vars[&get_value(cond)] != "bool")
            {
                errors.push(ErrorTypes::NotACondition(cond.line, cond.column));
            }
            analyze_expression(cond, vars, errors);
            analyze_expression(seq, vars, errors);
        }

        ExpressionKind::Eq(ref cmp1, ref cmp2)
        | ExpressionKind::Neq(ref cmp1, ref cmp2)
        | ExpressionKind::Lt(ref cmp1, ref cmp2)
        | ExpressionKind::Leq(ref cmp1, ref cmp2)
        | ExpressionKind::Gt(ref cmp1, ref cmp2)
        | ExpressionKind::Geq(ref cmp1, ref cmp2) => {
            analyze_expression(cmp1, vars, errors);
            analyze_expression(cmp2, vars, errors);

            let val1 = get_value(cmp1);
            let val2 = get_value(cmp2);

            if vars.contains_key(&val1) && vars.contains_key(&val2) && vars[&val1] != vars[&val2] {
                errors.push(ErrorTypes::InvalidComparison(
                    vars[&val1].clone(),
                    vars[&val2].clone(),
                    cmp1.line,
                    cmp1.column,
                ));
            } else if vars.contains_key(&val1)
                && !vars.contains_key(&val2)
                && vars[&val1] != get_type(cmp2)
            {
                errors.push(ErrorTypes::InvalidComparison(
                    vars[&val1].clone(),
                    get_type(cmp2),
                    cmp1.line,
                    cmp1.column,
                ));
            } else if !vars.contains_key(&val1)
                && vars.contains_key(&val2)
                && get_type(cmp1) != vars[&val2]
            {
                errors.push(ErrorTypes::InvalidComparison(
                    get_type(cmp1),
                    vars[&val2].clone(),
                    cmp1.line,
                    cmp1.column,
                ));
            } else if !vars.contains_key(&val1)
                && !vars.contains_key(&val2)
                && get_type(cmp1) != get_type(cmp2)
            {
                errors.push(ErrorTypes::InvalidComparison(
                    get_type(cmp1),
                    get_type(cmp2),
                    cmp1.line,
                    cmp1.column,
                ));
            }
        }
        ExpressionKind::Loop(ref seq) => analyze_expression(seq, vars, errors),
        ExpressionKind::Exit(ref val) => {
            analyze_expression(val, vars, errors);
            if (get_type(val) == "id"
                && vars.contains_key(&get_value(val))
                && vars[&get_value(val)] != "int")
                || (get_type(val) != "id" && get_type(val) != "int")
            {
                errors.push(ErrorTypes::InvalidExitCode(val.line, val.column));
            }
            if get_type(val) == "id" && !vars.contains_key(&get_value(val)) {
                errors.push(ErrorTypes::NotAnIdentifier(
                    get_value(val),
                    val.line,
                    val.column,
                ));
            }
        }
        ExpressionKind::Break
        | ExpressionKind::Identifier(..)
        | ExpressionKind::IntLit(..)
        | ExpressionKind::BoolLit(..)
        | ExpressionKind::Not(..)
        | ExpressionKind::And(..)
        | ExpressionKind::Or(..) => {}
    }
}
fn get_value(expr: &Expression) -> String {
    match expr.kind {
        ExpressionKind::Exit(..) => {
            println!("Can't evaluate exit");
            process::exit(1)
        }
        ExpressionKind::Assign(ref _dest, ref _src)
        | ExpressionKind::Declare(ref _dest, ref _src) => {
            println!("Can't evaluate assigns or declarations");
            process::exit(1)
        }
        ExpressionKind::Add(ref dest, _)
        | ExpressionKind::Sub(ref dest, _)
        | ExpressionKind::Mul(ref dest, _)
        | ExpressionKind::Div(ref dest, _)
        | ExpressionKind::Mod(ref dest, _) => get_value(dest),
        ExpressionKind::IntLit(ref int) => int.to_string(),
        ExpressionKind::BoolLit(ref bool) => bool.to_string(),
        ExpressionKind::Identifier(ref id) => id.to_string(),
        ExpressionKind::If(ref seq, ..) | ExpressionKind::Loop(ref seq) => {
            match &seq.deref().kind {
                ExpressionKind::Sequence(sequence) => get_type(&sequence[sequence.len() - 1]),
                _ => panic!("Sequence in the if is not a sequence."),
            }
        }
        ExpressionKind::Break => "".to_string(),
        ExpressionKind::Sequence(ref seq) => get_type(&seq[seq.len() - 1]),
        _ => {
            dbg!(&expr.kind);
            panic!("Invalid expression")
        }
    }
}

fn get_type(expr: &Expression) -> String {
    match expr.kind {
        ExpressionKind::Identifier(..) => "id".to_string(),
        ExpressionKind::IntLit(..) => "int".to_string(),
        ExpressionKind::BoolLit(..)
        | ExpressionKind::Eq(..)
        | ExpressionKind::Neq(..)
        | ExpressionKind::Lt(..)
        | ExpressionKind::Leq(..)
        | ExpressionKind::Gt(..)
        | ExpressionKind::Geq(..) => "bool".to_string(),
        ExpressionKind::Add(ref dest, ..)
        | ExpressionKind::Sub(ref dest, ..)
        | ExpressionKind::Mul(ref dest, ..)
        | ExpressionKind::Div(ref dest, ..)
        | ExpressionKind::Mod(ref dest, ..) => get_type(dest),
        _ => {
            dbg!(&expr);
            panic!("Not a valid type")
        }
    }
}

fn print_error(error: &ErrorTypes) {
    match error {
        ErrorTypes::UndeclaredVariable(var, line, column) => {
            println!("Variable with name {var} undeclared [line {line}: column {column}]")
        }
        ErrorTypes::NotAnIdentifier(id, line, column) => {
            println!("{id} is not an identifier [line {line}: column {column}]")
        }
        ErrorTypes::CannotMakeOperation(type1, type2, line, column) => {
            println!("Can't make an operation between type {type1} and {type2} [line {line}: column {column}]")
        }
        ErrorTypes::NotACondition(line, column) => {
            println!("Invalid condition for if statement [line {line}: column {column}]")
        }
        ErrorTypes::InvalidComparison(type1, type2, line, column) => {
            println!("Can't compare type {type1} with type {type2} [line {line}: column {column}]")
        }
        ErrorTypes::InvalidExitCode(line, column) => {
            println!("Exit codes can only be ints [line {line}: column {column}]")
        }
    }
}
