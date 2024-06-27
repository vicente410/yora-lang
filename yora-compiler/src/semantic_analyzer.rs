use crate::parser::*;
use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap},
    ops::Deref,
    process,
};

#[derive(PartialEq, Eq)]
enum ErrorKind {
    UndeclaredVariable { var: String },
    CannotMakeOperation { type1: String, type2: String },
    NotACondition,
    InvalidAssignment { type1: String, type2: String },
    InvalidComparison { type1: String, type2: String },
    InvalidExitCode,
}

#[derive(PartialEq, Eq)]
struct Error {
    kind: ErrorKind,
    line: usize,
    column: usize,
}

impl Ord for Error {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.line, &self.column).cmp(&(other.line, &other.column))
    }
}

impl PartialOrd for Error {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Error {
    fn new(kind: ErrorKind, line: usize, column: usize) -> Error {
        Error { kind, line, column }
    }
}

pub fn analyze(ast: &Vec<Expression>) {
    let mut variables: HashMap<String, String> = HashMap::new();
    let mut errors: BTreeSet<Error> = BTreeSet::new();

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
    errors: &mut BTreeSet<Error>,
) {
    match &expr.kind {
        ExpressionKind::Declare(ref dest, ref src) => {
            analyze_expression(src, vars, errors);
            vars.insert(get_value(dest, vars, errors), get_type(src, vars, errors));
        }
        ExpressionKind::Assign(ref dest, ref src) => {
            analyze_expression(src, vars, errors);

            let type1 = get_type(src, vars, errors);
            let type2 = get_type(dest, vars, errors);

            if type1 != type2 {
                errors.insert(Error::new(
                    ErrorKind::InvalidAssignment { type1, type2 },
                    src.line,
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

            let type1 = get_type(src, vars, errors);
            let type2 = get_type(dest, vars, errors);

            if type1 != "int" || type2 != "int" {
                errors.insert(Error::new(
                    ErrorKind::CannotMakeOperation { type1, type2 },
                    expr.line,
                    expr.column,
                ));
            }
        }
        ExpressionKind::Sequence(seq) => {
            for expr in &seq[0..seq.len()] {
                analyze_expression(expr, vars, errors);
            }
        }
        ExpressionKind::If(ref cond, ref seq) => {
            if get_type(cond, vars, errors) != "bool" {
                errors.insert(Error::new(ErrorKind::NotACondition, cond.line, cond.column));
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

            let type1 = get_type(cmp1, vars, errors);
            let type2 = get_type(cmp2, vars, errors);

            if type1 != type2 {
                errors.insert(Error::new(
                    ErrorKind::InvalidComparison { type1, type2 },
                    cmp1.line,
                    cmp1.column,
                ));
            }
        }
        ExpressionKind::Loop(ref seq) => analyze_expression(seq, vars, errors),
        ExpressionKind::Exit(ref val) => {
            analyze_expression(val, vars, errors);

            if get_type(val, vars, errors) != "int" {
                errors.insert(Error::new(ErrorKind::InvalidExitCode, val.line, val.column));
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
fn get_value(
    expr: &Expression,
    vars: &HashMap<String, String>,
    errors: &mut BTreeSet<Error>,
) -> String {
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
        | ExpressionKind::Mod(ref dest, _) => get_value(dest, vars, errors),
        ExpressionKind::IntLit(ref int) => int.to_string(),
        ExpressionKind::BoolLit(ref bool) => bool.to_string(),
        ExpressionKind::Identifier(ref id) => id.to_string(),
        ExpressionKind::If(ref seq, ..) | ExpressionKind::Loop(ref seq) => {
            match &seq.deref().kind {
                ExpressionKind::Sequence(sequence) => {
                    get_type(&sequence[sequence.len() - 1], vars, errors)
                }
                _ => panic!("Sequence in the if is not a sequence."),
            }
        }
        ExpressionKind::Break => "".to_string(),
        ExpressionKind::Sequence(ref seq) => get_type(&seq[seq.len() - 1], vars, errors),
        _ => {
            dbg!(&expr.kind);
            panic!("Invalid expression")
        }
    }
}

fn get_type(
    expr: &Expression,
    vars: &HashMap<String, String>,
    errors: &mut BTreeSet<Error>,
) -> String {
    match &expr.kind {
        ExpressionKind::Identifier(id) => {
            if vars.contains_key(id) {
                (*vars[id]).to_string()
            } else {
                errors.insert(Error::new(
                    ErrorKind::UndeclaredVariable {
                        var: id.to_string(),
                    },
                    expr.line,
                    expr.column,
                ));
                "null".to_string()
            }
        }
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
        | ExpressionKind::Mod(ref dest, ..) => get_type(dest, vars, errors),
        _ => {
            dbg!(&expr);
            panic!("Not a valid type")
        }
    }
}

fn print_error(error: &Error) {
    match &error.kind {
        ErrorKind::UndeclaredVariable { var } => {
            println!(
                "Variable with name {var} undeclared [line {}: column {}]",
                error.line, error.column
            )
        }
        ErrorKind::CannotMakeOperation { type1, type2 } => {
            println!(
                "Can't make an operation between ints, types {type1} and {type2} were given[line {}: column {}]",
                error.line, error.column
            )
        }
        ErrorKind::NotACondition => {
            println!(
                "Invalid condition for if statement [line {}: column {}]",
                error.line, error.column
            )
        }
        ErrorKind::InvalidComparison { type1, type2 } => {
            println!(
                "Can't compare type {type1} with type {type2} [line {}: column {}]",
                error.line, error.column
            )
        }
        ErrorKind::InvalidExitCode => {
            println!(
                "Exit codes can only be ints [line {}: column {}]",
                error.line, error.column
            )
        }
        ErrorKind::InvalidAssignment { type1, type2 } => {
            println!(
                "Can't assign variable with type {type1} to expression with type {type2} [line {}: column {}]",
                error.line, error.column
            )
        }
    }
}
