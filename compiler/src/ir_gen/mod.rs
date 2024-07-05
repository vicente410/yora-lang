use std::collections::HashMap;

use crate::parser::*;

pub mod ir_pretty;

#[derive(Debug, PartialEq, Clone)]
pub enum Ir {
    // Operations
    Ass {
        dest: String,
        src: String,
    },
    Not {
        dest: String,
        src: String,
    },
    Op {
        dest: String,
        src1: String,
        op: Op,
        src2: String,
    },

    // Control-flow
    Label(String),
    Goto {
        label: String,
    },
    IfGoto {
        src1: String,
        src2: String,
        cond: Op,
        label: String,
    },

    // Funcion calls
    Param {
        src: String,
    },
    Call {
        label: String,
    },
    Ret {
        src: String,
    },
}

struct Nums {
    tmp: u32,
    ifs: u32,
    loops: u32,
}

pub struct IrGenerator<'a> {
    type_table: &'a mut HashMap<String, String>,
    nums: Nums,
    inter_repr: Vec<Ir>,
}

impl IrGenerator<'_> {
    fn new(type_table: &mut HashMap<String, String>) -> IrGenerator {
        IrGenerator {
            type_table,
            nums: Nums {
                tmp: 0,
                ifs: 0,
                loops: 0,
            },
            inter_repr: Vec::new(),
        }
    }

    fn gen_ir(&mut self, ast: Vec<Expression>) {
        for expr in ast {
            self.get_value(&expr);
        }
    }

    fn get_value(&mut self, expr: &Expression) -> String {
        match &expr.kind {
            ExpressionKind::Identifier(id) => id.to_string(),
            ExpressionKind::IntLit(int) => int.to_string(),
            ExpressionKind::StringLit(string) => string.to_string(),
            ExpressionKind::BoolLit(bool) => {
                if bool == "true" {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            }
            ExpressionKind::Call(name, arg) => {
                let arg = self.get_value(arg);
                self.inter_repr.push(Ir::Param { src: arg.clone() });
                self.inter_repr.push(Ir::Call {
                    label: name.to_string(),
                });
                arg
            }
            ExpressionKind::Assign(ref dest, ref src)
            | ExpressionKind::Declare(ref dest, ref src) => {
                let dest_str = self.get_value(dest);
                // no need to call get_value on not because temporary value can be assigned
                // directly
                let instruction = match &src.kind {
                    ExpressionKind::Not(expr) => Ir::Not {
                        dest: dest_str.clone(),
                        src: self.get_value(expr),
                    },
                    ExpressionKind::Op(ref src1, op, ref src2) => Ir::Op {
                        dest: dest_str.clone(),
                        src1: self.get_value(src1),
                        op: op.clone(),
                        src2: self.get_value(src2),
                    },
                    _ => Ir::Ass {
                        dest: dest_str.clone(),
                        src: self.get_value(src),
                    },
                };
                self.inter_repr.push(instruction);

                dest_str
            }
            ExpressionKind::Op(ref src1, op, ref src2) => {
                self.nums.tmp += 1;
                let dest = format!("t{}", self.nums.tmp);

                let arg1 = self.get_value(src1);
                let arg2 = self.get_value(src2);

                self.inter_repr.push(Ir::Op {
                    dest: dest.clone(),
                    src1: arg1.clone(),
                    op: op.clone(),
                    src2: arg2.clone(),
                });

                self.type_table
                    .insert(dest.clone(), IrGenerator::get_op_type(op));

                dest
            }

            ExpressionKind::If(cond, seq) => {
                // todo: remove current_ifs
                self.nums.ifs += 1;
                let current_ifs = self.nums.ifs;

                self.get_condition(cond, &expr.kind, current_ifs);

                let seq_value = self.get_value(seq);

                self.inter_repr
                    .push(Ir::Label(format!("end_if_{}", current_ifs)));

                seq_value
            }
            ExpressionKind::IfElse(cond, if_seq, else_seq) => {
                self.nums.ifs += 1;
                let current_ifs = self.nums.ifs;

                self.get_condition(cond, &expr.kind, current_ifs);

                let if_seq_value = self.get_value(if_seq);
                self.inter_repr.push(Ir::Goto {
                    label: format!("end_if_{}", current_ifs),
                });

                self.inter_repr
                    .push(Ir::Label(format!("else_{}", current_ifs)));
                self.get_value(else_seq);
                self.inter_repr
                    .push(Ir::Label(format!("end_if_{}", current_ifs)));

                if_seq_value
            }
            ExpressionKind::Loop(seq) => {
                self.nums.loops += 1;
                let current_loops = self.nums.loops;

                self.inter_repr
                    .push(Ir::Label(format!("loop_{}", current_loops)));
                let seq_value = self.get_value(seq);

                self.inter_repr.push(Ir::Goto {
                    label: format!("loop_{}", current_loops),
                });

                self.inter_repr
                    .push(Ir::Label(format!("loop_end_{}", current_loops)));

                seq_value
            }
            ExpressionKind::While(cond, seq) => {
                self.nums.loops += 1;
                let current_loops = self.nums.loops;

                self.inter_repr
                    .push(Ir::Label(format!("loop_{}", current_loops)));

                self.get_condition(cond, &expr.kind, current_loops);

                let seq_value = self.get_value(seq);
                self.inter_repr.push(Ir::Goto {
                    label: format!("loop_{}", current_loops),
                });

                self.inter_repr
                    .push(Ir::Label(format!("loop_end_{}", current_loops)));

                seq_value
            }
            ExpressionKind::Break => {
                self.inter_repr.push(Ir::Goto {
                    label: format!("loop_end_{}", self.nums.loops),
                });
                "".to_string()
            }
            ExpressionKind::Continue => {
                self.inter_repr.push(Ir::Goto {
                    label: format!("loop_{}", self.nums.loops),
                });
                "".to_string()
            }
            ExpressionKind::Sequence(seq) => {
                for expr in &seq[0..seq.len() - 1] {
                    self.get_value(expr);
                }
                self.get_value(&seq[seq.len() - 1])
            }
            ExpressionKind::Not(arg) => {
                self.nums.tmp += 1;
                let destination = format!("t{}", self.nums.tmp);

                let arg1 = self.get_value(arg);

                self.inter_repr.push(Ir::Not {
                    dest: destination.clone(),
                    src: arg1.clone(),
                });

                self.type_table
                    .insert(destination.clone(), "bool".to_string());

                destination
            }
        }
    }

    fn get_op_type(op: &Op) -> String {
        match op {
            Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Mod => "int".to_string(),
            Op::And | Op::Or | Op::Eq | Op::Neq | Op::Lt | Op::Leq | Op::Gt | Op::Geq => {
                "bool".to_string()
            }
        }
    }

    fn get_condition(&mut self, cond: &Expression, kind: &ExpressionKind, num: u32) {
        if let ExpressionKind::Op(src1, op, src2) = &cond.kind {
            if *op != Op::And && *op != Op::Or {
                let src1 = self.get_value(src1);
                let src2 = self.get_value(src2);
                return self.inter_repr.push(Ir::IfGoto {
                    src1,
                    src2,
                    cond: match op {
                        Op::Eq => Op::Neq,
                        Op::Neq => Op::Eq,
                        Op::Lt => Op::Geq,
                        Op::Leq => Op::Gt,
                        Op::Gt => Op::Leq,
                        Op::Geq => Op::Lt,
                        _ => panic!("Must be a boolean expression"),
                    },
                    label: match kind {
                        ExpressionKind::If(..) => format!("end_if_{}", num),
                        ExpressionKind::IfElse(..) => format!("else_{}", num),
                        ExpressionKind::While(..) => format!("loop_end_{}", num),
                        _ => panic!("Not a condition expression"),
                    },
                });
            }
        }
        let cond_value = self.get_value(cond);
        self.inter_repr.push(Ir::IfGoto {
            src1: cond_value,
            src2: "0".to_string(),
            cond: Op::Eq,
            label: match kind {
                ExpressionKind::If(..) => format!("end_if_{}", num),
                ExpressionKind::IfElse(..) => format!("else_{}", num),
                ExpressionKind::While(..) => format!("loop_end_{}", num),
                _ => panic!("Not a condition expression"),
            },
        });
    }
}

pub fn generate_ir(ast: Vec<Expression>, type_table: &mut HashMap<String, String>) -> Vec<Ir> {
    let mut generator = IrGenerator::new(type_table);

    generator.gen_ir(ast);

    generator.inter_repr
}
