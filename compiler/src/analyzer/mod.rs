use std::collections::HashMap;

use crate::core::*;
use crate::errors::*;
use crate::parser::expression::*;
use crate::parser::op::*;

struct Analyzer<'a> {
    type_table: HashMap<String, PrimitiveType>,
    errors: &'a mut Errors,
}

pub fn analyze(ast: &mut Vec<Expression>, errors: &mut Errors) -> HashMap<String, PrimitiveType> {
    let mut analyzer = Analyzer::new(errors);

    for expr in ast {
        analyzer.analyze(expr);
    }

    if analyzer.errors.should_abort() {
        analyzer.errors.print_and_abort();
    }

    analyzer.type_table
}

impl Analyzer<'_> {
    fn new(errors: &mut Errors) -> Analyzer {
        Analyzer {
            type_table: HashMap::new(),
            errors,
        }
    }

    fn analyze(&mut self, expr: &Expression) {
        match &expr.kind {
            ExpressionKind::Declare(ref dest, ref src, type_hint) => {
                self.analyze(src);

                match &dest.kind {
                    ExpressionKind::Identifier(id) => {
                        let type_to_add = self.get_type(src);
                        if let Some(type_hint) = type_hint {
                            if !is_valid_type(type_hint.to_string()) {
                                self.errors.add(
                                    ErrorKind::UndefinedType {
                                        type1: type_hint.clone(),
                                    },
                                    src.line,
                                    src.col,
                                )
                            } else if *type_hint != type_to_add {
                                self.errors.add(
                                    ErrorKind::MismatchedTypes {
                                        expected: type_hint.clone(),
                                        found: type_to_add.clone(),
                                    },
                                    src.line,
                                    src.col,
                                )
                            } else {
                                self.type_table
                                    .insert(id.to_string(), get_type(type_hint.to_string()));
                            }
                        } else {
                            self.type_table
                                .insert(id.to_string(), get_type(type_to_add));
                        }
                    }
                    _ => {
                        self.errors
                            .add(ErrorKind::InvalidIdentifier, dest.line, dest.col);
                    }
                };
            }
            ExpressionKind::Assign(ref dest, ref src) => {
                self.analyze(src);

                match &dest.kind {
                    ExpressionKind::Identifier(id) => {
                        let type_to_add = self.get_type(src);
                        if self.type_table.contains_key(id)
                            && self.type_table[id].as_string() != type_to_add
                        {
                            self.errors.add(
                                ErrorKind::MismatchedTypes {
                                    expected: self.type_table[id].as_string(),
                                    found: type_to_add,
                                },
                                src.line,
                                src.col,
                            )
                        } else {
                            self.type_table
                                .insert(id.to_string(), get_type(type_to_add));
                        }
                    }
                    ExpressionKind::Idx(id, offset) => {
                        self.type_table.insert(
                            format!("[{} + {}]", id.to_str(), offset.to_str()),
                            get_type("ptr".to_string()),
                        );
                    }
                    _ => {
                        self.errors
                            .add(ErrorKind::InvalidIdentifier, dest.line, dest.col);
                    }
                };
            }
            ExpressionKind::ArrayLit(contents) => {
                for expr in contents {
                    self.analyze(expr);
                }
            }
            ExpressionKind::Op(ref arg1, _, ref arg2) => {
                self.analyze(arg1);
                self.analyze(arg2);

                self.check_type(expr);
            }
            ExpressionKind::Sequence(seq) => {
                for expr in seq {
                    self.analyze(expr);
                }
            }
            ExpressionKind::If(ref cond, ref seq) => {
                self.analyze(cond);
                self.analyze(seq);

                self.check_type(expr);
            }
            ExpressionKind::IfElse(ref cond, ref if_seq, ref else_seq) => {
                self.analyze(cond);
                self.analyze(if_seq);
                self.analyze(else_seq);

                self.check_type(expr);
            }
            ExpressionKind::Not(ref arg) => {
                self.analyze(arg);

                self.check_type(expr);
            }
            ExpressionKind::Loop(ref seq) => self.analyze(seq),
            ExpressionKind::While(ref cond, ref seq) => {
                self.analyze(cond);
                self.analyze(seq);

                self.check_type(expr);
            }
            ExpressionKind::Call(.., ref arg) => {
                self.analyze(arg);

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
                    _ => type1 != "i8" || type2 != "i8",
                } && (type1 != "ptr" || type2 != "i8")
                {
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
            | ExpressionKind::While(ref arg, ..)
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
            ExpressionKind::Call(name, ref arg) => {
                let type1 = self.get_type(arg);

                match name.as_str() {
                    "exit" => {
                        if type1 != "i8" {
                            self.errors.add(
                                ErrorKind::MismatchedTypes {
                                    expected: "i8".to_string(),
                                    found: type1,
                                },
                                arg.line,
                                arg.col,
                            );
                        }
                    }
                    "print" => {
                        if type1 != "string" && type1 != "ptr" {
                            self.errors.add(
                                ErrorKind::MismatchedTypes {
                                    expected: "string".to_string(),
                                    found: type1,
                                },
                                arg.line,
                                arg.col,
                            );
                        }
                    }
                    _ => panic!("Unrecognized procedure name"),
                }
            }
            _ => {}
        }
    }

    fn get_type(&mut self, expr: &Expression) -> String {
        match &expr.kind {
            ExpressionKind::Identifier(id) => {
                if self.type_table.contains_key(id) {
                    return self.type_table[id].as_string();
                } else {
                    self.errors.add(
                        ErrorKind::UndeclaredVariable {
                            var: id.to_string(),
                        },
                        expr.line,
                        expr.col,
                    );
                    "null"
                }
            }
            ExpressionKind::IntLit(..) | ExpressionKind::Idx(..) => "i8",
            ExpressionKind::BoolLit(..) | ExpressionKind::Not(..) => "bool",
            ExpressionKind::StringLit(..) | ExpressionKind::ArrayLit(..) => "ptr",
            ExpressionKind::Op(_, op, _) => op.get_type(),
            _ => {
                dbg!(&expr);
                panic!("Not a valid type")
            }
        }
        .to_string()
    }
}
