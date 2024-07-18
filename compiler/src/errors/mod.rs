use crate::parser::op::*;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::process;

pub struct Errors {
    errors: BTreeSet<Error>,
}

impl Errors {
    pub fn new() -> Errors {
        Errors {
            errors: BTreeSet::new(),
        }
    }

    pub fn add(&mut self, kind: ErrorKind, line: usize, col: usize) {
        self.errors.insert(Error { kind, line, col });
    }

    pub fn should_abort(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn print_and_abort(&self) {
        for err in &self.errors {
            Errors::print_error(err);
        }
        process::exit(1);
    }

    fn print_error(error: &Error) {
        match &error.kind {
            ErrorKind::UndeclaredVariable { var } => {
                println!(
                    "{}:{}: use of undeclared variable '{var}'",
                    error.line, error.col
                )
            }
            ErrorKind::AlreadyDeclared { var } => {
                println!("{}:{}: '{var}' previously declared", error.line, error.col)
            }
            ErrorKind::OperationNotImplemented { op, type1, type2 } => {
                println!(
                    "{}:{}: operation '{}' not implemented between types '{type1}' and '{type2}'",
                    error.line, error.col, op
                )
            }
            ErrorKind::MismatchedTypes { expected, found } => {
                println!(
                    "{}:{}: mismatched types\n\texpected '{}', found '{}'",
                    error.line, error.col, expected, found
                )
            }
            ErrorKind::InvalidArray => println!(
                "{}:{}: All array elements must have the same type",
                error.line, error.col
            ),
            ErrorKind::InvalidIdentifier => {
                println!("{}:{}: invalid identifier", error.line, error.col)
            }
            ErrorKind::UndefinedType { type1 } => {
                println!("{}:{}: undefined type '{}'", error.line, error.col, type1);
            }
            ErrorKind::UndefinedProcedure => {
                println!("{}:{}: undefined procedure", error.line, error.col);
            }
        }
    }
}

#[derive(PartialEq, Eq)]
struct Error {
    kind: ErrorKind,
    line: usize,
    col: usize,
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

#[derive(PartialEq, Eq)]
pub enum ErrorKind {
    UndeclaredVariable {
        var: String,
    },
    AlreadyDeclared {
        var: String,
    },
    OperationNotImplemented {
        op: Op,
        type1: String,
        type2: String,
    },
    MismatchedTypes {
        expected: String,
        found: String,
    },
    InvalidArray,
    InvalidIdentifier,
    UndefinedType {
        type1: String,
    },
    UndefinedProcedure,
}
