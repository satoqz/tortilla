mod formatter;
mod lexer;
mod parser;

use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let tokens = lexer::lex(&input);

    // TODO: Handle errors once it makes sense.
    let paragraphs = parser::parse(&tokens).unwrap();

    let stdout = io::stdout().lock();
    let options = formatter::Options::default();
    formatter::format(stdout, &paragraphs, options)
}
