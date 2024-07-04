use std::collections::HashMap;

use crate::errors::*;
use crate::parser::*;

struct Analyzer<'a> {
    symbol_table: HashMap<String, String>,
    errors: &'a mut Errors,
}

pub fn analyze(ast: &mut Vec<Expression>, errors: &mut Errors) -> HashMap<String, String> {
    let mut analyzer = Analyzer::new(errors);

    for expr in ast {
        analyzer.check(expr);
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

    fn check(&mut self, expr: &Expression) {
        match &expr.kind {
            ExpressionKind::Declare(ref dest, ref src)
            | ExpressionKind::Assign(ref dest, ref src) => {
                self.check(src);

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
            ExpressionKind::Op(ref arg1, _, ref arg2) => {
                self.check(arg1);
                self.check(arg2);

                self.check_type(expr);
            }
            ExpressionKind::Sequence(seq) => {
                for expr in &seq[0..seq.len()] {
                    self.check(expr);
                }
            }
            ExpressionKind::If(ref cond, ref seq) => {
                self.check(cond);
                self.check(seq);

                self.check_type(expr);
            }
            ExpressionKind::IfElse(ref cond, ref if_seq, ref else_seq) => {
                self.check(cond);
                self.check(if_seq);
                self.check(else_seq);

                self.check_type(expr);
            }
            ExpressionKind::Not(ref arg) => {
                self.check(arg);

                self.check_type(expr);
            }
            ExpressionKind::Loop(ref seq) => self.check(seq),
            ExpressionKind::Exit(ref val) => {
                self.check(val);

                self.check_type(expr);
            }
            _ => {}
        }
    }

    fn check_type(&mut self, expr: &Expression) {
        match &expr.kind {
            ExpressionKind::Assign(ref dest, ref src) => {
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
            ExpressionKind::If(ref arg, ..)
            | ExpressionKind::IfElse(ref arg, ..)
            | ExpressionKind::Not(ref arg) => {
                let type1 = self.get_type(arg);

                if type1 != "bool" {
                    self.errors.add(
                        ErrorKind::MismatchedTypes {
                            expected: "bool".to_string(),
                            found: type1,
                        },
                        arg.line,
                        arg.col,
                    );
                }
            }
            ExpressionKind::Exit(ref val) => {
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
            _ => {}
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
