use crate::parser::*;

enum InterRepr {
    Exit(Value, Value),
    Add(Value, Value, Value),
    Sub(Value, Value, Value),
    Mul(Value, Value, Value),
    Div(Value, Value, Value),
    Rem(Value, Value, Value),
}

pub fn generate_ir(ast: Vec<Expression>) -> Vec<InterRepr> {
    let inter_repr = Vec::new();
    let num_tmp = 0;

    for expr in ast {
        inter_repr.push(match expr {
            Expression::Exit() => InterRepr::Exit((), ()),
        })
    }

    inter_repr
}
