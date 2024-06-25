use crate::ir_generation::Ir;

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
        if let Ir::Assign(dest, src) = instruction {
            if dest == src {
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
        if let Ir::Assign(dest, src) = instruction {
            if src.parse::<i64>().is_ok() {
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
            Ir::Add(dest, src) => {
                if src == source {
                    *instruction = Ir::Add(dest.to_string(), constant.to_string());
                }
            }
            Ir::Sub(dest, src) => {
                if src == source {
                    *instruction = Ir::Sub(dest.to_string(), constant.to_string());
                }
            }
            Ir::Mul(..) => {
                needed = true;
            }
            Ir::Div(..) => {
                needed = true;
            }
            Ir::Mod(..) => {
                needed = true;
            }
            Ir::Assign(dest, src) => {
                if src == source {
                    *instruction = Ir::Assign(dest.to_string(), constant.to_string());
                }
            }
            Ir::Exit(src) => {
                if src == source {
                    *instruction = Ir::Exit(constant.to_string());
                }
            }
            Ir::JmpCmp(arg1, arg2, label, jmp_type) => {
                if arg1 == source {
                    *instruction = Ir::JmpCmp(
                        constant.to_string(),
                        arg2.to_string(),
                        label.to_string(),
                        jmp_type.clone(),
                    );
                } else if arg2 == source {
                    *instruction = Ir::JmpCmp(
                        arg1.to_string(),
                        constant.to_string(),
                        label.to_string(),
                        jmp_type.clone(),
                    );
                }
            }
            _ => {}
        }
    }
    needed
}
