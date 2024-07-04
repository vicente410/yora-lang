use std::collections::HashMap;

use crate::errors::*;
use crate::parser::*;

struct Analyzer<'a> {
    symbol_table: HashMap<String, String>,
    errors: &'a mut Errors,
}

pub fn analyze(ast: &Vec<Expression>, errors: &mut Errors) -> HashMap<String, String> {
    let mut analyzer = Analyzer::new(errors);

    for expr in ast {
        analyzer.analyze_expression(expr);
    }

    if analyzer.errors.should_abort() {
        analyzer.errors.print_and_abort();
    }

    analyzer.symbol_table
}

impl Analyzer<'_> {
    fn new(errors: &mut Errors) -> Analyzer {
        Analyzer {
            symbol_table: HashMap::new(),
            errors,
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
                        self.errors
                            .add(ErrorKind::InvalidIdentifier, dest.line, dest.col);
                    }
                };
            }
            ExpressionKind::Assign(ref dest, ref src) => {
                self.analyze_expression(src);

                let type1 = self.get_type(src);
                let type2 = self.get_type(dest);

                if type1 != type2 {
                    self.errors.add(
                        ErrorKind::MismatchedTypes {
                            expected: type1,
                            found: type2,
                        },
                        src.line,
                        dest.col,
                    );
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
                    self.errors.add(
                        ErrorKind::OperationNotImplemented {
                            op: op.clone(),
                            type1,
                            type2,
                        },
                        expr.line,
                        expr.col,
                    );
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

                let type1 = self.get_type(cond);

                if type1 != "bool" {
                    self.errors.add(
                        ErrorKind::MismatchedTypes {
                            expected: "bool".to_string(),
                            found: type1,
                        },
                        cond.line,
                        cond.col,
                    );
                }
            }
            ExpressionKind::IfElse(ref cond, ref if_seq, ref else_seq) => {
                self.analyze_expression(cond);
                self.analyze_expression(if_seq);
                self.analyze_expression(else_seq);

                let type1 = self.get_type(cond);

                if type1 != "bool" {
                    self.errors.add(
                        ErrorKind::MismatchedTypes {
                            expected: "bool".to_string(),
                            found: type1,
                        },
                        cond.line,
                        cond.col,
                    );
                }
            }
            ExpressionKind::Not(ref arg) => {
                self.analyze_expression(arg);

                let type1 = self.get_type(arg);

                if type1 != "bool" {
                    self.errors.add(
                        // Todo: add proper error type
                        ErrorKind::MismatchedTypes {
                            expected: "bool".to_string(),
                            found: type1,
                        },
                        expr.line,
                        expr.col,
                    );
                }
            }
            ExpressionKind::Loop(ref seq) => self.analyze_expression(seq),
            ExpressionKind::Exit(ref val) => {
                self.analyze_expression(val);

                let type1 = self.get_type(val);

                if type1 != "int" {
                    self.errors.add(
                        ErrorKind::MismatchedTypes {
                            expected: "int".to_string(),
                            found: type1,
                        },
                        val.line,
                        val.col,
                    );
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
                    self.errors.add(
                        ErrorKind::UndeclaredVariable {
                            var: id.to_string(),
                        },
                        expr.line,
                        expr.col,
                    );
                    "null".to_string()
                }
            }
            ExpressionKind::IntLit(..) => "int".to_string(),
            ExpressionKind::BoolLit(..) | ExpressionKind::Not(..) => "bool".to_string(),
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
