use crate::parser::Expression;

#[derive(Debug, PartialEq)]
pub enum Ir {
    Exit(String),
    Assign(String, String),
    Add(String, String),
    Sub(String, String),
    Mul(String, String),
    Div(String, String),
    Mod(String, String),
}

pub fn generate_ir(ast: Vec<Expression>) -> Vec<Ir> {
    let mut inter_repr = Vec::new();
    let mut tmp_vec = Vec::new();
    let mut num_tmp = 0;

    for expr in ast {
        get_value(&expr, &mut tmp_vec, &mut num_tmp);
        inter_repr.append(&mut tmp_vec);
        tmp_vec.clear();
    }

    inter_repr
}

fn get_value(expr: &Expression, tmp_vec: &mut Vec<Ir>, num_tmp: &mut u32) -> String {
    match expr {
        Expression::Exit(val) => {
            let arg = get_value(val, tmp_vec, num_tmp);
            tmp_vec.push(Ir::Exit(arg.clone()));
            arg
        }
        Expression::Assign(ref dest, ref src) | Expression::Declaration(ref dest, ref src) => {
            let arg1 = get_value(dest, tmp_vec, num_tmp);
            let arg2 = get_value(src, tmp_vec, num_tmp);
            tmp_vec.push(Ir::Assign(arg1.clone(), arg2));
            arg1
        }
        Expression::Add(ref dest, ref src)
        | Expression::Sub(ref dest, ref src)
        | Expression::Mul(ref dest, ref src)
        | Expression::Div(ref dest, ref src)
        | Expression::Mod(ref dest, ref src) => {
            let arg1 = get_value(dest, tmp_vec, num_tmp);
            let arg2 = get_value(src, tmp_vec, num_tmp);
            tmp_vec.push(get_operation(expr, arg1.clone(), arg2));
            arg1
        }
        Expression::IntLit(int) => {
            *num_tmp += 1;
            tmp_vec.push(Ir::Assign(format!("t{num_tmp}"), int.to_string()));
            format!("t{num_tmp}")
        }
        Expression::BoolLit(bool) => {
            *num_tmp += 1;
            tmp_vec.push(Ir::Assign(
                format!("t{num_tmp}"),
                if bool == "true" {
                    "1".to_string()
                } else {
                    "0".to_string()
                },
            ));
            format!("t{num_tmp}")
        }
        Expression::Identifier(id) => id.to_string(),
        Expression::If(..) => todo!(),
        Expression::Sequence(..) => todo!(),
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
