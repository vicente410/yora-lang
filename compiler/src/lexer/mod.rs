use std::process;

pub mod tokens_pretty;

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
    StringLit,
}

impl Token {
    pub fn new(buffer: &Buffer) -> Token {
        Token {
            str: buffer.str.clone(),
            kind: Token::get_token_kind(&buffer.str),
            line: buffer.y,
            col: buffer.x,
        }
    }

    fn get_token_kind(string: &String) -> TokenKind {
        if string.parse::<i64>().is_ok() {
            return TokenKind::IntLit;
        }

        if string[0..1] == *"\"" {
            return TokenKind::StringLit;
        }

        match string.as_str() {
            "var" | "if" | "else" | "loop" | "while" | "continue" | "break" => TokenKind::Keyword,

            "=" | "+" | "-" | "*" | "/" | "%" | "+=" | "-=" | "*=" | "/=" | "%=" | "!" | "and"
            | "or" | "==" | "!=" | "<" | "<=" | ">" | ">=" => TokenKind::Operator,

            ":" | "(" | ")" | "[" | "]" | "," => TokenKind::Separator,

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

pub struct Buffer {
    pub str: String,
    pub first_ch: char,
    pub x: usize,
    pub y: usize,
}

impl Buffer {
    fn new() -> Buffer {
        Buffer {
            str: String::new(),
            first_ch: '\0',
            x: 1,
            y: 1,
        }
    }

    fn push(&mut self, ch: char, cursor: &Cursor) {
        if self.str.is_empty() {
            self.first_ch = ch;
            self.x = cursor.x;
            self.y = cursor.y;
        }
        self.str.push(ch);
    }

    fn clear(&mut self) {
        self.str.clear();
    }

    fn should_tokenize(&self, ch: char) -> bool {
        !(self.str.is_empty()
            || (Buffer::is_id_or_num(self.first_ch) && Buffer::is_id_or_num(ch))
            || (Buffer::is_symbol(self.first_ch)
                && Buffer::is_symbol(ch)
                && ch != '['
                && ch != ']'
                && ch != '('
                && ch != ')'
                && ch != ','
                && ch != ':'))
    }

    fn is_symbol(ch: char) -> bool {
        "=+-*/%!<>:()[],".contains(ch)
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
    let mut chars = source.chars();

    while let Some(ch) = chars.next() {
        if buffer.str == "#" {
            for next_ch in chars.by_ref() {
                cursor.advance(next_ch);
                if next_ch == '\n' {
                    break;
                }
            }
            buffer.clear();
            continue;
        }
        if buffer.str == "\"" {
            buffer.push(ch, &cursor);
            for next_ch in chars.by_ref() {
                buffer.push(next_ch, &cursor);
                cursor.advance(next_ch);
                if next_ch == '\"' {
                    break;
                }
            }
            tokens.push(Token::new(&buffer));
            buffer.clear();
            continue;
        }
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
