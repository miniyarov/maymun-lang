use crate::ast::{Expression, Program, Statement, Statements};
use crate::object::Object;

pub fn eval_program(program: Program) -> Option<Object> {
    let mut result = None;
    for stmt in program.all() {
        match stmt {
            Statement::Expression(expr) => {
                let eval = eval_expression(expr);
                if let Object::Return(o) = eval {
                    return Some(*o);
                }
                result = Some(eval)
            }
            Statement::Return(expr) => {
                return Some(eval_expression(expr));
            }
            _ => {}
        }
    }

    result
}

fn eval_block_statements(stmts: &Statements) -> Option<Object> {
    let mut result = None;
    for stmt in stmts {
        match stmt {
            Statement::Expression(expr) => {
                let eval = eval_expression(expr);
                if let Object::Return(o) = eval {
                    return Some(*o);
                }
                result = Some(eval)
            }
            Statement::Return(expr) => {
                return Some(Object::Return(Box::new(eval_expression(expr))));
            }
            _ => {}
        }
    }

    result
}

fn eval_expression(expr: &Expression) -> Object {
    match expr {
        Expression::Int(i) => Object::Integer(*i),
        Expression::Boolean(b) => Object::Boolean(*b),
        Expression::Prefix(op, right) => {
            let right = eval_expression(right);
            match op.as_str() {
                "!" => match right {
                    Object::Boolean(b) => Object::Boolean(!b),
                    Object::Integer(i) => Object::Boolean(i == 0),
                    _ => Object::Null,
                },
                "-" => {
                    if let Object::Integer(i) = right {
                        Object::Integer(-i)
                    } else {
                        Object::Null
                    }
                }
                _ => Object::Null,
            }
        }
        Expression::Infix(left, op, right) => {
            let left = eval_expression(left);
            let right = eval_expression(right);
            match (&left, &right) {
                (Object::Integer(li), Object::Integer(ri)) => match op.as_str() {
                    "+" => Object::Integer(li + ri),
                    "-" => Object::Integer(li - ri),
                    "*" => Object::Integer(li * ri),
                    "/" => Object::Integer(li / ri),
                    "<" => Object::Boolean(li < ri),
                    ">" => Object::Boolean(li > ri),
                    "==" => Object::Boolean(li == ri),
                    "!=" => Object::Boolean(li != ri),
                    _ => Object::Null,
                },
                _ => match op.as_str() {
                    "==" => Object::Boolean(left == right),
                    "!=" => Object::Boolean(left != right),
                    _ => Object::Null,
                },
            }
        }
        Expression::If(cond, conseq, alter) => {
            let cond = eval_expression(cond);
            match cond {
                Object::Boolean(b) => {
                    if b {
                        eval_block_statements(conseq).unwrap()
                    } else {
                        if let Some(alter) = alter {
                            eval_block_statements(alter).unwrap()
                        } else {
                            Object::Null
                        }
                    }
                }
                Object::Null => {
                    if let Some(alter) = alter {
                        eval_block_statements(alter).unwrap()
                    } else {
                        Object::Null
                    }
                }
                _ => eval_block_statements(conseq).unwrap(),
            }
        }
        _ => Object::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn test_eval_integer_expression() {
        let tests = vec![
            ("5", 5),
            ("10", 10),
            ("-5", -5),
            ("-10", -10),
            ("5 + 5 + 5 + 5 - 10", 10),
            ("2 * 2 * 2 * 2 * 2", 32),
            ("-50 + 100 + -50", 0),
            ("5 * 2 + 10", 20),
            ("5 + 2 * 10", 25),
            ("20 + 2 * -10", 0),
            ("50 / 2 * 2 + 10", 60),
            ("2 * (5 + 10)", 30),
            ("3 * 3 * 3 + 10", 37),
            ("3 * (3 * 3) + 10", 37),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", 50),
        ];

        for (input, expect) in tests {
            let eval = test_eval(input);
            assert!(matches!(eval, Object::Integer(i) if expect == i));
        }
    }

    #[test]
    fn test_eval_boolean_expression() {
        let tests = vec![
            ("true", true),
            ("false", false),
            ("1 < 2", true),
            ("1 > 2", false),
            ("1 < 1", false),
            ("1 > 1", false),
            ("1 == 1", true),
            ("1 != 1", false),
            ("1 == 2", false),
            ("1 != 2", true),
            ("true == true", true),
            ("false == false", true),
            ("true == false", false),
            ("true != false", true),
            ("false != true", true),
            ("(1 < 2) == true", true),
            ("(1 < 2) == false", false),
            ("(1 > 2) == true", false),
            ("(1 > 2) == false", true),
            ("!true", false),
            ("!false", true),
            ("!5", false),
            ("!!true", true),
            ("!!false", false),
            ("!!5", true),
        ];

        for (input, expect) in tests {
            let eval = test_eval(input);
            assert!(matches!(eval, Object::Boolean(b) if expect == b));
        }
    }

    #[test]
    fn test_if_else_expressions() {
        let tests = vec![
            ("if (true) { 10 }", Some(10)),
            ("if (false) { 10 }", None),
            ("if (1) { 10 }", Some(10)),
            ("if (1 < 2) { 10 }", Some(10)),
            ("if (1 > 2) { 10 }", None),
            ("if (1 > 2) { 10 } else { 20 }", Some(20)),
            ("if (1 < 2) { 10 } else { 20 }", Some(10)),
        ];

        for (input, expect) in tests {
            let eval = test_eval(input);
            match eval {
                Object::Integer(i) => assert_eq!(expect.unwrap(), i),
                Object::Null => assert_eq!(expect, None),
                _ => panic!("unexpected match eval_expression"),
            }
        }
    }

    #[test]
    fn test_return_statement() {
        let tests = vec![
            ("return 10;", 10),
            ("return 10; 9;", 10),
            ("return 2 * 5; 9;", 10),
            ("9; return 2 * 5; 9;", 10),
            (
                "if (10 > 1) { if (10 > 1) { return 10; } return 1; } else { return 11; }",
                10,
            ),
        ];

        for (input, expect) in tests {
            let eval = test_eval(input);
            assert!(matches!(eval, Object::Integer(i) if expect == i));
        }
    }

    fn test_eval(input: &str) -> Object {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        eval_program(parser.parse_program()).unwrap()
    }
}
