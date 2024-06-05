#[derive(Debug, PartialEq)]
pub enum Token {
    Exit,
    Integer(String),
    SemiColon,
    Identifier(String),
}

pub fn lex(source: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut buffer: String = String::new();
    let mut iter = source.chars().peekable();

    while let Some(c) = iter.next() {
        if c.is_alphanumeric() {
            buffer.push(c);
            if let Some(c) = iter.peek() {
                if !c.is_alphanumeric() {
                    tokens.push(get_token(buffer.clone()));
                    buffer.clear();
                }
            }
        } else if !c.is_whitespace() {
            tokens.push(get_token(c.to_string()));
        }
    }

    tokens
}

fn get_token(string: String) -> Token {
    if string.parse::<i64>().is_ok() {
        return Token::Integer(string);
    }

    match string.as_str() {
        "exit" => Token::Exit,
        ";" => Token::SemiColon,
        _ => Token::Identifier(string),
    }
}
