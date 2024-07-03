use std::collections::HashMap;

use crate::parser::*;

pub mod ir_pretty;

#[derive(Debug, PartialEq, Clone)]
pub enum Ir {
    // Operations
    Op1 {
        // operations of arity 1
        dest: String,
        op: Op1,
        src: String,
    },
    Op2 {
        // operations of arity 2
        dest: String,
        src1: String,
        op: Op2,
        src2: String,
    },

    // Control-flow
    Label(String),
    Goto {
        label: String,
    },
    IfGoto {
        // if src != 0 goto label
        src: String,
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

#[derive(Debug, PartialEq, Clone)]
pub enum Op1 {
    Ass,
    Not,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Op2 {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
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
            ExpressionKind::Identifier(id) => id.to_string(),
            ExpressionKind::IntLit(int) => int.to_string(),
            ExpressionKind::BoolLit(bool) => {
                if bool == "true" {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            }
            ExpressionKind::Exit(val) => {
                let arg = self.get_value(val);
                self.inter_repr.push(Ir::Param { src: arg.clone() });
                self.inter_repr.push(Ir::Call {
                    label: "exit".to_string(),
                });
                arg
            }
            ExpressionKind::Assign(ref dest, ref src)
            | ExpressionKind::Declare(ref dest, ref src) => {
                let dest_str = self.get_value(dest);
                // no need to call get_value on not because temporary value can be assigned
                // directly
                let instruction = match &src.kind {
                    ExpressionKind::Not(expr) => Ir::Op1 {
                        dest: dest_str.clone(),
                        op: Op1::Not,
                        src: self.get_value(&expr),
                    },
                    ExpressionKind::Add(ref src1, ref src2)
                    | ExpressionKind::Sub(ref src1, ref src2)
                    | ExpressionKind::Mul(ref src1, ref src2)
                    | ExpressionKind::Div(ref src1, ref src2)
                    | ExpressionKind::Mod(ref src1, ref src2)
                    | ExpressionKind::And(ref src1, ref src2)
                    | ExpressionKind::Or(ref src1, ref src2)
                    | ExpressionKind::Eq(ref src1, ref src2)
                    | ExpressionKind::Neq(ref src1, ref src2)
                    | ExpressionKind::Lt(ref src1, ref src2)
                    | ExpressionKind::Leq(ref src1, ref src2)
                    | ExpressionKind::Gt(ref src1, ref src2)
                    | ExpressionKind::Geq(ref src1, ref src2) => Ir::Op2 {
                        dest: dest_str.clone(),
                        src1: self.get_value(src1),
                        op: IrGenerator::get_operation(&src),
                        src2: self.get_value(src2),
                    },
                    _ => Ir::Op1 {
                        dest: dest_str.clone(),
                        op: Op1::Ass,
                        src: self.get_value(src),
                    },
                };
                self.inter_repr.push(instruction);

                dest_str
            }
            ExpressionKind::Add(ref src1, ref src2)
            | ExpressionKind::Sub(ref src1, ref src2)
            | ExpressionKind::Mul(ref src1, ref src2)
            | ExpressionKind::Div(ref src1, ref src2)
            | ExpressionKind::Mod(ref src1, ref src2)
            | ExpressionKind::And(ref src1, ref src2)
            | ExpressionKind::Or(ref src1, ref src2)
            | ExpressionKind::Eq(ref src1, ref src2)
            | ExpressionKind::Neq(ref src1, ref src2)
            | ExpressionKind::Lt(ref src1, ref src2)
            | ExpressionKind::Leq(ref src1, ref src2)
            | ExpressionKind::Gt(ref src1, ref src2)
            | ExpressionKind::Geq(ref src1, ref src2) => {
                self.nums.tmp += 1;
                let dest = format!("t{}", self.nums.tmp);

                let arg1 = self.get_value(src1);
                let arg2 = self.get_value(src2);

                self.inter_repr.push(Ir::Op2 {
                    dest: dest.clone(),
                    src1: arg1.clone(),
                    op: IrGenerator::get_operation(expr),
                    src2: arg2.clone(),
                });

                self.type_table
                    .insert(dest.clone(), IrGenerator::get_op_type(expr));

                dest
            }

            ExpressionKind::If(cond, seq) => {
                // todo: remove current_ifs
                self.nums.ifs += 1;
                let current_ifs = self.nums.ifs;
                let src = self.get_value(cond);

                self.inter_repr.push(Ir::IfGoto {
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

                self.inter_repr.push(Ir::IfGoto {
                    src,
                    label: format!("else_{}", current_ifs),
                });

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
                self.inter_repr
                    .push(Ir::Label(format!("loop_{}", self.nums.loops)));
                let seq_value = self.get_value(seq);

                self.inter_repr.push(Ir::Goto {
                    label: format!("loop_{}", self.nums.loops),
                });

                self.inter_repr
                    .push(Ir::Label(format!("loop_end_{}", self.nums.loops)));
                self.nums.loops += 1;

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

                self.inter_repr.push(Ir::Op1 {
                    dest: destination.clone(),
                    src: arg1.clone(),
                    op: Op1::Not,
                });

                self.type_table
                    .insert(destination.clone(), "bool".to_string());

                destination
            }
        }
    }

    fn get_operation(expr: &Expression) -> Op2 {
        match expr.kind {
            ExpressionKind::Add(..) => Op2::Add,
            ExpressionKind::Sub(..) => Op2::Sub,
            ExpressionKind::Mul(..) => Op2::Mul,
            ExpressionKind::Div(..) => Op2::Div,
            ExpressionKind::Mod(..) => Op2::Mod,
            ExpressionKind::And(..) => Op2::And,
            ExpressionKind::Or(..) => Op2::Or,
            ExpressionKind::Eq(..) => Op2::Eq,
            ExpressionKind::Neq(..) => Op2::Neq,
            ExpressionKind::Lt(..) => Op2::Lt,
            ExpressionKind::Leq(..) => Op2::Leq,
            ExpressionKind::Gt(..) => Op2::Gt,
            ExpressionKind::Geq(..) => Op2::Geq,
            _ => panic!("Given expression does not correspond to an operation."),
        }
    }

    fn get_op_type(expr: &Expression) -> String {
        match expr.kind {
            ExpressionKind::Add(..)
            | ExpressionKind::Sub(..)
            | ExpressionKind::Mul(..)
            | ExpressionKind::Div(..)
            | ExpressionKind::Mod(..) => "int".to_string(),
            ExpressionKind::And(..)
            | ExpressionKind::Or(..)
            | ExpressionKind::Eq(..)
            | ExpressionKind::Neq(..)
            | ExpressionKind::Lt(..)
            | ExpressionKind::Leq(..)
            | ExpressionKind::Gt(..)
            | ExpressionKind::Geq(..) => "bool".to_string(),
            _ => panic!("Given expression does not correspond to an operation."),
        }
    }
}

pub fn generate_ir(ast: Vec<Expression>, type_table: &mut HashMap<String, String>) -> Vec<Ir> {
    let mut generator = IrGenerator::new(type_table);

    generator.gen_ir(ast);

    generator.inter_repr
}
