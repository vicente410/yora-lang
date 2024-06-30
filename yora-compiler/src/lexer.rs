use std::fmt;
use std::process;

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub str: String,
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Keyword,
    Operator,
    Separator,
    Identifier,

    //Literals
    IntLit,
    BoolLit,
}

impl Token {
    fn new(buffer: &Buffer) -> Token {
        Token {
            str: buffer.string.clone(),
            kind: Token::get_token_kind(&buffer.string),
            line: buffer.y,
            col: buffer.x,
        }
    }

    fn get_token_kind(string: &String) -> TokenKind {
        if string.parse::<i64>().is_ok() {
            return TokenKind::IntLit;
        }

        match string.as_str() {
            "var" | "if" | "loop" | "while" | "break" => TokenKind::Keyword,

            "=" | "+" | "-" | "*" | "/" | "%" | "+=" | "-=" | "*=" | "/=" | "!" | "and" | "or"
            | "==" | "!=" | "<" | "<=" | ">" | ">=" => TokenKind::Operator,

            ":" | "(" | ")" => TokenKind::Separator,

            "true" | "false" => TokenKind::BoolLit,

            _ => {
                if Token::is_valid_identifier(string) {
                    TokenKind::Identifier
                } else {
                    println!("Invalid identifier:\n{}", string);
                    process::exit(1);
                }
            }
        }
    }

    fn is_valid_identifier(string: &String) -> bool {
        for ch in string.chars() {
            if !ch.is_alphanumeric() && ch != '_' {
                return false;
            }
        }
        !string.is_empty() && string != "_"
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}\t{}\t\"{}\"",
            self.line, self.col, self.kind, self.str
        )
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TokenKind::Keyword => "key",
                TokenKind::Operator => "op",
                TokenKind::Separator => "sep",
                TokenKind::Identifier => "id",
                TokenKind::IntLit => "int",
                TokenKind::BoolLit => "bool",
            }
        )
    }
}

struct Buffer {
    string: String,
    first_ch: char,
    x: usize,
    y: usize,
}

impl Buffer {
    fn new() -> Buffer {
        Buffer {
            string: String::new(),
            first_ch: '\0',
            x: 1,
            y: 1,
        }
    }

    fn push(&mut self, ch: char, cursor: &Cursor) {
        if self.string.is_empty() {
            self.first_ch = ch;
            self.x = cursor.x;
            self.y = cursor.y;
        }
        self.string.push(ch);
    }

    fn clear(&mut self) {
        self.string.clear();
    }

    fn is_empty(&self) -> bool {
        self.string.is_empty()
    }

    fn should_tokenize(&self, ch: char) -> bool {
        !(self.is_empty()
            || (Buffer::is_id_or_num(self.first_ch) && Buffer::is_id_or_num(ch))
            || (Buffer::is_symbol(self.first_ch) && Buffer::is_symbol(ch)))
    }

    fn is_symbol(ch: char) -> bool {
        "=+-*/%!<>:()".contains(ch)
    }

    fn is_id_or_num(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_'
    }
}

struct Cursor {
    x: usize,
    y: usize,
}

impl Cursor {
    fn new() -> Cursor {
        Cursor { x: 1, y: 1 }
    }

    fn advance(&mut self, ch: char) {
        if ch == '\n' {
            self.x = 1;
            self.y += 1;
        } else if ch == '\t' {
            self.x += 4;
        } else {
            self.x += 1;
        }
    }
}

pub fn lex(source: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut buffer = Buffer::new();
    let mut cursor = Cursor::new();

    for ch in source.chars() {
        if buffer.should_tokenize(ch) {
            tokens.push(Token::new(&buffer));
            buffer.clear();
        }
        if !ch.is_whitespace() {
            buffer.push(ch, &cursor);
        }
        cursor.advance(ch);
    }

    tokens
}
