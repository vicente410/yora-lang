use crate::parser::*;
use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap},
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
    InvalidIdentifier,
}

#[derive(PartialEq, Eq)]
struct Error {
    kind: ErrorKind,
    line: usize,
    col: usize,
}

impl Error {
    fn new(kind: ErrorKind, line: usize, col: usize) -> Error {
        Error { kind, line, col }
    }
}

impl Ord for Error {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.line, &self.col).cmp(&(other.line, &other.col))
    }
}

impl PartialOrd for Error {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct Analyzer {
    errors: BTreeSet<Error>,
    symbol_table: HashMap<String, String>,
}

pub fn analyze(ast: &Vec<Expression>) {
    let mut analyzer = Analyzer::new();

    for expr in ast {
        analyzer.analyze_expression(expr);
    }

    for error in &analyzer.errors {
        print_error(error);
    }

    if !analyzer.errors.is_empty() {
        process::exit(1);
    }
}

impl Analyzer {
    fn new() -> Analyzer {
        Analyzer {
            errors: BTreeSet::new(),
            symbol_table: HashMap::new(),
        }
    }

    fn analyze_expression(&mut self, expr: &Expression) {
        match &expr.kind {
            ExpressionKind::Declare(ref dest, ref src) => {
                self.analyze_expression(src);

                match &dest.kind {
                    ExpressionKind::Identifier(id) => {
                        let type_to_add = self.get_type(src);
                        self.symbol_table.insert(id.to_string(), type_to_add);
                    }
                    _ => {
                        self.errors.insert(Error::new(
                            ErrorKind::InvalidIdentifier,
                            dest.line,
                            dest.col,
                        ));
                    }
                };
            }
            ExpressionKind::Assign(ref dest, ref src) => {
                self.analyze_expression(src);

                let type1 = self.get_type(src);
                let type2 = self.get_type(dest);

                if type1 != type2 {
                    self.errors.insert(Error::new(
                        ErrorKind::InvalidAssignment { type1, type2 },
                        src.line,
                        dest.col,
                    ));
                }
            }
            ExpressionKind::Add(ref dest, ref src)
            | ExpressionKind::Sub(ref dest, ref src)
            | ExpressionKind::Mul(ref dest, ref src)
            | ExpressionKind::Div(ref dest, ref src)
            | ExpressionKind::Mod(ref dest, ref src) => {
                self.analyze_expression(src);
                self.analyze_expression(dest);

                let type1 = self.get_type(src);
                let type2 = self.get_type(dest);

                if type1 != "int" || type2 != "int" {
                    self.errors.insert(Error::new(
                        ErrorKind::CannotMakeOperation { type1, type2 },
                        expr.line,
                        expr.col,
                    ));
                }
            }
            ExpressionKind::Sequence(seq) => {
                for expr in &seq[0..seq.len()] {
                    self.analyze_expression(expr);
                }
            }
            ExpressionKind::If(ref cond, ref seq) => {
                self.analyze_expression(cond);
                self.analyze_expression(seq);

                if self.get_type(cond) != "bool" {
                    self.errors
                        .insert(Error::new(ErrorKind::NotACondition, cond.line, cond.col));
                }
            }
            ExpressionKind::Eq(ref cmp1, ref cmp2)
            | ExpressionKind::Neq(ref cmp1, ref cmp2)
            | ExpressionKind::Lt(ref cmp1, ref cmp2)
            | ExpressionKind::Leq(ref cmp1, ref cmp2)
            | ExpressionKind::Gt(ref cmp1, ref cmp2)
            | ExpressionKind::Geq(ref cmp1, ref cmp2) => {
                self.analyze_expression(cmp1);
                self.analyze_expression(cmp2);

                let type1 = self.get_type(cmp1);
                let type2 = self.get_type(cmp2);

                if type1 != type2 {
                    self.errors.insert(Error::new(
                        ErrorKind::InvalidComparison { type1, type2 },
                        cmp1.line,
                        cmp1.col,
                    ));
                }
            }
            ExpressionKind::Loop(ref seq) => self.analyze_expression(seq),
            ExpressionKind::Exit(ref val) => {
                self.analyze_expression(val);

                if self.get_type(val) != "int" {
                    self.errors
                        .insert(Error::new(ErrorKind::InvalidExitCode, val.line, val.col));
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

    fn get_type(&mut self, expr: &Expression) -> String {
        match &expr.kind {
            ExpressionKind::Identifier(id) => {
                if self.symbol_table.contains_key(id) {
                    (*self.symbol_table[id]).to_string()
                } else {
                    self.errors.insert(Error::new(
                        ErrorKind::UndeclaredVariable {
                            var: id.to_string(),
                        },
                        expr.line,
                        expr.col,
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
            | ExpressionKind::Mod(ref dest, ..) => self.get_type(dest),
            _ => {
                dbg!(&expr);
                panic!("Not a valid type")
            }
        }
    }
}

fn print_error(error: &Error) {
    match &error.kind {
        ErrorKind::UndeclaredVariable { var } => {
            println!(
                "Variable with name {var} undeclared [line {}: col {}]",
                error.line, error.col
            )
        }
        ErrorKind::CannotMakeOperation { type1, type2 } => {
            println!(
                "Can't make an operation between ints, types {type1} and {type2} were given [line {}: col {}]",
                error.line, error.col
            )
        }
        ErrorKind::NotACondition => {
            println!(
                "Invalid condition for if statement [line {}: col {}]",
                error.line, error.col
            )
        }
        ErrorKind::InvalidComparison { type1, type2 } => {
            println!(
                "Can't compare type {type1} with type {type2} [line {}: col {}]",
                error.line, error.col
            )
        }
        ErrorKind::InvalidExitCode => {
            println!(
                "Exit codes can only be ints [line {}: col {}]",
                error.line, error.col
            )
        }
        ErrorKind::InvalidAssignment { type1, type2 } => {
            println!(
                "Can't assign variable with type {type1} to expression with type {type2} [line {}: col {}]",
                error.line, error.col
            )
        }
        ErrorKind::InvalidIdentifier => {
            println!(
                "Destination is not a valid identifier [line {}: col {}]",
                error.line, error.col
            )
        }
    }
}
