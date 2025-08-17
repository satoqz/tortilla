use std::io::{self, Read};
use wraplines::*;

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    write(
        io::stdout().lock(),
        transform(lex(&input), Options::default()),
    )
}
