use crate::ir_generation::*;

pub fn optimize(ir: Vec<Ir>) -> Vec<Ir> {
    let mut optimized_ir = ir;

    optimized_ir = remove_self_assign(optimized_ir);
    optimized_ir = constant_propagation(optimized_ir);

    optimized_ir
}

fn remove_self_assign(ir: Vec<Ir>) -> Vec<Ir> {
    let mut new_ir = ir.clone();
    let mut num_removed = 0;

    for (i, instruction) in ir.iter().enumerate() {
        if let Ir::Op { dest, src, op } = instruction {
            if *op == Op::Assign && dest == src {
                new_ir.remove(i - num_removed);
                num_removed += 1;
            }
        }
    }

    new_ir
}

fn constant_propagation(ir: Vec<Ir>) -> Vec<Ir> {
    let mut new_ir = ir.clone();
    let mut num_removed = 0;

    for (i, instruction) in ir.iter().enumerate() {
        if let Ir::Op { dest, src, op } = instruction {
            if *op == Op::Assign && src.parse::<i64>().is_ok() {
                let needed = replace_with_constants(&mut new_ir, dest, src);
                if !needed {
                    new_ir.remove(i - num_removed);
                    num_removed += 1;
                }
            }
        }
    }

    new_ir
}

fn replace_with_constants(ir: &mut Vec<Ir>, source: &String, constant: &String) -> bool {
    let mut needed = false;
    for instruction in ir {
        match instruction {
            Ir::Op { src, dest, op } => match op {
                Op::Assign | Op::Add | Op::Sub | Op::And | Op::Or | Op::Not | Op::Cmp => {
                    if src == source {
                        *instruction = Ir::Op {
                            dest: dest.to_string(),
                            src: constant.to_string(),
                            op: op.clone(),
                        };
                    }
                }
                Op::Mul | Op::Div | Op::Mod => {
                    if src == source || dest == source {
                        needed = true;
                    }
                }
            },
            Ir::Exit { src } => {
                if src == source {
                    *instruction = Ir::Exit {
                        src: constant.to_string(),
                    };
                }
            }
            Ir::JmpCond { src, label } => {
                if src == source {
                    *instruction = Ir::JmpCond {
                        src: constant.to_string(),
                        label: label.to_string(),
                    };
                }
            }
            _ => {}
        }
    }
    needed
}
