use std::io::{self, Read, Write};

const HELP: &str = "Usage: tortilla [-h, --help] [--width <WIDTH>] [--tabs <TABS>] [--crlf]\n";

fn main() -> io::Result<()> {
    let mut toppings = tortilla::Toppings::default();
    args(&mut toppings)?;

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let mut stdout = io::stdout().lock();

    for token in tortilla::wrap(&input, toppings) {
        stdout.write_all(token.as_bytes())?;
    }

    stdout.flush()
}

fn args(toppings: &mut tortilla::Toppings) -> io::Result<()> {
    enum State {
        Clean,
        Width,
        Tabs,
    }

    let mut state = State::Clean;

    for arg in std::env::args().skip(1) {
        match state {
            State::Clean => match arg.as_str() {
                "--width" => state = State::Width,
                "--tabs" => state = State::Tabs,

                "--crlf" => toppings.newline = tortilla::Newline::CRLF,

                "-h" | "--help" => {
                    io::stderr().lock().write_all(HELP.as_bytes())?;
                    std::process::exit(0);
                }

                other => {
                    eprintln!("Unexpected argument '{other}'");
                    std::process::exit(1);
                }
            },

            State::Width => {
                state = State::Clean;
                toppings.line_width = arg.parse().unwrap_or_else(|err| {
                    eprintln!("Bad argument '{arg}' for option '--width': {err}");
                    std::process::exit(1);
                });
            }

            State::Tabs => {
                state = State::Clean;
                toppings.tab_size = arg.parse().unwrap_or_else(|err| {
                    eprintln!("Bad argument '{arg}' for option '--tabs': {err}");
                    std::process::exit(1);
                });
            }
        }
    }

    match state {
        State::Clean => return Ok(()),
        State::Width => eprintln!("Missing argument for option '--width'"),
        State::Tabs => eprintln!("Missing argument for option '--tabs'"),
    }

    std::process::exit(1);
}
