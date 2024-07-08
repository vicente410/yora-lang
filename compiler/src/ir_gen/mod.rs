use std::collections::HashMap;

use crate::core::*;
use crate::ir::*;
use crate::op::*;
use crate::parser::expression::*;

pub mod ir;

pub fn generate_ir(ast: Vec<Expression>, type_table: &mut HashMap<String, PrimitiveType>) -> Ir {
    let mut generator = IrGenerator::new(type_table);

    generator.gen_ir(ast);

    generator.ir
}

pub struct IrGenerator<'a> {
    type_table: &'a mut HashMap<String, PrimitiveType>,
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
    fn new(type_table: &mut HashMap<String, PrimitiveType>) -> IrGenerator {
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

    fn get_value(&mut self, expr: &Expression) -> Value {
        match &expr.kind {
            ExpressionKind::Identifier(id) => Value::Identifier { id: id.to_string() },
            ExpressionKind::IntLit(int) => Value::Constant {
                value: int.to_string(),
            },
            ExpressionKind::StringLit(string) => self.get_string_lit(string),
            ExpressionKind::BoolLit(bool) => self.get_bool_lit(bool),
            ExpressionKind::ArrayLit(contents) => self.get_array_lit(contents),
            ExpressionKind::Call(name, arg) => self.get_call(name, arg),
            ExpressionKind::Assign(ref dest, ref src)
            | ExpressionKind::Declare(ref dest, ref src, _) => self.get_declare_assign(dest, src),
            ExpressionKind::Op(ref src1, op, ref src2) => self.get_operation(src1, op, src2),
            ExpressionKind::If(cond, seq) => self.get_if(cond, seq, expr),
            ExpressionKind::IfElse(cond, if_seq, else_seq) => {
                self.get_if_else(cond, if_seq, else_seq, expr)
            }
            ExpressionKind::Loop(seq) => self.get_loop(seq),
            ExpressionKind::While(cond, seq) => self.get_while(cond, seq, expr),
            ExpressionKind::Break => self.get_break(),
            ExpressionKind::Continue => self.get_continue(),
            ExpressionKind::Sequence(seq) => self.get_sequence(seq),
            ExpressionKind::Not(arg) => self.get_not(arg),
            ExpressionKind::Idx(id, offset) => self.get_idx(id, offset),
        }
    }

    fn get_string_lit(&mut self, string: &str) -> Value {
        self.nums.buf += 1;
        self.ir.add_data(
            format!("buf_{}", self.nums.buf),
            string.to_string() + ", 10",
            string.len() - 1,
        );
        Value::Identifier {
            id: format!("buf_{}", self.nums.buf),
        }
    }

    fn get_bool_lit(&mut self, bool: &str) -> Value {
        Value::Constant {
            value: if bool == "true" { "1" } else { "0" }.to_string(),
        }
    }

    fn get_array_lit(&mut self, contents: &Vec<Expression>) -> Value {
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
        Value::Identifier {
            id: format!("buf_{}", self.nums.buf),
        }
    }

    fn get_call(&mut self, name: &str, arg: &Expression) -> Value {
        let arg = self.get_value(arg);
        match name {
            "exit" => self
                .ir
                .add_instruction(IrInstruction::Param { src: arg.clone() }),
            "print" => {
                self.ir.add_instruction(IrInstruction::Param {
                    src: Value::Constant {
                        value: "1".to_string(),
                    },
                });
                self.ir
                    .add_instruction(IrInstruction::Param { src: arg.clone() });

                let mut string_size = 0;
                for buffer in &self.ir.data {
                    if let Value::Identifier { ref id } = arg {
                        if *id == buffer.label {
                            string_size = buffer.size;
                            break;
                        }
                    }
                }
                self.ir.add_instruction(IrInstruction::Param {
                    src: Value::Constant {
                        value: format!("{}", string_size),
                    },
                });
            }
            _ => panic!("Undefined function"),
        };
        self.ir.add_instruction(IrInstruction::Call {
            label: name.to_string(),
        });
        arg
    }

    fn get_declare_assign(&mut self, dest: &Expression, src: &Expression) -> Value {
        let dest_str = self.get_value(dest);

        if let ExpressionKind::ArrayLit(..) = &src.kind {
            let buf_name = self.get_value(src);
            for buffer in &mut self.ir.data {
                if let Value::Identifier { ref id } = buf_name {
                    if buffer.label == *id {
                        buffer.label = if let Value::Identifier { ref id } = dest_str {
                            (*id).clone()
                        } else {
                            panic!("Invalid value")
                        }
                    }
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

    fn get_operation(&mut self, src1: &Expression, op: &Op, src2: &Expression) -> Value {
        self.nums.tmp += 1;

        let dest = Value::Identifier {
            id: format!("t{}", self.nums.tmp),
        };

        let arg1 = self.get_value(src1);
        let arg2 = self.get_value(src2);

        self.ir.add_instruction(IrInstruction::Op {
            dest: dest.clone(),
            src1: arg1.clone(),
            op: op.clone(),
            src2: arg2.clone(),
        });

        if let Value::Identifier { ref id } = dest.clone() {
            self.type_table
                .insert((*id).clone(), get_type(op.get_type().to_string()));
        }

        dest
    }

    fn get_if(&mut self, cond: &Expression, seq: &Expression, expr: &Expression) -> Value {
        // todo: remove current_ifs
        self.nums.ifs += 1;
        let current_ifs = self.nums.ifs;

        self.get_condition(cond, &expr.kind, current_ifs);

        let seq_value = self.get_value(seq);

        self.ir
            .add_instruction(IrInstruction::Label(format!("end_if_{}", current_ifs)));

        seq_value
    }

    fn get_if_else(
        &mut self,
        cond: &Expression,
        if_seq: &Expression,
        else_seq: &Expression,
        expr: &Expression,
    ) -> Value {
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
            src2: Value::Constant {
                value: "0".to_string(),
            },
            cond: Op::Eq,
            label: match kind {
                ExpressionKind::If(..) => format!("end_if_{}", num),
                ExpressionKind::IfElse(..) => format!("else_{}", num),
                ExpressionKind::While(..) => format!("loop_end_{}", num),
                _ => panic!("Not a condition expression"),
            },
        });
    }

    fn get_loop(&mut self, seq: &Expression) -> Value {
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

    fn get_while(&mut self, cond: &Expression, seq: &Expression, expr: &Expression) -> Value {
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

    fn get_break(&mut self) -> Value {
        self.ir.add_instruction(IrInstruction::Goto {
            label: format!("loop_end_{}", self.nums.loops),
        });
        Value::Constant {
            value: "".to_string(),
        }
    }

    fn get_continue(&mut self) -> Value {
        self.ir.add_instruction(IrInstruction::Goto {
            label: format!("loop_{}", self.nums.loops),
        });
        Value::Constant {
            value: "".to_string(),
        }
    }

    fn get_sequence(&mut self, seq: &Vec<Expression>) -> Value {
        for expr in &seq[0..seq.len() - 1] {
            self.get_value(expr);
        }
        self.get_value(&seq[seq.len() - 1])
    }

    fn get_not(&mut self, arg: &Expression) -> Value {
        self.nums.tmp += 1;
        let destination = Value::Identifier {
            id: format!("t{}", self.nums.tmp),
        };

        let arg1 = self.get_value(arg);

        self.ir.add_instruction(IrInstruction::Not {
            dest: destination.clone(),
            src: arg1.clone(),
        });

        if let Value::Identifier { ref id } = destination.clone() {
            self.type_table.insert((*id).clone(), PrimitiveType::Bool);
        }
        destination
    }

    fn get_idx(&mut self, id: &Expression, offset: &Expression) -> Value {
        let offset_val = self.get_value(offset);
        Value::MemPos {
            id: id.to_str().to_string(),
            offset: if matches!(offset_val, Value::MemPos { .. }) {
                self.nums.tmp += 1;

                let destination = Value::Identifier {
                    id: format!("t{}", self.nums.tmp),
                };

                self.ir.add_instruction(IrInstruction::Ass {
                    dest: destination.clone(),
                    src: offset_val,
                });

                if let Value::Identifier { ref id } = destination.clone() {
                    self.type_table.insert((*id).clone(), PrimitiveType::I8);
                }

                Box::new(destination)
            } else {
                Box::new(self.get_value(offset))
            },
        }
    }
}
