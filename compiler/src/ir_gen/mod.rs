use std::collections::HashMap;

use crate::parser::*;

pub mod ir_pretty;

#[derive(Debug, PartialEq, Clone)]
pub enum Ir {
    Op { dest: String, src: String, op: Op },
    Exit { src: String },
    Set { dest: String, cond: Cond },
    Label(String),
    Jmp { label: String },
    JmpCond { src: String, label: String },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Op {
    Assign,
    Cmp,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Not,
    And,
    Or,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Cond {
    Eq,
    Neq,
    Lt,
    Leq,
    Gt,
    Geq,
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
            ExpressionKind::Exit(val) => {
                let arg = self.get_value(val);
                self.inter_repr.push(Ir::Exit { src: arg.clone() });
                arg
            }
            ExpressionKind::Assign(ref dest, ref src)
            | ExpressionKind::Declare(ref dest, ref src) => {
                let arg1 = self.get_value(dest);
                let arg2 = self.get_value(src);

                self.inter_repr.push(Ir::Op {
                    dest: arg1.clone(),
                    src: arg2,
                    op: Op::Assign,
                });

                arg1
            }
            ExpressionKind::Add(ref dest, ref src)
            | ExpressionKind::Sub(ref dest, ref src)
            | ExpressionKind::Mul(ref dest, ref src)
            | ExpressionKind::Div(ref dest, ref src)
            | ExpressionKind::Mod(ref dest, ref src)
            | ExpressionKind::And(ref dest, ref src)
            | ExpressionKind::Or(ref dest, ref src) => {
                self.nums.tmp += 1;
                let destination = format!("t{}", self.nums.tmp);

                let arg1 = self.get_value(dest);
                let arg2 = self.get_value(src);

                self.inter_repr.push(Ir::Op {
                    dest: destination.clone(),
                    src: arg1.clone(),
                    op: Op::Assign,
                });

                self.type_table
                    .insert(destination.clone(), self.type_table[&arg2].clone());

                self.inter_repr
                    .push(IrGenerator::get_operation(expr, destination.clone(), arg2));
                destination
            }
            ExpressionKind::Eq(ref dest, ref src)
            | ExpressionKind::Neq(ref dest, ref src)
            | ExpressionKind::Lt(ref dest, ref src)
            | ExpressionKind::Leq(ref dest, ref src)
            | ExpressionKind::Gt(ref dest, ref src)
            | ExpressionKind::Geq(ref dest, ref src) => {
                let arg1 = self.get_value(dest);
                let arg2 = self.get_value(src);

                self.inter_repr.push(Ir::Op {
                    dest: arg1.clone(),
                    src: arg2,
                    op: Op::Cmp,
                });

                self.nums.tmp += 1;
                let destination = format!("t{}", self.nums.tmp);

                self.type_table
                    .insert(destination.clone(), "bool".to_string());

                self.inter_repr.push(Ir::Set {
                    dest: destination,
                    cond: IrGenerator::get_condition(expr),
                });

                format!("t{}", self.nums.tmp)
            }
            ExpressionKind::IntLit(int) => {
                self.nums.tmp += 1;

                let destination = format!("t{}", self.nums.tmp);

                self.inter_repr.push(Ir::Op {
                    dest: destination.clone(),
                    src: int.to_string(),
                    op: Op::Assign,
                });

                self.type_table
                    .insert(destination.clone(), "int".to_string());

                destination
            }
            ExpressionKind::BoolLit(bool) => {
                self.nums.tmp += 1;
                let destination = format!("t{}", self.nums.tmp);

                self.inter_repr.push(Ir::Op {
                    dest: destination.clone(),
                    src: if bool == "true" {
                        "1".to_string()
                    } else {
                        "0".to_string()
                    },
                    op: Op::Assign,
                });

                self.type_table
                    .insert(destination.clone(), "bool".to_string());

                destination
            }
            ExpressionKind::Identifier(id) => id.to_string(),
            ExpressionKind::If(cond, seq) => {
                // todo: remove current_ifs
                self.nums.ifs += 1;
                let current_ifs = self.nums.ifs;
                let src = self.get_value(cond);

                self.inter_repr.push(Ir::JmpCond {
                    src,
                    label: format!("end_if_{}", current_ifs),
                });

                let seq_value = self.get_value(seq);

                self.inter_repr
                    .push(Ir::Label(format!("end_if_{}", current_ifs)));

                seq_value
            }
            ExpressionKind::IfElse(cond, if_seq, else_seq) => {
                self.nums.ifs += 1;
                let current_ifs = self.nums.ifs;

                let src = self.get_value(cond);

                self.inter_repr.push(Ir::JmpCond {
                    src,
                    label: format!("else_{}", current_ifs),
                });

                let if_seq_value = self.get_value(if_seq);
                self.inter_repr.push(Ir::Jmp {
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
                self.inter_repr
                    .push(Ir::Label(format!("loop_{}", self.nums.loops)));
                let seq_value = self.get_value(seq);

                self.inter_repr.push(Ir::Jmp {
                    label: format!("loop_{}", self.nums.loops),
                });

                self.inter_repr
                    .push(Ir::Label(format!("loop_end_{}", self.nums.loops)));
                self.nums.loops += 1;

                seq_value
            }
            ExpressionKind::Break => {
                self.inter_repr.push(Ir::Jmp {
                    label: format!("loop_end_{}", self.nums.loops),
                });
                "".to_string()
            }
            ExpressionKind::Continue => {
                self.inter_repr.push(Ir::Jmp {
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

                self.inter_repr.push(Ir::Op {
                    dest: destination.clone(),
                    src: arg1.clone(),
                    op: Op::Assign,
                });

                self.inter_repr.push(Ir::Op {
                    dest: destination.clone(),
                    src: destination.clone(),
                    op: Op::Not,
                });

                self.type_table
                    .insert(destination.clone(), "bool".to_string());

                destination
            }
        }
    }

    fn get_operation(operation: &Expression, arg1: String, arg2: String) -> Ir {
        Ir::Op {
            dest: arg1,
            src: arg2,
            op: match operation.kind {
                ExpressionKind::Add(..) => Op::Add,
                ExpressionKind::Sub(..) => Op::Sub,
                ExpressionKind::Mul(..) => Op::Mul,
                ExpressionKind::Div(..) => Op::Div,
                ExpressionKind::Mod(..) => Op::Mod,
                ExpressionKind::And(..) => Op::And,
                ExpressionKind::Or(..) => Op::Or,
                _ => panic!("Unexpected operation"),
            },
        }
    }

    fn get_condition(expr: &Expression) -> Cond {
        match &expr.kind {
            ExpressionKind::Eq(..) => Cond::Eq,
            ExpressionKind::Neq(..) => Cond::Neq,
            ExpressionKind::Lt(..) => Cond::Lt,
            ExpressionKind::Leq(..) => Cond::Leq,
            ExpressionKind::Gt(..) => Cond::Gt,
            ExpressionKind::Geq(..) => Cond::Geq,
            _ => panic!("Invalid condition"),
        }
    }
}

pub fn generate_ir(ast: Vec<Expression>, type_table: &mut HashMap<String, String>) -> Vec<Ir> {
    let mut generator = IrGenerator::new(type_table);

    generator.gen_ir(ast);

    generator.inter_repr
}
