pub type Identifier = String;

pub enum Expression {
    Literal(String),
}

pub enum Statement {
    Let(Identifier, Expression),
    Return(Expression),
}

pub type Statements = Vec<Statement>;

pub type Program = Statements;