use crate::ast::{Expression, Program, Statement, Statements};
use crate::object::{Environment, Object};

pub fn eval_program(program: Program, env: &mut Environment) -> Option<Object> {
    let mut result = None;
    for stmt in program.all() {
        match stmt {
            Statement::Expression(expr) => {
                let eval = eval_expression(expr, env);

                match eval {
                    Object::Return(o) => return Some(*o),
                    Object::Error(msg) => return Some(Object::Error(msg)),
                    _ => {}
                }

                result = Some(eval)
            }
            Statement::Let(ident, expr) => {
                let eval = eval_expression(expr, env);
                if let Object::Error(msg) = eval {
                    return Some(Object::Error(msg));
                }

                env.insert(ident.to_string(), eval);
                result = None
            }
            Statement::Return(expr) => {
                return Some(eval_expression(expr, env));
            }
        }
    }

    result
}

fn eval_block_statements(stmts: &Statements, env: &mut Environment) -> Option<Object> {
    let mut result = None;
    for stmt in stmts {
        match stmt {
            Statement::Expression(expr) => {
                let eval = eval_expression(expr, env);

                match eval {
                    Object::Return(o) => return Some(*o),
                    Object::Error(msg) => return Some(Object::Error(msg)),
                    _ => {}
                }

                result = Some(eval)
            }
            Statement::Return(expr) => {
                let eval = eval_expression(expr, env);

                if let Object::Error(msg) = eval {
                    return Some(Object::Error(msg));
                }

                return Some(Object::Return(Box::new(eval)));
            }
            _ => {}
        }
    }

    result
}

fn eval_expression(expr: &Expression, env: &mut Environment) -> Object {
    match expr {
        Expression::Int(i) => Object::Integer(*i),
        Expression::Boolean(b) => Object::Boolean(*b),
        Expression::Literal(l) => {
            if let Some(o) = env.get(l) {
                return (*o).clone();
            }

            return Object::Error("identifier not found: ".to_string() + l);
        }
        Expression::Prefix(op, right) => {
            let right = eval_expression(right, env);

            if let Object::Error(msg) = right {
                return Object::Error(msg);
            }

            match op.as_str() {
                "!" => match right {
                    Object::Boolean(b) => Object::Boolean(!b),
                    Object::Integer(i) => Object::Boolean(i == 0),
                    _ => Object::Error(format!("unknown prefix type: {}", right.to_string())),
                },
                "-" => {
                    if let Object::Integer(i) = right {
                        Object::Integer(-i)
                    } else {
                        Object::Error(format!("unknown operator: -{}", right.to_string()))
                    }
                }
                _ => Object::Error(format!("unknown operator: {}{}", op, right.to_string())),
            }
        }
        Expression::Infix(left, op, right) => {
            let left = eval_expression(left, env);
            if let Object::Error(msg) = left {
                return Object::Error(msg);
            }

            let right = eval_expression(right, env);
            if let Object::Error(msg) = right {
                return Object::Error(msg);
            }

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
                    _ => Object::Error(format!(
                        "unknown operator: {} {} {}",
                        left.to_string(),
                        op,
                        right.to_string()
                    )),
                },
                _ => match op.as_str() {
                    "==" => Object::Boolean(left == right),
                    "!=" => Object::Boolean(left != right),
                    _ => Object::Error(format!(
                        "mismatch expression operation: {} {} {}",
                        left.to_string(),
                        op,
                        right.to_string()
                    )),
                },
            }
        }
        Expression::If(cond, conseq, alter) => {
            let cond = eval_expression(cond, env);
            if let Object::Error(msg) = cond {
                return Object::Error(msg);
            }

            match cond {
                Object::Boolean(b) => {
                    if b {
                        eval_block_statements(conseq, env).unwrap()
                    } else {
                        if let Some(alter) = alter {
                            eval_block_statements(alter, env).unwrap()
                        } else {
                            Object::Null
                        }
                    }
                }
                Object::Null => {
                    if let Some(alter) = alter {
                        eval_block_statements(alter, env).unwrap()
                    } else {
                        Object::Null
                    }
                }
                _ => eval_block_statements(conseq, env).unwrap(),
            }
        }
        _ => Object::Null,
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    use super::*;

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

    #[test]
    fn test_error_handling() {
        let tests = vec![
            (
                "5 + true;",
                "mismatch expression operation: Integer(5) + Boolean(true)",
            ),
            (
                "5 + true; 5;",
                "mismatch expression operation: Integer(5) + Boolean(true)",
            ),
            ("-true", "unknown operator: -Boolean(true)"),
            (
                "true + false;",
                "mismatch expression operation: Boolean(true) + Boolean(false)",
            ),
            (
                "5; true + false; 5",
                "mismatch expression operation: Boolean(true) + Boolean(false)",
            ),
            (
                "if (10 > 1) { true + false; }",
                "mismatch expression operation: Boolean(true) + Boolean(false)",
            ),
            (
                "if (10 > 1) { if (10 > 1) { return true + false; } return 1; }",
                "mismatch expression operation: Boolean(true) + Boolean(false)",
            ),
            ("foobar", "identifier not found: foobar"),
        ];

        for (input, expect) in tests {
            let eval = test_eval(input);
            match eval {
                Object::Error(msg) => {
                    assert_eq!(expect, msg)
                }
                _ => panic!("unexpected eval object {}", eval),
            }
        }
    }

    #[test]
    fn test_let_statements() {
        let tests = vec![
            ("let a = 5; a;", 5),
            ("let a = 5 * 5; a;", 25),
            ("let a = 5; let b = a; b;", 5),
            ("let a = 5; let b = a; let c = a + b + 5; c;", 15),
        ];

        for (input, expected) in tests {
            let eval = test_eval(input);
            assert!(matches!(eval, Object::Integer(i) if expected == i));
        }
    }

    fn test_eval(input: &str) -> Object {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let mut env = Environment::new();

        eval_program(parser.parse_program(), &mut env).unwrap()
    }
}
