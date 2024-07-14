use std::collections::HashMap;

use crate::core::*;
use crate::errors::*;
use crate::parser::expression::*;
use crate::parser::op::*;

struct Analyzer<'a> {
    type_table: HashMap<String, PrimitiveType>,
    errors: &'a mut Errors,
}

pub fn analyze(ast: &mut Vec<Expression>, errors: &mut Errors) {
    let mut analyzer = Analyzer::new(errors);

    for expr in ast {
        analyzer.analyze(expr);
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

    fn analyze(&mut self, expr: &mut Expression) {
        match &mut expr.kind {
            ExpressionKind::Declare(ref mut dest, ref mut src) => {
                self.analyze_declare(dest, src);
                expr.r#type = PrimitiveType::Unit;
            }
            ExpressionKind::Assign(ref mut dest, ref mut src) => {
                self.analyze(src);

                match &dest.kind {
                    ExpressionKind::Identifier(id) => {
                        let src_type = self.get_type(src);
                        if self.type_table.contains_key(id) && self.type_table[id] != src_type {
                            self.errors.add(
                                ErrorKind::MismatchedTypes {
                                    expected: self.type_table[id].as_string(),
                                    found: src_type.as_string(),
                                },
                                src.line,
                                src.col,
                            )
                        } else {
                            self.type_table.insert(id.to_string(), src_type.clone());
                        }
                    }
                    ExpressionKind::Idx(id, ..) => {
                        expr.r#type = self.get_type(id);
                    }
                    _ => {
                        self.errors
                            .add(ErrorKind::InvalidIdentifier, dest.line, dest.col);
                    }
                };
                expr.r#type = dest.r#type.clone();
            }
            ExpressionKind::ArrayLit(contents) => {
                let array_type = self.get_type(&contents[0]);
                for expr in contents {
                    self.analyze(expr);
                    if self.get_type(expr) != array_type {
                        self.errors
                            .add(ErrorKind::InvalidArray, expr.line, expr.col);
                        break;
                    }
                    expr.r#type = array_type.clone();
                }
                expr.r#type = self.get_type(expr);
            }
            ExpressionKind::Op(ref mut arg1, _, ref mut arg2) => {
                self.analyze(arg1);
                self.analyze(arg2);

                self.check_type(expr);
            }
            ExpressionKind::Sequence(seq) => {
                for expr in seq {
                    self.analyze(expr);
                }
            }
            ExpressionKind::If(ref mut cond, ref mut seq) => {
                self.analyze(cond);
                self.analyze(seq);

                self.check_type(expr);
            }
            ExpressionKind::IfElse(ref mut cond, ref mut if_seq, ref mut else_seq) => {
                self.analyze(cond);
                self.analyze(if_seq);
                self.analyze(else_seq);

                self.check_type(expr);
            }
            ExpressionKind::Not(ref mut arg) => {
                self.analyze(arg);

                self.check_type(expr);
            }
            ExpressionKind::Loop(ref mut seq) => self.analyze(seq),
            ExpressionKind::While(ref mut cond, ref mut seq) => {
                self.analyze(cond);
                self.analyze(seq);

                self.check_type(expr);
            }
            ExpressionKind::Call(.., ref mut arg) => {
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
                            expected: type1.as_string(),
                            found: type2.as_string(),
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
                    Op::And | Op::Or => {
                        type1 != PrimitiveType::Bool || type2 != PrimitiveType::Bool
                    }
                    _ => type1 != PrimitiveType::I32 || type2 != PrimitiveType::I32,
                } && (!matches!(type1, PrimitiveType::Arr(..)) || type2 != PrimitiveType::I32)
                {
                    self.errors.add(
                        ErrorKind::OperationNotImplemented {
                            op: op.clone(),
                            type1: type1.as_string(),
                            type2: type2.as_string(),
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

                if type1 != PrimitiveType::Bool {
                    self.errors.add(
                        ErrorKind::MismatchedTypes {
                            expected: "bool".to_string(),
                            found: type1.as_string(),
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
                        if type1 != PrimitiveType::I32 {
                            self.errors.add(
                                ErrorKind::MismatchedTypes {
                                    expected: "i8".to_string(),
                                    found: type1.as_string(),
                                },
                                arg.line,
                                arg.col,
                            );
                        }
                    }
                    "print" => {
                        if !matches!(type1, PrimitiveType::Arr(..)) {
                            self.errors.add(
                                ErrorKind::MismatchedTypes {
                                    expected: "string".to_string(),
                                    found: type1.as_string(),
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

    fn analyze_declare(&mut self, dest: &mut Box<Expression>, src: &mut Box<Expression>) {
        self.analyze(src);

        match &dest.kind {
            ExpressionKind::Identifier(id) => {
                if dest.r#type != PrimitiveType::Void {
                    let src_type = self.get_type(&src);
                    if !is_valid_type(dest.r#type.as_string()) {
                        self.errors.add(
                            ErrorKind::UndefinedType {
                                type1: dest.r#type.as_string(),
                            },
                            src.line,
                            src.col,
                        )
                    } else if dest.r#type != src_type && src.r#type != PrimitiveType::Void {
                        self.errors.add(
                            ErrorKind::MismatchedTypes {
                                expected: dest.r#type.as_string(),
                                found: src_type.as_string(),
                            },
                            src.line,
                            src.col,
                        )
                    } else {
                        src.r#type = dest.r#type.clone();
                    }
                } else {
                    self.get_type(src);
                    dest.r#type = src.r#type.clone();
                }
                self.type_table.insert(id.to_string(), dest.r#type.clone());
            }
            _ => {
                self.errors
                    .add(ErrorKind::InvalidIdentifier, dest.line, dest.col);
            }
        };
    }

    fn get_type(&mut self, expr: &Expression) -> PrimitiveType {
        match &expr.kind {
            ExpressionKind::Identifier(id) => {
                if self.type_table.contains_key(id) {
                    self.type_table[id].clone()
                } else {
                    self.errors.add(
                        ErrorKind::UndeclaredVariable {
                            var: id.to_string(),
                        },
                        expr.line,
                        expr.col,
                    );
                    PrimitiveType::Void
                }
            }
            ExpressionKind::IntLit(..) | ExpressionKind::Idx(..) => PrimitiveType::I32,
            ExpressionKind::BoolLit(..) | ExpressionKind::Not(..) => PrimitiveType::Bool,
            ExpressionKind::StringLit(..) => PrimitiveType::Arr(Box::new(PrimitiveType::U8)),
            ExpressionKind::ArrayLit(expr) => PrimitiveType::Arr(Box::new(self.get_type(&expr[0]))),
            ExpressionKind::Op(_, op, _) => op.get_type(),
            _ => {
                dbg!(&expr);
                panic!("Not a valid type")
            }
        }
    }
}
