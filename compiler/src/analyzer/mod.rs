use std::collections::HashMap;

use crate::core::*;
use crate::errors::*;
use crate::parser::expression::*;
use crate::statement::*;

struct Analyzer<'a> {
    type_table: HashMap<String, PrimitiveType>,
    errors: &'a mut Errors,
}

pub fn analyze(ast: &mut Vec<Statement>, errors: &mut Errors) {
    let mut analyzer = Analyzer::new(errors);

    for statement in ast {
        analyzer.analyze_statement(statement);
    }

    if analyzer.errors.should_abort() {
        analyzer.errors.print_and_abort();
    }
}

impl Analyzer<'_> {
    fn new(errors: &mut Errors) -> Analyzer {
        Analyzer {
            type_table: HashMap::new(),
            errors,
        }
    }

    fn analyze_statement(&mut self, statement: &mut Statement) {
        match &mut statement.kind {
            StatementKind::Declare { .. } => {
                self.analyze_declare(statement);
            }
            StatementKind::Assign { dest, src } => {
                self.analyze_expression(dest);
                self.analyze_expression(src);

                if dest.r#type != src.r#type {
                    self.errors.add(
                        ErrorKind::MismatchedTypes {
                            expected: dest.r#type.as_string(),
                            found: src.r#type.as_string(),
                        },
                        src.line,
                        dest.col,
                    );
                }
            }
            StatementKind::If { cond, block } => {
                self.analyze_condition(cond);

                for statement in block {
                    self.analyze_statement(statement);
                }
            }
            StatementKind::IfElse {
                cond,
                true_block,
                false_block,
            } => {
                self.analyze_condition(cond);

                for statement in true_block {
                    self.analyze_statement(statement);
                }

                for statement in false_block {
                    self.analyze_statement(statement);
                }
            }
            StatementKind::Loop { block } => {
                for statement in block {
                    self.analyze_statement(statement);
                }
            }
            StatementKind::While { cond, block } => {
                self.analyze_condition(cond);

                for statement in block {
                    self.analyze_statement(statement);
                }
            }
            StatementKind::Call { name, args } => {
                for arg in args {
                    self.analyze_expression(arg);
                }
            }
            _ => {}
        }
    }

    fn analyze_condition(&mut self, cond: &Expression) {
        self.analyze_expression(cond);
        if cond.r#type != PrimitiveType::Bool {
            self.errors.add(
                ErrorKind::MismatchedTypes {
                    expected: "bool".to_string(),
                    found: cond.r#type.as_string(),
                },
                cond.line,
                cond.col,
            );
        }
    }

    fn analyze_expression(&mut self, expr: &Expression) -> PrimitiveType {
        match &expr.kind {
            ExpressionKind::Call(name, args) => {
                for arg in args {
                    let expected_type = self.type_table[name].clone();
                    self.analyze_expression(&arg);
                }
            }
            ExpressionKind::Lit(..) => expr.r#type,
            ExpressionKind::Id(id) => if self.type_table.contains_key(id) {},
            ExpressionKind::Array(values) => {
                for value in values {
                    self.analyze_expression(&value)
                }
            }
        }
    }

    fn analyze_declare(&mut self, statement: &Statement) {
        if let StatementKind::Declare {
            name,
            type_hint,
            value,
        } = &statement.kind
        {
            if self.type_table.contains_key(name) {
                self.errors.add(
                    ErrorKind::AlreadyDeclared {
                        var: name.to_string(),
                    },
                    statement.line,
                    statement.col,
                )
            }

            if let Some(type_hint) = &type_hint {
                self.type_table.insert(name.to_string(), type_hint.clone());
            }

            if let Some(value) = &value {
                self.analyze_expression(value);
                if let Some(type_hint) = &type_hint {
                    if *type_hint != value.r#type {
                        self.errors.add(
                            ErrorKind::MismatchedTypes {
                                expected: type_hint.to_string(),
                                found: value.r#type.to_string(),
                            },
                            statement.line,
                            statement.col,
                        )
                    }
                } else {
                    self.type_table
                        .insert(name.to_string(), value.r#type.clone());
                }
            }
        } else {
            panic!("Must recieve a declare statement");
        }
    }
}
