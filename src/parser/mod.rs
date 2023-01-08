use crate::ast::{BlockStatement, Expression, Program, Statement};
use crate::lexer::Lexer;
use crate::token::Token;

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    Equals,      // ==
    LessGreater, // > or <
    Sum,         // +
    Product,     // *
    Prefix,      // -x or !x
    Call,        // my_function(x)
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    cur_token: Token,
    peek_token: Token,
    errors: Vec<String>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        let mut p = Self {
            lexer,
            cur_token: Token::Illegal,
            peek_token: Token::Illegal,
            errors: vec![],
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
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Option<Statement> {
        if let Token::Ident(ident) = self.peek_token.clone() {
            self.next_token();

            if !self.expect_peek(Token::Assign) {
                return None;
            }

            self.next_token();
            let expr = self.parse_expression(Precedence::Lowest).unwrap();

            while self.cur_token != Token::Semicolon {
                self.next_token();
            }

            return Some(Statement::Let(ident, expr));
        }

        self.peek_error(Token::Ident("".to_string()));
        None
    }

    fn parse_return_statement(&mut self) -> Option<Statement> {
        self.next_token();

        while self.cur_token != Token::Semicolon {
            self.next_token();
        }

        return Some(Statement::Return(Expression::Literal("".to_string())));
    }

    fn parse_expression_statement(&mut self) -> Option<Statement> {
        let expr = self.parse_expression(Precedence::Lowest);
        if let Some(expr) = expr {
            if self.peek_token == Token::Semicolon {
                self.next_token();
            }

            return Some(Statement::Expression(expr));
        }

        return None;
    }

    fn parse_expression(&mut self, pre: Precedence) -> Option<Expression> {
        let mut left_expr = match &self.cur_token {
            Token::Ident(ident) => Expression::Literal(ident.to_owned()),
            Token::Int(i) => Expression::Int(*i),
            Token::True | Token::False => Expression::Boolean(&self.cur_token == &Token::True),
            Token::Lparen => {
                self.next_token();

                let expr = self.parse_expression(Precedence::Lowest).unwrap();
                if !self.expect_peek(Token::Rparen) {
                    return None;
                }

                expr
            }
            Token::Bang | Token::Minus => {
                let op = (&self.cur_token).to_string();
                self.next_token();

                let expr = self.parse_expression(Precedence::Prefix).unwrap();
                Expression::Prefix(op, Box::new(expr))
            }
            Token::If => {
                if !self.expect_peek(Token::Lparen) {
                    return None;
                }

                self.next_token();
                let cond = self.parse_expression(Precedence::Lowest).unwrap();

                if !self.expect_peek(Token::Rparen) {
                    return None;
                }

                if !self.expect_peek(Token::Lbrace) {
                    return None;
                }

                let conseq = self.parse_block_statement();

                if self.peek_token == Token::Else {
                    self.next_token();

                    if !self.expect_peek(Token::Lbrace) {
                        return None;
                    }

                    Expression::If(Box::new(cond), conseq, Some(self.parse_block_statement()))
                } else {
                    Expression::If(Box::new(cond), conseq, None)
                }
            }
            _ => {
                self.errors.push(format!(
                    "undefined expression for {} found",
                    &self.cur_token.to_string()
                ));
                return None;
            }
        };

        while self.peek_token != Token::Semicolon && pre < self.precedence_for(&self.peek_token) {
            left_expr = match &self.peek_token {
                Token::Plus
                | Token::Minus
                | Token::Slash
                | Token::Asterisk
                | Token::Eq
                | Token::NotEq
                | Token::Lt
                | Token::Gt => {
                    self.next_token();

                    let op = (&self.cur_token).to_string();
                    let cur_pre = self.precedence_for(&self.cur_token);
                    self.next_token();

                    let expr = self.parse_expression(cur_pre).unwrap();
                    Expression::Infix(Box::new(left_expr), op, Box::new(expr))
                }
                _ => return Some(left_expr),
            }
        }

        return Some(left_expr);
    }

    fn parse_block_statement(&mut self) -> BlockStatement {
        let mut block_stmt = BlockStatement::new();
        self.next_token();

        while self.cur_token != Token::Rbrace && self.cur_token != Token::Eof {
            if let Some(stmt) = self.parse_statement() {
                block_stmt.push(stmt);
            }
            self.next_token();
        }

        block_stmt
    }

    fn expect_peek(&mut self, token: Token) -> bool {
        if self.peek_token == token {
            self.next_token();
            return true;
        }

        self.peek_error(token);
        return false;
    }

    fn precedence_for(&self, token: &Token) -> Precedence {
        match token {
            Token::Eq | Token::NotEq => Precedence::Equals,
            Token::Lt | Token::Gt => Precedence::LessGreater,
            Token::Plus | Token::Minus => Precedence::Sum,
            Token::Slash | Token::Asterisk => Precedence::Product,
            _ => Precedence::Lowest,
        }
    }

    fn peek_error(&mut self, token: Token) {
        self.errors.push(format!(
            "expected next token to be {:?}, got {:?} instead",
            token, self.peek_token
        ))
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

        let tests = vec!["x", "y", "foobar"];

        for (i, expected_ident) in tests.iter().enumerate() {
            assert!(
                matches!(&program.get(i), Statement::Let(ident, _) if ident == *expected_ident)
            );
        }
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

        for stmt in program.all() {
            assert!(matches!(stmt, Statement::Return(_)))
        }
    }

    #[test]
    fn test_identifier_expression() {
        let input = "foobar;";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();

        check_parser_errors(&parser);
        assert_eq!(1, program.len());

        let stmt = program.get(0);

        assert_eq!("foobar", stmt.to_string());
    }

    #[test]
    fn test_integer_expression() {
        let input = "5;";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();

        check_parser_errors(&parser);
        assert_eq!(1, program.len());

        let stmt = program.get(0);

        assert_eq!("5", stmt.to_string());
    }

    #[test]
    fn test_boolean_expression() {
        let input = "
true;
false;
let foobar = true; let barfoo = false;";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();

        check_parser_errors(&parser);
        assert_eq!(4, program.len());
        assert_eq!("true", program.get(0).to_string());
        assert_eq!("false", program.get(1).to_string());
        assert_eq!("let foobar = true;", program.get(2).to_string());
        assert_eq!("let barfoo = false;", program.get(3).to_string());
    }

    #[test]
    fn test_parse_prefix_expressions() {
        let tests = vec![("!5", "!", 5), ("-15", "-", 15)];

        for (input, operator, right) in tests {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);

            let program = parser.parse_program();

            check_parser_errors(&parser);
            assert_eq!(1, program.len());

            let stmt = program.get(0);

            match stmt {
                Statement::Expression(expr) => match expr {
                    Expression::Prefix(op, right_expr) => {
                        assert_eq!(operator, op);
                        match **right_expr {
                            Expression::Int(int) => {
                                assert_eq!(right, int);
                            }
                            _ => panic!("right expression cannot match"),
                        }
                    }
                    _ => panic!("expression cannot match: {}", expr.to_string()),
                },
                _ => panic!("statement cannot match"),
            }
        }
    }

    #[test]
    fn test_parse_infix_expressions() {
        let tests = vec![
            ("5 + 5;", 5, "+", 5),
            ("5 - 5;", 5, "-", 5),
            ("5 * 5;", 5, "*", 5),
            ("5 / 5;", 5, "/", 5),
            ("5 > 5;", 5, ">", 5),
            ("5 < 5;", 5, "<", 5),
            ("5 == 5;", 5, "==", 5),
            ("5 != 5;", 5, "!=", 5),
        ];

        for (input, left, operator, right) in tests {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);

            let program = parser.parse_program();

            check_parser_errors(&parser);
            assert_eq!(1, program.len());

            let stmt = program.get(0);

            match stmt {
                Statement::Expression(expr) => match expr {
                    Expression::Infix(left_expr, op, right_expr) => {
                        assert_eq!(operator, op);
                        match **left_expr {
                            Expression::Int(int) => {
                                assert_eq!(left, int);
                            }
                            _ => panic!("left expression cannot match"),
                        }
                        match **right_expr {
                            Expression::Int(int) => {
                                assert_eq!(right, int);
                            }
                            _ => panic!("right expression cannot match"),
                        }
                    }
                    _ => panic!("expression cannot match: {}", expr.to_string()),
                },
                _ => panic!("statement cannot match"),
            }
        }
    }

    #[test]
    fn test_operator_precedence_parsing() {
        let tests = vec![
            ("true", "true"),
            ("false", "false"),
            ("3 > 5 == false", "((3 > 5) == false)"),
            ("3 < 5 == true", "((3 < 5) == true)"),
            ("-a * b", "((-a) * b)"),
            ("!-a", "(!(-a))"),
            ("a + b + c", "((a + b) + c)"),
            ("a + b - c", "((a + b) - c)"),
            ("a * b * c", "((a * b) * c)"),
            ("a * b / c", "((a * b) / c)"),
            ("a + b / c", "(a + (b / c))"),
            ("a + b * c + d / e - f", "(((a + (b * c)) + (d / e)) - f)"),
            ("3 + 4; -5 * 5", "(3 + 4)((-5) * 5)"),
            ("5 > 4 == 3 < 4", "((5 > 4) == (3 < 4))"),
            ("5 < 4 != 3 > 4", "((5 < 4) != (3 > 4))"),
            (
                "3 + 4 * 5 == 3 * 1 + 4 * 5",
                "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
            ),
            (
                "3 + 4 * 5 == 3 * 1 + 4 * 5",
                "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
            ),
            ("1 + (2 + 3) + 4", "((1 + (2 + 3)) + 4)"),
            ("(5 + 5) * 2", "((5 + 5) * 2)"),
            ("2 / (5 + 5)", "(2 / (5 + 5))"),
            ("-(5 + 5)", "(-(5 + 5))"),
            ("!(true == true)", "(!(true == true))"),
        ];

        for (input, expected) in tests {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);

            let program = parser.parse_program();

            check_parser_errors(&parser);

            assert_eq!(program.to_string(), expected)
        }
    }

    #[test]
    fn test_if_expression() {
        let input = "if (x < y) { x }";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();

        check_parser_errors(&parser);

        assert_eq!(1, program.len());

        match program.get(0) {
            Statement::Expression(expr) => match expr {
                Expression::If(_, conseq, _) => {
                    match conseq.get(0).unwrap() {
                        Statement::Expression(expr) => {
                            assert_eq!("x", expr.to_string())
                        }
                        _ => panic!("cannot match consequence expression"),
                    }
                },
                _ => panic!("cannot match expression"),
            },
            _ => panic!("cannot match statement"),
        }
    }

    #[test]
    fn test_if_else_expression() {
        let input = "if (x < y) { x } else { y }";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();

        check_parser_errors(&parser);

        assert_eq!(1, program.len());

        match program.get(0) {
            Statement::Expression(expr) => match expr {
                Expression::If(_, conseq, alter) => {
                    match conseq.get(0).unwrap() {
                        Statement::Expression(expr) => {
                            assert_eq!("x", expr.to_string());
                        }
                        _ => panic!("cannot match consequence expression"),
                    }

                    if let Some(alter) = alter {
                        match alter.get(0).unwrap() {
                            Statement::Expression(expr) => {
                                assert_eq!("y", expr.to_string());
                            }
                            _ => panic!("cannot match alternative expression"),
                        }
                    } else {
                        panic!("cannot match alternative expression");
                    }
                }
                _ => panic!("cannot match expression"),
            },
            _ => panic!("cannot match statement"),
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
}
