use yora_compiler::lexer::Token;
use yora_compiler::parser::*;

#[test]
fn test_parser() {
    let input = vec![
        Token::Exit,
        Token::OpenParen,
        Token::IntLit("2".to_string()),
        Token::Add,
        Token::IntLit("3".to_string()),
        Token::CloseParen,
        Token::SemiColon,
    ];
    let output = vec![Expression::Sequence(vec![Expression::Exit(Box::new(
        Expression::Add(
            Box::new(Expression::IntLit("2".to_string())),
            Box::new(Expression::IntLit("3".to_string())),
        ),
    ))])];
    assert_eq!(parse(input), output);
}
