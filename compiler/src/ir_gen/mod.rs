use std::collections::HashMap;

use crate::ir::*;
use crate::op::*;
use crate::parser::expression::*;

pub mod ir;

pub fn generate_ir(ast: Vec<Expression>, type_table: &mut HashMap<String, String>) -> Ir {
    let mut generator = IrGenerator::new(type_table);

    generator.gen_ir(ast);

    generator.ir
}

pub struct IrGenerator<'a> {
    type_table: &'a mut HashMap<String, String>,
    nums: Nums,
    ir: Ir,
}

struct Nums {
    tmp: u32,
    ifs: u32,
    loops: u32,
    buf: u32,
}

impl IrGenerator<'_> {
    fn new(type_table: &mut HashMap<String, String>) -> IrGenerator {
        IrGenerator {
            type_table,
            nums: Nums {
                tmp: 0,
                ifs: 0,
                loops: 0,
                buf: 0,
            },
            ir: Ir::new(),
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
            ExpressionKind::StringLit(string) => {
                self.nums.buf += 1;
                self.ir.add_data(
                    format!("buf_{}", self.nums.buf),
                    string.to_string() + ", 10",
                    string.len() - 1,
                );
                format!("buf_{}", self.nums.buf)
            }
            ExpressionKind::BoolLit(bool) => {
                if bool == "true" {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            }
            ExpressionKind::Call(name, arg) => {
                let arg = self.get_value(arg);
                match name.as_str() {
                    "exit" => self
                        .ir
                        .add_instruction(IrInstruction::Param { src: arg.clone() }),
                    "print" => {
                        self.ir.add_instruction(IrInstruction::Param {
                            src: "1".to_string(),
                        });
                        self.ir
                            .add_instruction(IrInstruction::Param { src: arg.clone() });

                        let mut string_size = 0;
                        for buffer in &self.ir.data {
                            if arg == buffer.label {
                                string_size = buffer.size;
                                break;
                            }
                        }
                        self.ir.add_instruction(IrInstruction::Param {
                            src: format!("{}", string_size),
                        });
                    }
                    _ => panic!("Undefined function"),
                };
                self.ir.add_instruction(IrInstruction::Call {
                    label: name.to_string(),
                });
                arg
            }
            ExpressionKind::Assign(ref dest, ref src)
            | ExpressionKind::Declare(ref dest, ref src) => {
                let dest_str = self.get_value(dest);

                if let ExpressionKind::Array(..) = &src.kind {
                    let buf_name = self.get_value(src);
                    for buffer in &mut self.ir.data {
                        if buffer.label == buf_name {
                            buffer.label = dest_str.clone();
                        }
                    }
                } else {
                    // no need to call get_value on not because temporary value can be assigned directly
                    let instruction = match &src.kind {
                        ExpressionKind::Not(expr) => IrInstruction::Not {
                            dest: dest_str.clone(),
                            src: self.get_value(expr),
                        },
                        ExpressionKind::Op(ref src1, op, ref src2) => IrInstruction::Op {
                            dest: dest_str.clone(),
                            src1: self.get_value(src1),
                            op: op.clone(),
                            src2: self.get_value(src2),
                        },
                        _ => IrInstruction::Ass {
                            dest: dest_str.clone(),
                            src: self.get_value(src),
                        },
                    };
                    self.ir.add_instruction(instruction);
                }

                dest_str
            }
            ExpressionKind::Array(contents) => {
                let mut string = String::new();
                for expr in contents {
                    match &expr.kind {
                        ExpressionKind::IntLit(int) => string.push_str(&int),
                        _ => panic!("Array must be of int literals"),
                    }
                    string.push_str(", ")
                }
                string.pop();
                string.pop();
                self.nums.buf += 1;
                self.ir
                    .add_data(format!("buf_{}", self.nums.buf), string, contents.len());
                format!("buf_{}", self.nums.buf)
            }

            ExpressionKind::Op(ref src1, op, ref src2) => {
                self.nums.tmp += 1;
                let dest = format!("t{}", self.nums.tmp);

                let arg1 = self.get_value(src1);
                let arg2 = self.get_value(src2);

                self.ir.add_instruction(IrInstruction::Op {
                    dest: dest.clone(),
                    src1: arg1.clone(),
                    op: op.clone(),
                    src2: arg2.clone(),
                });

                self.type_table
                    .insert(dest.clone(), op.get_type().to_string());

                dest
            }

            ExpressionKind::If(cond, seq) => {
                // todo: remove current_ifs
                self.nums.ifs += 1;
                let current_ifs = self.nums.ifs;

                self.get_condition(cond, &expr.kind, current_ifs);

                let seq_value = self.get_value(seq);

                self.ir
                    .add_instruction(IrInstruction::Label(format!("end_if_{}", current_ifs)));

                seq_value
            }
            ExpressionKind::IfElse(cond, if_seq, else_seq) => {
                self.nums.ifs += 1;
                let current_ifs = self.nums.ifs;

                self.get_condition(cond, &expr.kind, current_ifs);

                let if_seq_value = self.get_value(if_seq);
                self.ir.add_instruction(IrInstruction::Goto {
                    label: format!("end_if_{}", current_ifs),
                });

                self.ir
                    .add_instruction(IrInstruction::Label(format!("else_{}", current_ifs)));
                self.get_value(else_seq);
                self.ir
                    .add_instruction(IrInstruction::Label(format!("end_if_{}", current_ifs)));

                if_seq_value
            }
            ExpressionKind::Loop(seq) => {
                self.nums.loops += 1;
                let current_loops = self.nums.loops;

                self.ir
                    .add_instruction(IrInstruction::Label(format!("loop_{}", current_loops)));
                let seq_value = self.get_value(seq);

                self.ir.add_instruction(IrInstruction::Goto {
                    label: format!("loop_{}", current_loops),
                });

                self.ir
                    .add_instruction(IrInstruction::Label(format!("loop_end_{}", current_loops)));

                seq_value
            }
            ExpressionKind::While(cond, seq) => {
                self.nums.loops += 1;
                let current_loops = self.nums.loops;

                self.ir
                    .add_instruction(IrInstruction::Label(format!("loop_{}", current_loops)));

                self.get_condition(cond, &expr.kind, current_loops);

                let seq_value = self.get_value(seq);
                self.ir.add_instruction(IrInstruction::Goto {
                    label: format!("loop_{}", current_loops),
                });

                self.ir
                    .add_instruction(IrInstruction::Label(format!("loop_end_{}", current_loops)));

                seq_value
            }
            ExpressionKind::Break => {
                self.ir.add_instruction(IrInstruction::Goto {
                    label: format!("loop_end_{}", self.nums.loops),
                });
                "".to_string()
            }
            ExpressionKind::Continue => {
                self.ir.add_instruction(IrInstruction::Goto {
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

                self.ir.add_instruction(IrInstruction::Not {
                    dest: destination.clone(),
                    src: arg1.clone(),
                });

                self.type_table
                    .insert(destination.clone(), "bool".to_string());

                destination
            }
            ExpressionKind::Idx(id, offset) => {
                format!("[{} + {}]", id.to_str(), offset.to_str())
            }
        }
    }

    fn get_condition(&mut self, cond: &Expression, kind: &ExpressionKind, num: u32) {
        if let ExpressionKind::Op(src1, op, src2) = &cond.kind {
            if *op != Op::And && *op != Op::Or {
                let src1 = self.get_value(src1);
                let src2 = self.get_value(src2);
                return self.ir.add_instruction(IrInstruction::IfGoto {
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
        self.ir.add_instruction(IrInstruction::IfGoto {
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
