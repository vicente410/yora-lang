use std::fmt;

use crate::core::PrimitiveType;
use crate::expression::Expression;
use crate::Token;

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
        body: Vec<Statement>,
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
        body: Vec<Statement>,
    },
    IfElse {
        cond: Expression,
        true_body: Vec<Statement>,
        false_body: Vec<Statement>,
    },
    Loop {
        body: Vec<Statement>,
    },
    While {
        cond: Expression,
        body: Vec<Statement>,
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
                body,
            } => Self::format_procedure(prefix, name, args, ret, body),
            StatementKind::Call { name, args } => Self::format_call(prefix, name, args),
            StatementKind::Return { value } => Self::format_return(prefix, value),
            StatementKind::Declare {
                name,
                type_hint,
                value,
            } => Self::format_declare(prefix, name, type_hint, value),
            StatementKind::Assign { dest, src } => Self::format_assign(prefix, dest, src),
            StatementKind::If { cond, body } => Self::format_if(prefix, cond, body),
            StatementKind::IfElse {
                cond,
                true_body,
                false_body,
            } => Self::format_if_else(prefix, cond, true_body, false_body),
            StatementKind::Loop { body } => Self::format_loop(prefix, body),
            StatementKind::While { cond, body } => Self::format_while(prefix, cond, body),
            StatementKind::Continue => format!("continue\n"),
            StatementKind::Break => format!("break\n"),
        }
    }

    fn format_procedure(
        prefix: &str,
        name: &String,
        args: &Vec<(String, Option<PrimitiveType>)>,
        ret: &Option<PrimitiveType>,
        body: &Vec<Statement>,
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
                } else {
                    if let Some(type_hint) = type_hint {
                        string.push_str(&format!("{prefix}│   └── {name}: {type_hint}\n"));
                    } else {
                        string.push_str(&format!("{prefix}│   └── {name}\n"));
                    }
                }
            }
        }

        if let Some(ret_type) = ret {
            string.push_str(&format!("{prefix}├── ret\n"));
            string.push_str(&format!("{prefix}│   └── {ret_type}\n"));
        }

        string.push_str(&format!("{prefix}└── body\n"));
        for (i, statement) in body.iter().enumerate() {
            if i < body.len() - 1 {
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

    fn format_call(prefix: &str, name: &String, args: &Vec<Expression>) -> String {
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
        } else {
            if let Some(type_hint) = type_hint {
                string.push_str(&format!("{prefix}└── {name}: {type_hint}\n"));
            } else {
                string.push_str(&format!("{prefix}└── {name}\n"));
            }
        }

        format!("var\n{string}")
    }

    fn format_assign(prefix: &str, dest: &Expression, src: &Expression) -> String {
        format!(
            "=\n{prefix}├── {}{prefix}└── {}",
            dest.format(&format!("{prefix}    ")),
            src.format(&format!("{prefix}    "))
        )
    }

    fn format_if(prefix: &str, cond: &Expression, body: &Vec<Statement>) -> String {
        let mut string = String::new();

        string.push_str(&format!(
            "{prefix}├── {}",
            cond.format(&format!("{prefix}│   "))
        ));

        string.push_str(&format!("{prefix}└── then\n"));
        for (i, statement) in body.iter().enumerate() {
            if i < body.len() - 1 {
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
        true_body: &Vec<Statement>,
        false_body: &Vec<Statement>,
    ) -> String {
        let mut string = String::new();

        string.push_str(&format!(
            "{prefix}├── {}",
            cond.format(&format!("{prefix}│   "))
        ));

        string.push_str(&format!("{prefix}├── then\n"));
        for (i, statement) in true_body.iter().enumerate() {
            if i < true_body.len() - 1 {
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
        for (i, statement) in false_body.iter().enumerate() {
            if i < false_body.len() - 1 {
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

    fn format_loop(prefix: &str, body: &Vec<Statement>) -> String {
        let mut string = String::new();

        for (i, statement) in body.iter().enumerate() {
            if i < body.len() - 1 {
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

    fn format_while(prefix: &str, cond: &Expression, body: &Vec<Statement>) -> String {
        let mut string = String::new();

        string.push_str(&format!(
            "{prefix}├── {}",
            cond.format(&format!("{prefix}│   "))
        ));

        string.push_str(&format!("{prefix}└── then\n"));
        for (i, statement) in body.iter().enumerate() {
            if i < body.len() - 1 {
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
