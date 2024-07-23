use std::collections::HashMap;

use super::errors::*;
use super::parser::expression::*;
use super::parser::statement::*;
use crate::core::*;

struct Analyzer<'a> {
    variables: Vec<(String, PrimitiveType)>,
    num_vars_scope: Vec<i8>,
    call_signatures: HashMap<(String, Vec<PrimitiveType>), Option<PrimitiveType>>,
    errors: &'a mut Errors,
}

pub fn analyze(ast: &mut Vec<Statement>, errors: &mut Errors) {
    let mut analyzer = Analyzer::new(errors);

    analyzer.start_scope();
    for statement in ast {
        analyzer.analyze_statement(statement);
    }
    analyzer.end_scope();

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
                ("-".to_string(), vec![PrimitiveType::Int; 1]),
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
                    "string_to_int".to_string(),
                    vec![PrimitiveType::Arr(Box::new(PrimitiveType::Char))],
                ),
                Some(PrimitiveType::Int),
            ),
            (
                ("char_to_int".to_string(), vec![PrimitiveType::Char]),
                Some(PrimitiveType::Int),
            ),
            (
                ("int_to_char".to_string(), vec![PrimitiveType::Int]),
                Some(PrimitiveType::Char),
            ),
            (
                (
                    "string_len".to_string(),
                    vec![PrimitiveType::Arr(Box::new(PrimitiveType::Char))],
                ),
                Some(PrimitiveType::Int),
            ),
        ]);
        Analyzer {
            variables: Vec::new(),
            num_vars_scope: Vec::new(),
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
                                self.add_variable(name.clone(), value_type.clone());
                            }
                        } else {
                            self.add_variable(name.clone(), value_type.clone());
                        }
                    } else {
                        todo!("Infere a type in the expression");
                    }
                } else if let Some(type_hint) = type_hint {
                    self.add_variable(name.clone(), type_hint.clone());
                } else {
                    todo!("Insert type hint on uninitialized variables");
                }
            }
            StatementKind::Assign { dest, src } => {
                self.analyze_expression(dest);
                self.analyze_expression(src);

                let src_type = src.r#type.clone().unwrap();
                if let ExpressionKind::Id(id) = &dest.kind {
                    if self.contains_variable(id.clone()) {
                        let dest_type = dest.r#type.clone().unwrap();
                        if dest_type != src_type {
                            self.errors.add(
                                ErrorKind::MismatchedTypes {
                                    expected: dest_type.to_string(),
                                    found: src_type.to_string(),
                                },
                                dest.line,
                                dest.col,
                            );
                        }
                    } else {
                        self.errors.add(
                            ErrorKind::UndeclaredVariable { var: id.clone() },
                            dest.line,
                            dest.col,
                        );
                    }
                } else if let ExpressionKind::Call(name, ref mut exprs) = &mut dest.kind {
                    if name == "[]" {
                        for expr in exprs {
                            self.analyze_expression(expr);
                        }
                    }
                } else {
                    dbg!(&dest);
                    self.errors
                        .add(ErrorKind::InvalidIdentifier, dest.line, dest.col);
                }
            }
            StatementKind::If { cond, block } => {
                self.start_scope();
                self.analyze_condition(cond);

                for statement in block {
                    self.analyze_statement(statement);
                }

                self.end_scope();
            }
            StatementKind::IfElse {
                cond,
                true_block,
                false_block,
            } => {
                self.analyze_condition(cond);

                self.start_scope();
                for statement in true_block {
                    self.analyze_statement(statement);
                }

                self.end_scope();

                self.start_scope();
                for statement in false_block {
                    self.analyze_statement(statement);
                }
                self.end_scope();
            }
            StatementKind::Loop { block } => {
                self.start_scope();
                for statement in block {
                    self.analyze_statement(statement);
                }
                self.end_scope();
            }
            StatementKind::While { cond, block } => {
                self.start_scope();
                self.analyze_condition(cond);

                for statement in block {
                    self.analyze_statement(statement);
                }
                self.end_scope();
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
                block,
            } => {
                //todo change this approach
                let variables_copy = self.variables.clone();
                self.variables.clear();

                self.start_scope();

                let mut args_types: Vec<PrimitiveType> = Vec::with_capacity(args.len());
                for arg in args {
                    if let Some(arg_type) = &arg.1 {
                        args_types.push(arg_type.clone());
                        self.add_variable(arg.0.clone(), arg_type.clone());
                    } else {
                        panic!("Insert type hint in procedure declaration");
                    }
                }

                for statement in block {
                    self.analyze_statement(statement);
                }

                self.call_signatures
                    .insert((name.to_string(), args_types), ret.clone());

                self.end_scope();

                self.variables = variables_copy;
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
                        args_types.push(self.get_type_by_name(id));
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
                    );
                }
            }
            ExpressionKind::Lit(..) => {}
            ExpressionKind::Id(id) => {
                if self.contains_variable(id.clone()) {
                    expr.r#type = Some(self.get_type_by_name(id));
                } else {
                    self.errors.add(
                        ErrorKind::UndeclaredVariable { var: id.clone() },
                        expr.line,
                        expr.col,
                    );
                }
            }
            ExpressionKind::Array(values) => {
                for value in values {
                    self.analyze_expression(value)
                }
            }
        }
    }

    fn add_variable(&mut self, name: String, r#type: PrimitiveType) {
        let len = self.num_vars_scope.len();
        self.num_vars_scope[len - 1] += 1;
        self.variables.push((name, r#type));
    }

    fn contains_variable(&self, name: String) -> bool {
        let vars = self.variables.iter().rev();
        for var in vars {
            if var.0 == *name {
                return true;
            }
        }
        false
    }

    fn get_type_by_name(&self, name: &String) -> PrimitiveType {
        let vars = self.variables.iter().rev();
        for var in vars {
            if var.0 == *name {
                return var.1.clone();
            }
        }
        panic!("Undeclared variable '{name}' used");
    }

    fn start_scope(&mut self) {
        self.num_vars_scope.push(0);
    }

    fn end_scope(&mut self) {
        for _ in 0..self.num_vars_scope.pop().unwrap() {
            self.variables.pop();
        }
    }
}
