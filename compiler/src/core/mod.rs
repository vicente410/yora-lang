pub fn get_syscall(syscall: String) -> usize {
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

#[derive(Clone, PartialEq)]
pub enum PrimitiveType {
    Bool,
    Ptr,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
}

pub fn is_valid_type(type_str: String) -> bool {
    matches!(
        type_str.as_str(),
        "bool" | "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64"
    )
}

pub fn get_type(type_str: String) -> PrimitiveType {
    let mut type_copy = type_str.clone();
    if type_copy.contains("[]") {
        type_copy.pop();
        type_copy.pop();
    }

    match type_copy.as_str() {
        "bool" => PrimitiveType::Bool,
        "i8" => PrimitiveType::I8,
        "i16" => PrimitiveType::I16,
        "i32" => PrimitiveType::I32,
        "i64" => PrimitiveType::I64,
        "u8" => PrimitiveType::U8,
        "u16" => PrimitiveType::U16,
        "u32" => PrimitiveType::U32,
        "u64" => PrimitiveType::U64,
        _ => panic!("Invalid type"),
    }
}
impl PrimitiveType {
    pub fn as_string(&self) -> String {
        match self {
            PrimitiveType::Bool => "bool",
            PrimitiveType::I8 => "i8",
            PrimitiveType::I16 => "i16",
            PrimitiveType::I32 => "i32",
            PrimitiveType::I64 => "i64",
            PrimitiveType::U8 => "u8",
            PrimitiveType::U16 => "u16",
            PrimitiveType::U32 => "u32",
            PrimitiveType::U64 => "u64",
            PrimitiveType::Ptr => "ptr",
        }
        .to_string()
    }

    pub fn get_size(&self) -> usize {
        match self {
            PrimitiveType::Bool => 1,
            PrimitiveType::I8 => 1,
            PrimitiveType::I16 => 2,
            PrimitiveType::I32 => 4,
            PrimitiveType::I64 => 8,
            PrimitiveType::U8 => 1,
            PrimitiveType::U16 => 2,
            PrimitiveType::U32 => 4,
            PrimitiveType::U64 => 8,
            PrimitiveType::Ptr => 8,
        }
    }
}
