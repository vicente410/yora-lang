use std::fmt;

use super::Expression;
use super::Token;
use crate::core::PrimitiveType;

#[derive(Debug, PartialEq, Clone)]
pub struct Statement {
    pub kind: StatementKind,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StatementKind {
    // Procedures
    Procedure {
        name: String,
        args: Vec<(String, Option<PrimitiveType>)>,
        ret: Option<PrimitiveType>,
        block: Vec<Statement>,
    },
    Call {
        name: String,
        args: Vec<Expression>,
    },
    Return {
        value: Expression,
    },

    // Variables
    Declare {
        name: String,
        type_hint: Option<PrimitiveType>,
        value: Option<Expression>,
    },
    Assign {
        dest: Expression,
        src: Expression,
    },

    // Control flow
    If {
        cond: Expression,
        block: Vec<Statement>,
    },
    IfElse {
        cond: Expression,
        true_block: Vec<Statement>,
        false_block: Vec<Statement>,
    },
    Loop {
        block: Vec<Statement>,
    },
    While {
        cond: Expression,
        block: Vec<Statement>,
    },
    Continue,
    Break,
}

impl Statement {
    pub fn new(kind: StatementKind, token: &Token) -> Statement {
        Statement {
            kind,
            line: token.line,
            col: token.col,
        }
    }

    fn format(&self, prefix: &str) -> String {
        match &self.kind {
            StatementKind::Procedure {
                name,
                args,
                ret,
                block,
            } => Self::format_procedure(prefix, name, args, ret, block),
            StatementKind::Call { name, args } => Self::format_call(prefix, name, args),
            StatementKind::Return { value } => Self::format_return(prefix, value),
            StatementKind::Declare {
                name,
                type_hint,
                value,
            } => Self::format_declare(prefix, name, type_hint, value),
            StatementKind::Assign { dest, src } => Self::format_assign(prefix, dest, src),
            StatementKind::If { cond, block } => Self::format_if(prefix, cond, block),
            StatementKind::IfElse {
                cond,
                true_block,
                false_block,
            } => Self::format_if_else(prefix, cond, true_block, false_block),
            StatementKind::Loop { block } => Self::format_loop(prefix, block),
            StatementKind::While { cond, block } => Self::format_while(prefix, cond, block),
            StatementKind::Continue => "continue\n".to_string(),
            StatementKind::Break => "break\n".to_string(),
        }
    }

    fn format_procedure(
        prefix: &str,
        name: &String,
        args: &[(String, Option<PrimitiveType>)],
        ret: &Option<PrimitiveType>,
        block: &[Statement],
    ) -> String {
        let mut string = String::new();

        string.push_str(&format!("{prefix}├── {name}\n"));
        if !args.is_empty() {
            string.push_str(&format!("{prefix}├── args\n"));
            for (i, (name, type_hint)) in args.iter().enumerate() {
                if i < args.len() - 1 {
                    if let Some(type_hint) = type_hint {
                        string.push_str(&format!("{prefix}│   ├── {name}: {type_hint}\n"));
                    } else {
                        string.push_str(&format!("{prefix}│   ├── {name}\n"));
                    }
                } else if let Some(type_hint) = type_hint {
                    string.push_str(&format!("{prefix}│   └── {name}: {type_hint}\n"));
                } else {
                    string.push_str(&format!("{prefix}│   └── {name}\n"));
                }
            }
        }

        if let Some(ret_type) = ret {
            string.push_str(&format!("{prefix}├── ret\n"));
            string.push_str(&format!("{prefix}│   └── {ret_type}\n"));
        }

        string.push_str(&format!("{prefix}└── block\n"));
        for (i, statement) in block.iter().enumerate() {
            if i < block.len() - 1 {
                string.push_str(&format!(
                    "{prefix}    ├── {}",
                    statement.format(&format!("{prefix}    │   "))
                ));
            } else {
                string.push_str(&format!(
                    "{prefix}    └── {}",
                    statement.format(&format!("{prefix}        "))
                ));
            }
        }

        format!("pr\n{string}")
    }

    fn format_call(prefix: &str, name: &String, args: &[Expression]) -> String {
        let mut string = String::new();

        for (i, arg) in args.iter().enumerate() {
            if i < args.len() - 1 {
                string.push_str(&format!(
                    "{prefix}├── {}",
                    arg.format(&format!("{prefix}│   "))
                ));
            } else {
                string.push_str(&format!(
                    "{prefix}└── {}",
                    arg.format(&format!("{prefix}    "))
                ));
            }
        }

        format!("{name}\n{string}")
    }

    fn format_return(prefix: &str, value: &Expression) -> String {
        format!(
            "return\n{prefix}└── {}",
            value.format(&format!("{prefix}    "))
        )
    }

    fn format_declare(
        prefix: &str,
        name: &String,
        type_hint: &Option<PrimitiveType>,
        value: &Option<Expression>,
    ) -> String {
        let mut string = String::new();

        if let Some(value) = value {
            if let Some(type_hint) = type_hint {
                string.push_str(&format!("{prefix}├── {name}: {type_hint}\n"));
            } else {
                string.push_str(&format!("{prefix}├── {name}\n"));
            }
            string.push_str(&format!(
                "{prefix}└── {}",
                value.format(&format!("{prefix}    "))
            ));
        } else if let Some(type_hint) = type_hint {
            string.push_str(&format!("{prefix}└── {name}: {type_hint}\n"));
        } else {
            string.push_str(&format!("{prefix}└── {name}\n"));
        }

        format!("var\n{string}")
    }

    fn format_assign(prefix: &str, dest: &Expression, src: &Expression) -> String {
        format!(
            "=\n{prefix}├── {}{prefix}└── {}",
            dest.format(&format!("{prefix}│   ")),
            src.format(&format!("{prefix}   "))
        )
    }

    fn format_if(prefix: &str, cond: &Expression, block: &[Statement]) -> String {
        let mut string = String::new();

        string.push_str(&format!(
            "{prefix}├── {}",
            cond.format(&format!("{prefix}│   "))
        ));

        string.push_str(&format!("{prefix}└── then\n"));
        for (i, statement) in block.iter().enumerate() {
            if i < block.len() - 1 {
                string.push_str(&format!(
                    "{prefix}    ├── {}",
                    statement.format(&format!("{prefix}    │   "))
                ));
            } else {
                string.push_str(&format!(
                    "{prefix}    └── {}",
                    statement.format(&format!("{prefix}        "))
                ));
            }
        }

        format!("if\n{string}")
    }

    fn format_if_else(
        prefix: &str,
        cond: &Expression,
        true_block: &[Statement],
        false_block: &[Statement],
    ) -> String {
        let mut string = String::new();

        string.push_str(&format!(
            "{prefix}├── {}",
            cond.format(&format!("{prefix}│   "))
        ));

        string.push_str(&format!("{prefix}├── then\n"));
        for (i, statement) in true_block.iter().enumerate() {
            if i < true_block.len() - 1 {
                string.push_str(&format!(
                    "{prefix}│   ├── {}",
                    statement.format(&format!("{prefix}│   │   "))
                ));
            } else {
                string.push_str(&format!(
                    "{prefix}│   └── {}",
                    statement.format(&format!("{prefix}│       "))
                ));
            }
        }

        string.push_str(&format!("{prefix}└── else\n"));
        for (i, statement) in false_block.iter().enumerate() {
            if i < false_block.len() - 1 {
                string.push_str(&format!(
                    "{prefix}    ├── {}",
                    statement.format(&format!("{prefix}    │   "))
                ));
            } else {
                string.push_str(&format!(
                    "{prefix}    └── {}",
                    statement.format(&format!("{prefix}        "))
                ));
            }
        }

        format!("if\n{string}")
    }

    fn format_loop(prefix: &str, block: &[Statement]) -> String {
        let mut string = String::new();

        for (i, statement) in block.iter().enumerate() {
            if i < block.len() - 1 {
                string.push_str(&format!(
                    "{prefix}├── {}",
                    statement.format(&format!("{prefix}│   "))
                ));
            } else {
                string.push_str(&format!(
                    "{prefix}└── {}",
                    statement.format(&format!("{prefix}    "))
                ));
            }
        }

        format!("loop\n{string}")
    }

    fn format_while(prefix: &str, cond: &Expression, block: &[Statement]) -> String {
        let mut string = String::new();

        string.push_str(&format!(
            "{prefix}├── {}",
            cond.format(&format!("{prefix}│   "))
        ));

        string.push_str(&format!("{prefix}└── then\n"));
        for (i, statement) in block.iter().enumerate() {
            if i < block.len() - 1 {
                string.push_str(&format!(
                    "{prefix}    ├── {}",
                    statement.format(&format!("{prefix}    │   "))
                ));
            } else {
                string.push_str(&format!(
                    "{prefix}    └── {}",
                    statement.format(&format!("{prefix}        "))
                ));
            }
        }

        format!("while\n{string}")
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format(""))
    }
}
