mod lex;

use std::io::{self, Read};

// const DEFAULT_LINE_WIDTH: usize = 80;
// const DEFAULT_TAB_WIDTH: usize = 4;

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let tokens = lex::lex(&input);
    eprintln!("{tokens:?}");

    Ok(())
}
