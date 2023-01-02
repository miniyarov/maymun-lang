#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
    Illegal,
    Eof,

    // Identifiers + literals
    Ident(String), // add, foobar, x, y, ...
    Int(i64),      // 1234

    // Operators
    Assign,   // =
    Plus,     // +
    Minus,    // -
    Bang,     // !
    Asterisk, // *
    Slash,    // /

    // Comparisons
    Lt,     // <
    Gt,     // >
    Eq,     // ==
    NotEq, // !=

    // Delimiters
    Comma,     // ,
    Semicolon, // ;

    // Scopes
    Lparen, // (
    Rparen, // )
    Lbrace, // {
    Rbrace, // }

    // Keywords (reserved)
    Function, // fn
    Let,      // let
    True,     // true
    False,    // false
    If,       // if
    Else,     // else
    Return,   // return
}

pub fn lookup_ident(ident: &str) -> Token {
    match ident {
        "fn" => Token::Function,
        "let" => Token::Let,
        "true" => Token::True,
        "false" => Token::False,
        "if" => Token::If,
        "else" => Token::Else,
        "return" => Token::Return,
        _ => Token::Ident(ident.to_string()),
    }
}
