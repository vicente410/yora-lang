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
        args: Vec<(String, PrimitiveType)>,
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
        name: Expression,
        value: Expression,
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
            StatementKind::Procedure { name, args, body } => {
                let mut string = String::new();

                string.push_str(&format!("{prefix}├── {name}\n"));
                if !args.is_empty() {
                    string.push_str(&format!("{prefix}├── args\n"));
                    for (i, (name, type_hint)) in args.iter().enumerate() {
                        if i < args.len() - 1 {
                            string.push_str(&format!("{prefix}│   ├── {name}: {type_hint}\n"));
                        } else {
                            string.push_str(&format!("{prefix}│   └── {name}: {type_hint}\n"));
                        }
                    }
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
            StatementKind::Call { name, args } => {
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
            StatementKind::Return { value } => {
                format!(
                    "return\n{prefix}└── {}",
                    value.format(&format!("{prefix}    "))
                )
            }
            StatementKind::Declare {
                name,
                type_hint,
                value,
            } => {
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
            StatementKind::Assign { name, value } => {
                format!(
                    "=\n{prefix}├── {}{prefix}└── {}",
                    name.format(&format!("{prefix}    ")),
                    value.format(&format!("{prefix}    "))
                )
            }
            StatementKind::If { cond, body } => {
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
            StatementKind::IfElse {
                cond,
                true_body,
                false_body,
            } => {
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
            StatementKind::Loop { body } => {
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
            StatementKind::While { cond, body } => {
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
            StatementKind::Continue => format!("{prefix}continue\n"),
            StatementKind::Break => format!("{prefix}break\n"),
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format(""))
    }
}
