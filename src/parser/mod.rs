use crate::ast::{Expression, Program, Statement};
use crate::lexer::Lexer;
use crate::token::Token;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    cur_token: Token,
    peek_token: Token,
    errors: Vec<String>
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        let mut p = Self {
            lexer,
            cur_token: Token::Illegal,
            peek_token: Token::Illegal,
            errors: vec![]
        };

        p.next_token();
        p.next_token();
        p
    }

    pub fn parse_program(&mut self) -> Program {
        let mut program = Program::new();

        while self.cur_token != Token::Eof {
            if let Some(stmt) = self.parse_statement() {
                program.push(stmt);
            }
            self.next_token();
        }

        program
    }

    pub fn errors(&self) -> Vec<String> {
        return self.errors.clone();
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token {
            Token::Let => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            _ => None
        }
    }

    fn parse_let_statement(&mut self) -> Option<Statement> {
        if let Token::Ident(ident) = self.peek_token.clone() {
            self.next_token();

            if !self.expect_peek(Token::Assign) {
                return None;
            }

            while self.cur_token != Token::Semicolon {
                self.next_token();
            }

            return Some(Statement::Let(ident, Expression::Literal("".to_string())));
        }

        self.peek_error(Token::Ident("".to_string()));
        None
    }

    fn parse_return_statement(&mut self) -> Option<Statement> {
        self.next_token();

        while self.cur_token != Token::Semicolon {
            self.next_token();
        }

        return Some(Statement::Return(Expression::Literal("".to_string())))
    }

    fn expect_peek(&mut self, token: Token) -> bool {
        if self.peek_token == token {
            self.next_token();
            return true;
        }

        self.peek_error(token);
        return false;
    }

    fn peek_error(&mut self, token: Token) {
        self.errors.push(format!("expected next token to be {:?}, got {:?} instead", token, self.peek_token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_let_statements() {
        let input = "
let x = 5;
let y = 10;
let foobar = 934343;
";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();

        check_parser_errors(&parser);
        assert_eq!(3, program.len());

        let tests = vec![
            "x",
            "y",
            "foobar",
        ];

        for (i, expected_ident) in tests.iter().enumerate() {
            assert!(matches!(&program[i], Statement::Let(ident, _) if ident == *expected_ident));
        }
    }

    fn check_parser_errors(parser: &Parser) {
        if parser.errors.len() > 0 {
            for e in parser.errors.iter() {
                println!("parser error: {}", e);
            }
        }
        assert_eq!(0, parser.errors.len());
    }

    #[test]
    fn test_return_statement() {
        let input = "
return 5;
return 10;
return 909090;
";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();

        check_parser_errors(&parser);
        assert_eq!(3, program.len());

        for stmt in program {
            assert!(matches!(stmt, Statement::Return(_)))
        }
    }
}


























