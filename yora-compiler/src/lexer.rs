use std::process;

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
    While,
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
    Indent,
    Dedent,
    NewLine,
    OpenParen,
    CloseParen,
    Colon,
    Comment,
}

pub fn lex(source: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut current_indent = 0;

    // Lex each line in the input
    for line in source.lines() {
        current_indent = add_indent(&mut tokens, line, current_indent);
        add_tokens(&mut tokens, line);
        tokens.push(Token::NewLine);
    }

    // Add dedent to finish at indentation level 0
    for _ in 0..current_indent {
        tokens.push(Token::Dedent);
    }

    tokens
}

fn add_indent(tokens: &mut Vec<Token>, line: &str, current_indent: u32) -> u32 {
    let line = line.as_bytes();
    let indent_size = 4;
    let mut line_indent = 0;
    let mut i = 0;

    if line.is_empty() {
        return current_indent;
    }

    while line[i].is_ascii_whitespace() {
        if line[i] == b'\t' {
            line_indent += 1;
            i += 1;
        } else if line[i..i + indent_size] == *b"    " {
            line_indent += 1;
            i += 4;
        } else {
            println!("Indentation error");
            process::exit(1);
        }
    }

    if line_indent > current_indent {
        for _ in 0..line_indent - current_indent {
            tokens.push(Token::Indent);
        }
    } else {
        for _ in 0..current_indent - line_indent {
            tokens.push(Token::Dedent);
        }
    }

    line_indent
}

fn add_tokens(tokens: &mut Vec<Token>, line: &str) {
    let mut start = 0;
    let mut end = 1;
    let len = line.len();

    while end < len {
        // Skip whitespaces before a token
        while start < len && line.as_bytes()[start].is_ascii_whitespace() {
            start += 1;
        }
        end = start + 1;

        // Try to create token until not possible
        while end < len
            && get_token(line[start..end].to_string()).is_ok()
            && get_token(line[start..=end].to_string()).is_ok()
        {
            end += 1;
        }

        // Push token and skip if comment
        let token = get_token(line[start..end].to_string()).unwrap();
        match token {
            Token::Comment => break,
            _ => tokens.push(token),
        }
        start = end;
    }
}

fn get_token(string: String) -> Result<Token, String> {
    if string.parse::<i64>().is_ok() {
        return Ok(Token::IntLit(string));
    }

    Ok(match string.as_str() {
        "var" => Token::Var,
        "if" => Token::If,
        "loop" => Token::Loop,
        "while" => Token::While,
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

        ":" => Token::Colon,
        "=" => Token::Assign,
        "(" => Token::OpenParen,
        ")" => Token::CloseParen,

        "//" => Token::Comment,

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
    string != "_" && !string.is_empty()
}
