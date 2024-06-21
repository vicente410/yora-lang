#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Literals
    Identifier(String),
    BoolLit(String),
    IntLit(String),

    //Keywords
    Var,
    If,
    Loop,
    Break,

    // Built-in functions
    Exit,
    Print,

    // Operators
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Not,
    Eq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,

    // Other
    Assign,
    OpenCurly,
    CloseCurly,
    OpenParen,
    CloseParen,
    SemiColon,
    Comment,
    BlockComment,
}

pub fn lex(source: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut start = 0;
    let mut end = 0;
    let len = source.len();

    while end < len {
        // Skip whitespaces before a token
        while start + 1 < len && source.chars().nth(start).unwrap().is_whitespace() {
            start += 1;
        }
        if start + 1 == len {
            break;
        }
        end = start + 1;

        // Try to create token until not possible
        while get_token(source[start..end].to_string()).is_ok()
            && get_token(source[start..=end].to_string()).is_ok()
        {
            end += 1
        }

        // Skip comments or add token to tokens
        let token = get_token(source[start..end].to_string()).unwrap();
        match token {
            Token::Comment => {
                while end < len && source[end..=end] != *"\n" {
                    end += 1;
                }
            }
            Token::BlockComment => {
                while end + 1 < len && source[end..=end + 1] != *"*/" {
                    end += 1;
                }
                end += 2;
            }
            _ => tokens.push(token),
        }
        start = end;
    }

    tokens
}

fn get_token(string: String) -> Result<Token, String> {
    if string.contains(' ') {
        return Err(format!("Invalid token \"{}\"", string));
    }

    if string.parse::<i64>().is_ok() {
        return Ok(Token::IntLit(string));
    }

    Ok(match string.as_str() {
        "var" => Token::Var,
        "if" => Token::If,
        "loop" => Token::Loop,
        "break" => Token::Break,

        "exit" => Token::Exit,
        "print" => Token::Print,

        "+" => Token::Add,
        "-" => Token::Sub,
        "*" => Token::Mul,
        "/" => Token::Div,
        "%" => Token::Mod,
        "!" => Token::Not,
        "==" => Token::Eq,
        "!=" => Token::NotEq,
        "<" => Token::Less,
        "<=" => Token::LessEq,
        ">" => Token::Greater,
        ">=" => Token::GreaterEq,

        ";" => Token::SemiColon,
        "=" => Token::Assign,
        "(" => Token::OpenParen,
        ")" => Token::CloseParen,
        "{" => Token::OpenCurly,
        "}" => Token::CloseCurly,

        "//" => Token::Comment,
        "/*" => Token::BlockComment,

        "true" => Token::BoolLit(string),
        "false" => Token::BoolLit(string),
        _ => {
            if is_valid_identifier(&string) {
                Token::Identifier(string)
            } else {
                return Err(format!("Invalid identifier \"{}\"", string));
            }
        }
    })
}

fn is_valid_identifier(string: &String) -> bool {
    for char in string.chars() {
        if !char.is_alphabetic() && char != '_' {
            return false;
        }
    }
    string != "_"
}
