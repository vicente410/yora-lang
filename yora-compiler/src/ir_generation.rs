use std::ops::Deref;

use crate::parser::Expression;

#[derive(Debug, PartialEq, Clone)]
pub enum Ir {
    Exit(String),
    Assign(String, String),
    Label(String),
    Jmp(String),
    JmpCmp(String, String, String, JmpType),
    Add(String, String),
    Sub(String, String),
    Mul(String, String),
    Div(String, String),
    Mod(String, String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum JmpType {
    Je,
    Jne,
    Jl,
    Jle,
    Jg,
    Jge,
}

struct Nums {
    tmp: u32,
    ifs: u32,
    loops: u32,
}

pub fn generate_ir(ast: Vec<Expression>) -> Vec<Ir> {
    let mut inter_repr = Vec::new();
    let mut tmp_vec = Vec::new();
    let mut nums = Nums {
        tmp: 0,
        ifs: 0,
        loops: 0,
    };

    for expr in ast {
        get_value(&expr, &mut tmp_vec, &mut nums);
        inter_repr.append(&mut tmp_vec);
        tmp_vec.clear();
    }

    inter_repr
}

fn get_value(expr: &Expression, tmp_vec: &mut Vec<Ir>, nums: &mut Nums) -> String {
    match expr {
        Expression::Exit(val) => {
            let arg = get_value(val, tmp_vec, nums);
            tmp_vec.push(Ir::Exit(arg.clone()));
            arg
        }
        Expression::Assign(ref dest, ref src) | Expression::Declare(ref dest, ref src) => {
            let arg1 = get_value(dest, tmp_vec, nums);
            let arg2 = get_value(src, tmp_vec, nums);
            tmp_vec.push(Ir::Assign(arg1.clone(), arg2));
            arg1
        }
        Expression::Add(ref dest, ref src)
        | Expression::Sub(ref dest, ref src)
        | Expression::Mul(ref dest, ref src)
        | Expression::Div(ref dest, ref src)
        | Expression::Mod(ref dest, ref src) => {
            let arg1 = get_value(dest, tmp_vec, nums);
            let arg2 = get_value(src, tmp_vec, nums);
            tmp_vec.push(get_operation(expr, arg1.clone(), arg2));
            arg1
        }
        Expression::IntLit(int) => {
            nums.tmp += 1;
            tmp_vec.push(Ir::Assign(format!("t{}", nums.tmp), int.to_string()));
            format!("t{}", nums.tmp)
        }
        Expression::BoolLit(bool) => {
            nums.tmp += 1;
            tmp_vec.push(Ir::Assign(
                format!("t{}", nums.tmp),
                if bool == "true" {
                    "1".to_string()
                } else {
                    "0".to_string()
                },
            ));
            format!("t{}", nums.tmp)
        }
        Expression::Identifier(id) => id.to_string(),
        Expression::If(cond, seq) => match cond.deref() {
            Expression::BoolLit(..) => {
                let value = get_value(cond, tmp_vec, nums);
                nums.ifs += 1;
                let current_ifs = nums.ifs;
                tmp_vec.push(Ir::JmpCmp(
                    value,
                    "0".to_string(),
                    format!("end_if_{}", current_ifs),
                    get_jump(cond),
                ));
                let seq_value = get_value(seq, tmp_vec, nums);
                tmp_vec.push(Ir::Label(format!("end_if_{}", current_ifs)));
                seq_value
            }
            Expression::Eq(cmp1, cmp2)
            | Expression::NotEq(cmp1, cmp2)
            | Expression::Less(cmp1, cmp2)
            | Expression::LessEq(cmp1, cmp2)
            | Expression::Greater(cmp1, cmp2)
            | Expression::GreaterEq(cmp1, cmp2) => {
                //todo remove current_ifs
                nums.ifs += 1;
                let current_ifs = nums.ifs;
                let cmp1_value = get_value(cmp1, tmp_vec, nums);
                let cmp2_value = get_value(cmp2, tmp_vec, nums);
                tmp_vec.push(Ir::JmpCmp(
                    cmp1_value,
                    cmp2_value,
                    format!("end_if_{}", current_ifs),
                    get_jump(cond),
                ));
                let seq_value = get_value(seq, tmp_vec, nums);
                tmp_vec.push(Ir::Label(format!("end_if_{}", current_ifs)));
                seq_value
            }
            _ => {
                dbg!(cond);
                panic!("Unrecognized boolean expression.")
            }
        },
        Expression::Loop(seq) => {
            tmp_vec.push(Ir::Label(format!("loop_{}", nums.loops)));
            let seq_value = get_value(seq, tmp_vec, nums);
            tmp_vec.push(Ir::Jmp(format!("loop_{}", nums.loops)));
            tmp_vec.push(Ir::Label(format!("loop_end_{}", nums.loops)));
            nums.loops += 1;
            seq_value
        }
        Expression::Break => {
            tmp_vec.push(Ir::Jmp(format!("loop_end_{}", nums.loops)));
            "".to_string()
        }
        Expression::Sequence(seq) => {
            for expr in &seq[0..seq.len() - 1] {
                get_value(expr, tmp_vec, nums);
            }
            get_value(&seq[seq.len() - 1], tmp_vec, nums)
        }
        _ => panic!("Invalid expression"),
    }
}

fn get_operation(operation: &Expression, arg1: String, arg2: String) -> Ir {
    match operation {
        Expression::Add(..) => Ir::Add(arg1, arg2),
        Expression::Sub(..) => Ir::Sub(arg1, arg2),
        Expression::Mul(..) => Ir::Mul(arg1, arg2),
        Expression::Div(..) => Ir::Div(arg1, arg2),
        Expression::Mod(..) => Ir::Mod(arg1, arg2),
        _ => panic!("Unexpected operation."),
    }
}

fn get_jump(expr: &Expression) -> JmpType {
    match expr {
        Expression::BoolLit(..) => JmpType::Je,
        Expression::Eq(..) => JmpType::Jne,
        Expression::NotEq(..) => JmpType::Je,
        Expression::Less(..) => JmpType::Jge,
        Expression::LessEq(..) => JmpType::Jg,
        Expression::Greater(..) => JmpType::Jle,
        Expression::GreaterEq(..) => JmpType::Jl,
        _ => {
            dbg!(expr);
            panic!("Given expression is not a boolean operator.")
        }
    }
}
