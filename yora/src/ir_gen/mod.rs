use crate::core::*;
use crate::ir::*;
use crate::op::*;
use crate::parser::expression::*;
use crate::statement::*;

pub mod ir;

pub fn generate_ir(ast: Vec<Statement>) -> Ir {
    let mut generator = IrGenerator::new();

    generator.gen_ir(ast);

    generator.ir
}

pub struct IrGenerator {
    nums: Nums,
    ir: Ir,
}

struct Nums {
    tmp: u32,
    ifs: u32,
    loops: u32,
    buf: u32,
}

impl IrGenerator {
    fn new() -> IrGenerator {
        IrGenerator {
            nums: Nums {
                tmp: 0,
                ifs: 0,
                loops: 0,
                buf: 0,
            },
            ir: Ir::new(),
        }
    }

    fn gen_ir(&mut self, ast: Vec<Statement>) {
        for statement in ast {
            self.get_statement(&statement);
        }
    }

    fn get_statement(&mut self, statement: &Statement) {
        match &statement.kind {
            StatementKind::Procedure {
                name,
                args,
                ret,
                block,
            } => todo!(),
            StatementKind::Call { name, args } => self.get_call(name, args),
            StatementKind::Return { value } => todo!(),

            StatementKind::Declare { name, value, .. } => self.get_declare(name, value),
            StatementKind::Assign { dest, src } => self.get_assign(dest, src),

            StatementKind::If { cond, block } => self.get_if(cond, block, statement),
            StatementKind::IfElse {
                cond,
                true_block,
                false_block,
            } => self.get_if_else(cond, true_block, false_block, statement),
            StatementKind::Loop { block } => self.get_loop(block),
            StatementKind::While { cond, block } => self.get_while(cond, block, statement),
            StatementKind::Continue => self.get_continue(),
            StatementKind::Break => self.get_break(),
        }
    }

    fn get_array_lit(&mut self, contents: &Vec<Expression>) -> Value {
        let mut string = String::new();
        for expr in contents {
            match &expr.r#type {
                PrimitiveType::Int => string.push_str(&int),
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

    fn get_call(&mut self, name: &str, args: &Vec<Expression>) {
        let mut arg_vals = Vec::new();
        for arg in args {
            arg_vals.push(self.get_expression(arg));
        }

        match name {
            "exit" => self.ir.add_instruction(IrInstruction::Param {
                src: arg_vals[0].clone(),
                r#type: PrimitiveType::Int,
            }),
            "print" => {
                self.ir.add_instruction(IrInstruction::Param {
                    src: Value::Constant {
                        value: "1".to_string(),
                    },
                    r#type: PrimitiveType::Int,
                });
                self.ir.add_instruction(IrInstruction::Param {
                    src: arg_vals[0].clone(),
                    r#type: PrimitiveType::Arr(Box::new(PrimitiveType::Char)),
                });

                let mut string_size = 0; // get size of buffer from ir.data
                for buffer in &self.ir.data {
                    if let Value::Identifier { ref id } = arg_vals[0] {
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
                    r#type: PrimitiveType::Int,
                });
            }
            _ => panic!("Undefined function"),
        };
        self.ir.add_instruction(IrInstruction::Call {
            label: name.to_string(),
        });
    }

    fn get_declare(&mut self, name: &String, value: &Option<Expression>) {
        let src_val = self.get_expression(&value.unwrap());
        let name_val = Value::Identifier {
            id: name.to_string(),
        };

        self.ir.add_instruction(IrInstruction::Ass {
            dest: name_val,
            src: src_val,
            r#type: value.unwrap().r#type.unwrap(),
        });
    }

    fn get_assign(&mut self, dest: &Expression, src: &Expression) {
        let src_val = self.get_expression(src);
        let dest_val = self.get_expression(dest);

        self.ir.add_instruction(IrInstruction::Ass {
            dest: dest_val,
            src: src_val,
            r#type: dest_val.r#type.unwrap(),
        });
    }

    fn get_operation(
        &mut self,
        src1: &Expression,
        op: &Op,
        src2: &Expression,
        r#type: PrimitiveType,
    ) {
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
            r#type,
        });
    }

    fn get_if(&mut self, cond: &Expression, seq: &Expression, expr: &Expression) {
        // todo: remove current_ifs
        self.nums.ifs += 1;
        let current_ifs = self.nums.ifs;

        self.get_condition(cond, &expr.kind, current_ifs);

        let seq_value = self.get_value(seq);

        self.ir
            .add_instruction(IrInstruction::Label(format!("end_if_{}", current_ifs)));
    }

    fn get_if_else(
        &mut self,
        cond: &Expression,
        if_seq: &Expression,
        else_seq: &Expression,
        expr: &Expression,
    ) {
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
                let val1 = self.get_value(src1);
                let val2 = self.get_value(src2);
                return self.ir.add_instruction(IrInstruction::IfGoto {
                    src1: val1,
                    src2: val2,
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
                    r#type: src1.r#type.clone(),
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
            r#type: cond.r#type.clone(),
        });
    }

    fn get_loop(&mut self, block: &Vec<Statement>) {
        self.nums.loops += 1;
        let current_loops = self.nums.loops;

        self.ir
            .add_instruction(IrInstruction::Label(format!("loop_{}", current_loops)));
        for statement in block {
            self.get_statement(statement);
        }

        self.ir.add_instruction(IrInstruction::Goto {
            label: format!("loop_{}", current_loops),
        });

        self.ir
            .add_instruction(IrInstruction::Label(format!("loop_end_{}", current_loops)));
    }

    fn get_while(&mut self, cond: &Expression, seq: &Expression, expr: &Expression) {
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

    fn get_break(&mut self) {
        self.ir.add_instruction(IrInstruction::Goto {
            label: format!("loop_end_{}", self.nums.loops),
        });
    }

    fn get_continue(&mut self) {
        self.ir.add_instruction(IrInstruction::Goto {
            label: format!("loop_{}", self.nums.loops),
        });
    }

    fn get_not(&mut self, arg: &Expression) -> Value {
        self.nums.tmp += 1;
        let destination = Value::Identifier {
            id: format!("t{}", self.nums.tmp),
        };

        let val = self.get_value(arg);

        self.ir.add_instruction(IrInstruction::Not {
            dest: destination.clone(),
            src: val.clone(),
            r#type: arg.r#type.clone(),
        });

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
                    r#type: offset.r#type.clone(),
                });

                Box::new(destination)
            } else {
                Box::new(self.get_value(offset))
            },
        }
    }

    fn get_expression(&mut self, expr: &Expression) -> Value {
        match &expr.kind {
            ExpressionKind::Id(id) => Value::Identifier { id: id.to_string() },
            ExpressionKind::Lit(lit) => match expr.r#type.unwrap() {
                PrimitiveType::Int | PrimitiveType::Char => Value::Constant {
                    value: lit.to_string(),
                },
                PrimitiveType::Bool => Value::Constant {
                    value: if lit == "true" { "1" } else { "0" }.to_string(),
                },
                PrimitiveType::Arr(..) => panic!("Lit expressions cannot have type array") 
            },
            ExpressionKind::Array(contents) => {
                    self.nums.buf += 1;
                    self.ir.add_data(
                        format!("buf_{}", self.nums.buf),
                        lit.to_string() + ", 10",
                        lit.len() - 1,
                    );
                    Value::Identifier {
                        id: format!("buf_{}", self.nums.buf),
                    }
                }
            ExpressionKind::Call(name, args) => 
        }
    }
}
