use crate::parser::*;
use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap},
    process,
};

#[derive(PartialEq, Eq)]
enum ErrorKind {
    UndeclaredVariable {
        var: String,
    },
    OperationNotImplemented {
        op: Op,
        type1: String,
        type2: String,
    },
    NotACondition,
    InvalidAssignment {
        type1: String,
        type2: String,
    },
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

pub fn analyze(ast: &Vec<Expression>) -> HashMap<String, String> {
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

    analyzer.symbol_table
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
            ExpressionKind::Op(ref arg1, op, ref arg2) => {
                self.analyze_expression(arg1);
                self.analyze_expression(arg2);

                let type1 = self.get_type(arg1);
                let type2 = self.get_type(arg2);
                if match op {
                    Op::And | Op::Or => type1 != "bool" || type2 != "bool",
                    _ => type1 != "int" || type2 != "int",
                } {
                    self.errors.insert(Error::new(
                        ErrorKind::OperationNotImplemented {
                            op: op.clone(),
                            type1,
                            type2,
                        },
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
            ExpressionKind::IfElse(ref cond, ref if_seq, ref else_seq) => {
                self.analyze_expression(cond);
                self.analyze_expression(if_seq);
                self.analyze_expression(else_seq);

                if self.get_type(cond) != "bool" {
                    self.errors
                        .insert(Error::new(ErrorKind::NotACondition, cond.line, cond.col));
                }
            }
            ExpressionKind::Not(ref arg) => {
                self.analyze_expression(arg);

                let type1 = self.get_type(arg);

                if type1 != "bool" {
                    self.errors.insert(Error::new(
                        // Todo: add proper error type
                        ErrorKind::NotACondition,
                        expr.line,
                        expr.col,
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
            ExpressionKind::Continue
            | ExpressionKind::Break
            | ExpressionKind::Identifier(..)
            | ExpressionKind::IntLit(..)
            | ExpressionKind::BoolLit(..) => {}
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
            ExpressionKind::BoolLit(..) => "bool".to_string(),
            ExpressionKind::Op(_, op, _) => match op {
                Op::And | Op::Or | Op::Eq | Op::Neq | Op::Lt | Op::Leq | Op::Gt | Op::Geq => {
                    "bool".to_string()
                }
                Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Mod => "int".to_string(),
            },
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
        ErrorKind::OperationNotImplemented { op, type1, type2 } => {
            println!(
                "Operation '{}' not implemented for types {type1} and {type2} [line {}: col {}]",
                op, error.line, error.col
            )
        }
        ErrorKind::NotACondition => {
            println!(
                "Invalid condition for if statement [line {}: col {}]",
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
