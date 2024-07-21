use yora_compiler::lexer::*;

#[test]
fn test_input() {
    let input = "exit(2 + 3);\n";

    let output = vec![
        Token::Exit,
        Token::OpenParen,
        Token::IntLit("2".to_string()),
        Token::Add,
        Token::IntLit("3".to_string()),
        Token::CloseParen,
        Token::SemiColon,
    ];

    assert_eq!(lex(input.to_string()), output);
}
