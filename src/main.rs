use std::io::{self, Read, Write};

use tortilla::*;

fn main() -> io::Result<()> {
    let options = args();

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let newline = options
        .newline
        .unwrap_or(Newline::first_in(&input).unwrap_or_default());

    let mut stdout = io::stdout().lock();

    write_all(&mut stdout, transform(lex(&input), options), newline)?;
    stdout.flush()
}

const HELP: &str = "Usage: tortilla [-h, --help] [--width <WIDTH>] [--tabs <TABS>] [--lf] [--crlf]";

fn args() -> Options {
    let mut options = Options::default();

    enum State {
        Clean,
        Width,
        Tabs,
    }

    let mut state = State::Clean;

    for arg in std::env::args().skip(1) {
        match state {
            State::Clean => match arg.as_str() {
                "-h" | "--help" => {
                    eprintln!("{HELP}");
                    std::process::exit(0);
                }

                "--width" => state = State::Width,
                "--tabs" => state = State::Tabs,

                "--lf" => options.newline = Some(Newline::LF),
                "--crlf" => options.newline = Some(Newline::CRLF),

                other => {
                    eprintln!("Unexpected argument '{other}'");
                    std::process::exit(1);
                }
            },

            State::Width => {
                state = State::Clean;
                options.line_width = arg.parse().unwrap_or_else(|err| {
                    eprintln!("Bad argument '{arg}' for option '--width': {err}");
                    std::process::exit(1);
                })
            }

            State::Tabs => {
                state = State::Clean;
                options.tab_width = arg.parse().unwrap_or_else(|err| {
                    eprintln!("Bad argument '{arg}' for option '--tabs': {err}");
                    std::process::exit(1);
                })
            }
        }
    }

    match state {
        State::Clean => return options,
        State::Width => eprintln!("Missing argument for option '--width'"),
        State::Tabs => eprintln!("Missing argument for option '--tabs'"),
    }

    std::process::exit(1);
}
