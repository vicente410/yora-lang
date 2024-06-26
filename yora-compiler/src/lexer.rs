use std::process;

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

impl Token {
    fn new(kind: TokenKind, line: usize, column: usize) -> Token {
        Token { kind, line, column }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
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
    And,
    Or,
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
    let num_lines = source.lines().count();

    // Lex each line in the input
    for (i, line) in source.lines().enumerate() {
        current_indent = add_indent(&mut tokens, line, current_indent, i);
        add_tokens(&mut tokens, line, i);
        tokens.push(Token::new(TokenKind::NewLine, i, 0));
    }

    // Add dedent to finish at indentation level 0
    for i in 0..current_indent {
        tokens.push(Token::new(TokenKind::Dedent, num_lines + i as usize, 0));
    }

    tokens
}

fn add_indent(tokens: &mut Vec<Token>, line: &str, current_indent: u32, line_num: usize) -> u32 {
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
            tokens.push(Token::new(TokenKind::Indent, line_num, 0));
        }
    } else {
        for _ in 0..current_indent - line_indent {
            tokens.push(Token::new(TokenKind::Dedent, line_num, 0));
        }
    }

    line_indent
}

fn add_tokens(tokens: &mut Vec<Token>, line: &str, line_num: usize) {
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
            && get_token_kind(line[start..end].to_string()).is_ok()
            && get_token_kind(line[start..=end].to_string()).is_ok()
        {
            end += 1;
        }

        // Push token and skip if comment
        let token_kind = get_token_kind(line[start..end].to_string()).unwrap();
        match token_kind {
            TokenKind::Comment => break,
            _ => tokens.push(Token {
                kind: token_kind,
                line: line_num,
                column: start,
            }),
        }
        start = end;
    }
}

fn get_token_kind(string: String) -> Result<TokenKind, String> {
    if string.parse::<i64>().is_ok() {
        return Ok(TokenKind::IntLit(string));
    }

    Ok(match string.as_str() {
        "var" => TokenKind::Var,
        "if" => TokenKind::If,
        "loop" => TokenKind::Loop,
        "while" => TokenKind::While,
        "break" => TokenKind::Break,

        "exit" => TokenKind::Exit,
        "print" => TokenKind::Print,

        "+" => TokenKind::Add,
        "-" => TokenKind::Sub,
        "*" => TokenKind::Mul,
        "/" => TokenKind::Div,
        "%" => TokenKind::Mod,
        "!" => TokenKind::Not,
        "and" => TokenKind::And,
        "or" => TokenKind::Or,
        "==" => TokenKind::Eq,
        "!=" => TokenKind::NotEq,
        "<" => TokenKind::Less,
        "<=" => TokenKind::LessEq,
        ">" => TokenKind::Greater,
        ">=" => TokenKind::GreaterEq,

        ":" => TokenKind::Colon,
        "=" => TokenKind::Assign,
        "(" => TokenKind::OpenParen,
        ")" => TokenKind::CloseParen,

        "//" => TokenKind::Comment,

        "true" => TokenKind::BoolLit(string),
        "false" => TokenKind::BoolLit(string),
        _ => {
            if is_valid_identifier(&string) {
                TokenKind::Identifier(string)
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
