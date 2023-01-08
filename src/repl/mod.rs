use std::io::prelude::*;
use std::io::BufRead;
use std::io::BufReader;

use crate::lexer::Lexer;
use crate::token::Token;

const PROMPT: &str = ">> ";

pub fn start<R, W>(input: R, output: W)
where
    R: Read,
    W: Write,
{
    let mut reader = BufReader::new(input);
    let mut writer = output;

    loop {
        write!(writer, "{}", PROMPT).unwrap();
        writer.flush().unwrap();

        let mut line = String::new();
        if reader.read_line(&mut line).is_err() {
            return;
        }

        let mut lexer = Lexer::new(&line);
        loop {
            let token = lexer.next_token();
            if token == Token::Eof {
                break;
            }
            writeln!(writer, "{:?}", token).unwrap();
        }
    }
}
