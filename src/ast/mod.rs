use std::fmt::{Display, Formatter};

pub type Identifier = String;
pub type Operator = String;

pub enum Expression {
    Literal(String),
    Int(i64),
    Prefix(Operator, Box<Expression>),
    Infix(Box<Expression>, Operator, Box<Expression>),
    Boolean(bool),
    If(Box<Expression>, BlockStatement, Option<BlockStatement>),
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Literal(literal) => write!(f, "{}", literal),
            Expression::Int(int) => write!(f, "{}", int),
            Expression::Boolean(val) => write!(f, "{}", val),
            Expression::Prefix(operator, right) => {
                write!(f, "({}{})", operator, right.to_string())
            }
            Expression::Infix(left, operator, right) => {
                write!(
                    f,
                    "({} {} {})",
                    left.to_string(),
                    operator,
                    right.to_string()
                )
            }
            Expression::If(expression, consequence, alternative) => {
                let mut s = format!("if {}", expression);
                for stmt in consequence {
                    s.push_str(&stmt.to_string())
                }

                if let Some(alternative) = alternative {
                    s.push_str("else ");
                    for stmt in alternative {
                        s.push_str(&stmt.to_string())
                    }
                }

                write!(f, "{}", s)
            }
        }
    }
}

pub enum Statement {
    Let(Identifier, Expression),
    Return(Expression),
    Expression(Expression),
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Let(i, e) => write!(f, "let {} = {};", i, e.to_string()),
            Statement::Return(e) => write!(f, "return {}", e),
            Statement::Expression(e) => write!(f, "{}", e),
        }
    }
}

pub type BlockStatement = Vec<Statement>;

pub type Statements = Vec<Statement>;

pub struct Program(Statements);

impl Program {
    pub fn new() -> Self {
        Self(Statements::new())
    }

    pub fn all(&self) -> &Statements {
        &self.0
    }

    pub fn get(&self, i: usize) -> &Statement {
        self.0.get(i).unwrap()
    }

    pub fn push(&mut self, s: Statement) {
        self.0.push(s);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut o = String::new();
        for s in self.0.iter() {
            o.push_str(&s.to_string());
        }

        write!(f, "{}", o)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string() {
        let program = Program(Statements::from([Statement::Let(
            Identifier::from("myVar"),
            Expression::Literal("anotherVar".to_string()),
        )]));

        assert_eq!(program.to_string(), "let myVar = anotherVar;");
    }
}
