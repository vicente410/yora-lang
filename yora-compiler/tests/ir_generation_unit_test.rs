use yora_compiler::ir_generation::{generate_ir, Ir, JmpType};
use yora_compiler::parser::Expression;

#[test]
fn test_ir_generation_bool_literals() {
    let input = vec![Expression::Sequence(vec![
        Expression::Declare(
            Box::new(Expression::Identifier("a".to_string())),
            Box::new(Expression::IntLit("2".to_string())),
        ),
        Expression::If(
            Box::new(Expression::BoolLit("true".to_string())),
            Box::new(Expression::Sequence(vec![
                Expression::Declare(
                    Box::new(Expression::Identifier("b".to_string())),
                    Box::new(Expression::IntLit("3".to_string())),
                ),
                Expression::If(
                    Box::new(Expression::BoolLit("false".to_string())),
                    Box::new(Expression::Sequence(vec![Expression::Exit(Box::new(
                        Expression::Identifier("b".to_string()),
                    ))])),
                ),
                Expression::Assign(
                    Box::new(Expression::Identifier("a".to_string())),
                    Box::new(Expression::Add(
                        Box::new(Expression::Identifier("a".to_string())),
                        Box::new(Expression::Identifier("b".to_string())),
                    )),
                ),
            ])),
        ),
        Expression::Exit(Box::new(Expression::Identifier("a".to_string()))),
    ])];

    let output = vec![
        Ir::Assign("t1".to_string(), "2".to_string()),
        Ir::Assign("a".to_string(), "t1".to_string()),
        Ir::Assign("t2".to_string(), "1".to_string()),
        Ir::Jmp(
            "t2".to_string(),
            "0".to_string(),
            "end_if1".to_string(),
            JmpType::Je,
        ),
        Ir::Assign("t3".to_string(), "3".to_string()),
        Ir::Assign("b".to_string(), "t3".to_string()),
        Ir::Assign("t4".to_string(), "0".to_string()),
        Ir::Jmp(
            "t4".to_string(),
            "0".to_string(),
            "end_if2".to_string(),
            JmpType::Je,
        ),
        Ir::Exit("b".to_string()),
        Ir::Label("end_if2".to_string()),
        Ir::Add("a".to_string(), "b".to_string()),
        Ir::Assign("a".to_string(), "a".to_string()),
        Ir::Label("end_if1".to_string()),
        Ir::Exit("a".to_string()),
    ];

    assert_eq!(generate_ir(input), output);
}

#[test]
fn test_ir_generation_greater() {
    let input = vec![Expression::Sequence(vec![
        Expression::Declare(
            Box::new(Expression::Identifier("a".to_string())),
            Box::new(Expression::IntLit("2".to_string())),
        ),
        Expression::If(
            Box::new(Expression::Greater(
                Box::new(Expression::Identifier("a".to_string())),
                Box::new(Expression::IntLit("1".to_string())),
            )),
            Box::new(Expression::Sequence(vec![
                Expression::Declare(
                    Box::new(Expression::Identifier("b".to_string())),
                    Box::new(Expression::IntLit("3".to_string())),
                ),
                Expression::If(
                    Box::new(Expression::Greater(
                        Box::new(Expression::Identifier("a".to_string())),
                        Box::new(Expression::IntLit("b".to_string())),
                    )),
                    Box::new(Expression::Sequence(vec![Expression::Exit(Box::new(
                        Expression::Identifier("b".to_string()),
                    ))])),
                ),
                Expression::Assign(
                    Box::new(Expression::Identifier("a".to_string())),
                    Box::new(Expression::Add(
                        Box::new(Expression::Identifier("a".to_string())),
                        Box::new(Expression::Identifier("b".to_string())),
                    )),
                ),
            ])),
        ),
        Expression::Exit(Box::new(Expression::Identifier("a".to_string()))),
    ])];

    let output = vec![
        Ir::Assign("t1".to_string(), "2".to_string()),
        Ir::Assign("a".to_string(), "t1".to_string()),
        Ir::Assign("t2".to_string(), "1".to_string()),
        Ir::Jmp(
            "a".to_string(),
            "t2".to_string(),
            "end_if1".to_string(),
            JmpType::Jle,
        ),
        Ir::Assign("t3".to_string(), "3".to_string()),
        Ir::Assign("b".to_string(), "t3".to_string()),
        Ir::Assign("t4".to_string(), "b".to_string()),
        Ir::Jmp(
            "a".to_string(),
            "t4".to_string(),
            "end_if2".to_string(),
            JmpType::Jle,
        ),
        Ir::Exit("b".to_string()),
        Ir::Label("end_if2".to_string()),
        Ir::Add("a".to_string(), "b".to_string()),
        Ir::Assign("a".to_string(), "a".to_string()),
        Ir::Label("end_if1".to_string()),
        Ir::Exit("a".to_string()),
    ];

    assert_eq!(generate_ir(input), output);
}
