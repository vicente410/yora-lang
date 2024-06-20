#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Exit,
    Print,
    Integer(String),
    SemiColon,
    Identifier(String),
    Equal,
    Var,
    LeftParen,
    RightParen,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

pub fn lex(source: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut buffer: String = String::new();
    let chars: Vec<char> = source.chars().collect();
    let len = source.len();
    let mut i = 0;

    while i < len {
        if chars[i].is_alphanumeric() {
            buffer.push(chars[i]);
            if i + 1 < len && !chars[i + 1].is_alphanumeric() {
                tokens.push(get_token(buffer.clone()));
                buffer.clear();
            }
        // Ignores comments
        } else if i + 1 < len && source[i..=i + 1] == *"//" {
            while i < len && chars[i] != '\n' {
                i += 1;
            }
        } else if i + 1 < len && source[i..=i + 1] == *"/*" {
            while i + 1 < len && source[i..=i + 1] != *"*/" {
                // println!("a a a a stayin' alive");
                i += 1;
            }
            i += 1;
        } else if !chars[i].is_whitespace() {
            tokens.push(get_token(chars[i].to_string()));
        }

        i += 1;
    }

    tokens
}

fn get_token(string: String) -> Token {
    if string.parse::<i64>().is_ok() {
        return Token::Integer(string);
    }

    match string.as_str() {
        "exit" => Token::Exit,
        "print" => Token::Print,
        "var" => Token::Var,
        ";" => Token::SemiColon,
        "=" => Token::Equal,
        "(" => Token::LeftParen,
        ")" => Token::RightParen,
        "+" => Token::Add,
        "-" => Token::Sub,
        "*" => Token::Mul,
        "/" => Token::Div,
        "%" => Token::Mod,
        _ => Token::Identifier(string),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input() {
        let input = "exit(2 + 3);";

        let output = vec![
            Token::Exit,
            Token::LeftParen,
            Token::Integer("2".to_string()),
            Token::Add,
            Token::Integer("3".to_string()),
            Token::RightParen,
            Token::SemiColon,
        ];

        assert_eq!(lex(input.to_string()), output);
    }
}
