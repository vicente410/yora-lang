#[derive(Debug, PartialEq)]
pub enum Token {
    Return,
    Integer(String),
    SemiColon,
    Identifier(String),
}

pub fn lex(source: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut buffer: String = String::new();

    for c in source.chars() {
        if c.is_alphanumeric() {
            buffer.push(c);
        } else {
            if !buffer.is_empty() {
                tokens.push(get_token(buffer.clone()));
                buffer.clear();
            }
            if c != ' ' && c != '\n' && c != '\t' {
                tokens.push(get_token(c.to_string()));
            }
        }
    }

    tokens
}

fn get_token(string: String) -> Token {
    if string.parse::<i64>().is_ok() {
        return Token::Integer(string);
    }

    match string.as_str() {
        "return" => Token::Return,
        ";" => Token::SemiColon,
        _ => Token::Identifier(string),
    }
}
