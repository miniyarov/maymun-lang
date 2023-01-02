use std::io;

use maymun_lang::repl;

fn main() -> io::Result<()> {
    println!("Hello! This is the Maymun programming language!");
    println!("Feel free to type in commands");

    repl::start(io::stdin(), io::stdout());
    Ok(())
}
