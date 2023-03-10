use crate::eval::eval_program;
use std::io::prelude::*;
use std::io::BufRead;
use std::io::BufReader;

use crate::lexer::Lexer;
use crate::object::Environment;
use crate::parser::Parser;

const PROMPT: &str = ">> ";

pub fn start<R, W>(input: R, output: W)
where
    R: Read,
    W: Write,
{
    let mut reader = BufReader::new(input);
    let mut writer = output;
    let mut env = Environment::new();

    loop {
        write!(writer, "{}", PROMPT).unwrap();
        writer.flush().unwrap();

        let mut line = String::new();
        if reader.read_line(&mut line).is_err() {
            return;
        }

        let lexer = Lexer::new(&line);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        if parser.errors().len() > 0 {
            for err in parser.errors() {
                writeln!(writer, "\t{}", err).unwrap();
            }
            continue;
        }

        if let Some(evaluated) = eval_program(program, &mut env) {
            writeln!(writer, "{}", evaluated.to_string()).unwrap()
        }
    }
}
