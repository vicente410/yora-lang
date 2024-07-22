use std::fmt;
use std::ops::Deref;

pub fn get_syscall_num(syscall: String) -> usize {
    match syscall.as_str() {
        "read" => 0,
        "write" => 1,
        "open" => 2,
        "close" => 3,
        "mmap" => 9,
        "exit" => 60,
        _ => panic!("Invalid syscall"),
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum PrimitiveType {
    Bool,
    Int,
    Char,
    Arr(Box<PrimitiveType>),
}

impl PrimitiveType {
    pub fn from_str(type_str: &str) -> PrimitiveType {
        let mut type_copy = type_str;
        let mut array_count = 0;

        while type_copy[type_copy.len() - 2..] == *"[]" {
            array_count += 1;
            type_copy = &type_copy[0..type_copy.len() - 2];
        }

        let mut r#type = match type_copy {
            "Bool" => PrimitiveType::Bool,
            "Char" => PrimitiveType::Char,
            "Int" => PrimitiveType::Int,
            _ => panic!("Invalid type '{type_str}'"),
        };

        for _ in 0..array_count {
            r#type = PrimitiveType::Arr(Box::new(r#type));
        }

        r#type
    }

    pub fn as_string(&self) -> String {
        match self {
            PrimitiveType::Bool => "Bool",
            PrimitiveType::Char => "Char",
            PrimitiveType::Int => "Int",
            PrimitiveType::Arr(r#type) => return format!("{}[]", r#type.deref().as_string()),
        }
        .to_string()
    }

    pub fn get_size(&self) -> usize {
        match self {
            PrimitiveType::Bool => 1,
            PrimitiveType::Char => 1,
            PrimitiveType::Int => 8,
            PrimitiveType::Arr(..) => 8,
        }
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PrimitiveType::Bool => "Bool",
                PrimitiveType::Int => "Int",
                PrimitiveType::Char => "Char",
                PrimitiveType::Arr(r#type) => return write!(f, "{}[]", r#type.deref()),
            }
        )
    }
}

pub fn is_valid_type(type_str: String) -> bool {
    matches!(type_str.as_str(), "Bool" | "Int" | "Byte")
}
