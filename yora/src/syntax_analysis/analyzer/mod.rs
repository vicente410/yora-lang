use std::collections::HashMap;

use super::errors::*;
use super::parser::expression::*;
use super::parser::statement::*;
use crate::core::*;

struct Analyzer<'a> {
    type_table: HashMap<String, PrimitiveType>,
    call_signatures: HashMap<(String, Vec<PrimitiveType>), Option<PrimitiveType>>,
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
        let call_signatures = HashMap::from([
            (
                ("+".to_string(), vec![PrimitiveType::Int; 2]),
                Some(PrimitiveType::Int),
            ),
            (
                ("-".to_string(), vec![PrimitiveType::Int; 2]),
                Some(PrimitiveType::Int),
            ),
            (
                ("/".to_string(), vec![PrimitiveType::Int; 2]),
                Some(PrimitiveType::Int),
            ),
            (
                ("*".to_string(), vec![PrimitiveType::Int; 2]),
                Some(PrimitiveType::Int),
            ),
            (
                ("%".to_string(), vec![PrimitiveType::Int; 2]),
                Some(PrimitiveType::Int),
            ),
            (
                ("==".to_string(), vec![PrimitiveType::Int; 2]),
                Some(PrimitiveType::Bool),
            ),
            (
                ("!=".to_string(), vec![PrimitiveType::Int; 2]),
                Some(PrimitiveType::Bool),
            ),
            (
                ("<".to_string(), vec![PrimitiveType::Int; 2]),
                Some(PrimitiveType::Bool),
            ),
            (
                ("<=".to_string(), vec![PrimitiveType::Int; 2]),
                Some(PrimitiveType::Bool),
            ),
            (
                (">".to_string(), vec![PrimitiveType::Int; 2]),
                Some(PrimitiveType::Bool),
            ),
            (
                (">=".to_string(), vec![PrimitiveType::Int; 2]),
                Some(PrimitiveType::Bool),
            ),
            (
                ("and".to_string(), vec![PrimitiveType::Bool; 2]),
                Some(PrimitiveType::Bool),
            ),
            (
                ("or".to_string(), vec![PrimitiveType::Bool; 2]),
                Some(PrimitiveType::Bool),
            ),
            (
                ("!".to_string(), vec![PrimitiveType::Bool]),
                Some(PrimitiveType::Bool),
            ),
            (
                (
                    "[]".to_string(),
                    vec![
                        PrimitiveType::Arr(Box::new(PrimitiveType::Int)),
                        PrimitiveType::Int,
                    ],
                ),
                Some(PrimitiveType::Int),
            ),
            (
                (
                    "[]".to_string(),
                    vec![
                        PrimitiveType::Arr(Box::new(PrimitiveType::Char)),
                        PrimitiveType::Int,
                    ],
                ),
                Some(PrimitiveType::Char),
            ),
            (
                ("input".to_string(), vec![]),
                Some(PrimitiveType::Arr(Box::new(PrimitiveType::Char))),
            ),
            (
                (
                    "parse".to_string(),
                    vec![PrimitiveType::Arr(Box::new(PrimitiveType::Char))],
                ),
                Some(PrimitiveType::Int),
            ),
        ]);
        Analyzer {
            type_table: HashMap::new(),
            call_signatures,
            errors,
        }
    }

    fn analyze_statement(&mut self, statement: &mut Statement) {
        match &mut statement.kind {
            StatementKind::Declare {
                name,
                type_hint,
                value,
            } => {
                if self.type_table.contains_key(name) {
                    self.errors.add(
                        ErrorKind::AlreadyDeclared {
                            var: name.to_string(),
                        },
                        statement.line,
                        statement.col,
                    )
                }

                if let Some(value) = value {
                    self.analyze_expression(value);

                    if let Some(value_type) = &value.r#type {
                        if let Some(type_hint) = type_hint {
                            if *type_hint != *value_type {
                                self.errors.add(
                                    ErrorKind::MismatchedTypes {
                                        expected: value_type.to_string(),
                                        found: type_hint.to_string(),
                                    },
                                    statement.line,
                                    statement.col,
                                )
                            } else {
                                self.type_table.insert(name.to_string(), value_type.clone());
                            }
                        } else {
                            self.type_table.insert(name.to_string(), value_type.clone());
                        }
                    } else {
                        todo!("Infere a type in the expression");
                    }
                } else if let Some(type_hint) = type_hint {
                    self.type_table.insert(name.to_string(), type_hint.clone());
                } else {
                    todo!("Insert type hint on uninitialized variables");
                }
            }
            StatementKind::Assign { dest, src } => {
                self.analyze_expression(dest);
                self.analyze_expression(src);

                let dest_type = dest.r#type.clone().unwrap();
                let src_type = src.r#type.clone().unwrap();

                if dest_type != src_type {
                    self.errors.add(
                        ErrorKind::MismatchedTypes {
                            expected: dest_type.to_string(),
                            found: src_type.to_string(),
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
            StatementKind::Call { name: _, args } => {
                for arg in args {
                    self.analyze_expression(arg);
                }
            }
            StatementKind::Procedure {
                name,
                args,
                ret,
                block: _,
            } => {
                let mut args_types: Vec<PrimitiveType> = Vec::with_capacity(args.len());
                for arg in args {
                    if let Some(arg_type) = &arg.1 {
                        args_types.push(arg_type.clone());
                    } else {
                        panic!("Insert type hint in procedure declaration");
                    }
                }
                self.call_signatures
                    .insert((name.to_string(), args_types), ret.clone());
            }
            _ => {}
        }
    }

    fn analyze_condition(&mut self, cond: &mut Expression) {
        self.analyze_expression(cond);

        let cond_type = cond.r#type.clone().unwrap();

        if cond_type != PrimitiveType::Bool {
            self.errors.add(
                ErrorKind::MismatchedTypes {
                    expected: "bool".to_string(),
                    found: cond_type.to_string(),
                },
                cond.line,
                cond.col,
            );
        }
    }

    fn analyze_expression(&mut self, expr: &mut Expression) {
        match &mut expr.kind {
            ExpressionKind::Call(name, args) => {
                let mut args_types: Vec<PrimitiveType> = Vec::new();
                for arg in args {
                    self.analyze_expression(arg);
                    if let ExpressionKind::Id(id) = &arg.kind {
                        args_types.push(self.type_table[&id.clone()].clone());
                    } else if let Some(arg_type) = &arg.r#type {
                        args_types.push(arg_type.clone());
                    }
                }
                if self
                    .call_signatures
                    .contains_key(&(name.clone(), args_types.clone()))
                {
                    expr.r#type = self.call_signatures[&(name.clone(), args_types)].clone();
                } else {
                    self.errors.add(
                        ErrorKind::UndefinedProcedure { name: name.clone() },
                        expr.line,
                        expr.col,
                    )
                }
            }
            ExpressionKind::Lit(..) => {}
            ExpressionKind::Id(id) => {
                if self.type_table.contains_key(&id.clone()) {
                    expr.r#type = Some(self.type_table[&id.clone()].clone());
                }
            }
            ExpressionKind::Array(values) => {
                for value in values {
                    self.analyze_expression(value)
                }
            }
        }
    }
}
