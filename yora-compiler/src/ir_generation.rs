use std::ops::Deref;

use crate::parser::Expression;

#[derive(Debug, PartialEq)]
pub enum Ir {
    Exit(String),
    Assign(String, String),
    Label(String),
    Jmp(String, String, String, JmpType),
    Add(String, String),
    Sub(String, String),
    Mul(String, String),
    Div(String, String),
    Mod(String, String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum JmpType {
    Jmp,
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
}

pub fn generate_ir(ast: Vec<Expression>) -> Vec<Ir> {
    let mut inter_repr = Vec::new();
    let mut tmp_vec = Vec::new();
    let mut nums = Nums { tmp: 0, ifs: 0 };

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
        Expression::Assign(ref dest, ref src) | Expression::Declaration(ref dest, ref src) => {
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
                tmp_vec.push(Ir::Jmp(
                    value,
                    "1".to_string(),
                    format!("end_if{}", current_ifs),
                    get_inverse_jump(JmpType::Je),
                ));
                let seq_value = get_value(seq, tmp_vec, nums);
                tmp_vec.push(Ir::Label(format!("end_if{}", current_ifs)));
                seq_value
            }
            Expression::Less(cmp1, cmp2, jmp_type)
            | Expression::LessEquals(cmp1, cmp2, jmp_type)
            | Expression::More(cmp1, cmp2, jmp_type)
            | Expression::MoreEquals(cmp1, cmp2, jmp_type)
            | Expression::Equals(cmp1, cmp2, jmp_type)
            | Expression::NotEquals(cmp1, cmp2, jmp_type) => {
                nums.ifs += 1;
                let current_ifs = nums.ifs;
                let jmp = jmp_type;
                let cmp1_value = get_value(cmp1, tmp_vec, nums);
                let cmp2_value = get_value(cmp2, tmp_vec, nums);
                tmp_vec.push(Ir::Jmp(
                    cmp1_value,
                    cmp2_value,
                    format!("end_if{}", current_ifs),
                    get_inverse_jump(jmp.clone()),
                ));
                let seq_value = get_value(seq, tmp_vec, nums);
                tmp_vec.push(Ir::Label(format!("end_if{}", current_ifs)));
                seq_value
            }
            _ => panic!("Unrecognized boolean expression."),
        },
        Expression::Sequence(seq) => {
            for expr in seq {
                get_value(expr, tmp_vec, nums);
            }
            "seq".to_string()
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

fn get_inverse_jump(jmp_type: JmpType) -> JmpType {
    match jmp_type {
        JmpType::Jmp => JmpType::Jmp,
        JmpType::Je => JmpType::Jne,
        JmpType::Jne => JmpType::Je,
        JmpType::Jl => JmpType::Jge,
        JmpType::Jle => JmpType::Jg,
        JmpType::Jg => JmpType::Jle,
        JmpType::Jge => JmpType::Jl,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ir_generation() {
        let input = vec![Expression::Exit(Box::new(Expression::Add(
            Box::new(Expression::IntLit("2".to_string())),
            Box::new(Expression::IntLit("3".to_string())),
        )))];
        let output = vec![
            Ir::Assign("t1".to_string(), "2".to_string()),
            Ir::Assign("t2".to_string(), "3".to_string()),
            Ir::Add("t1".to_string(), "t2".to_string()),
            Ir::Exit("t1".to_string()),
        ];
        assert_eq!(generate_ir(input), output);
    }
}
