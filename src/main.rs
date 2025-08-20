use std::io::{self, Read, Write};
use tortilla::{Guacamole, Salsa, Toppings};

const HELP: &str = "Usage: tortilla [-h, --help] [--width <WIDTH>] [--tabs <TABS>] [--crlf] [--salsa] [--guacamole]\n";

enum Sauce {
    Salsa,
    Guacamole,
}

fn order() -> io::Result<(Sauce, Toppings)> {
    let mut args = std::env::args().skip(1);

    let mut sauce = Sauce::Guacamole;
    let mut toppings = tortilla::Toppings::default();

    macro_rules! exit {
        ($($arg:tt)*) => {{
            eprintln!($($arg)*);
            std::process::exit(1);
        }};
    }

    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--width" => {
                let Some(value) = args.next() else {
                    exit!("Missing value for flag '--width'");
                };
                toppings = toppings.width(value.parse().unwrap_or_else(|err| {
                    exit!("Bad value '{value}' for option '--width': {err}");
                }));
            }

            "--tabs" => {
                let Some(value) = args.next() else {
                    exit!("Missing value for flag '--tabs'");
                };
                toppings = toppings.tabs(value.parse().unwrap_or_else(|err| {
                    exit!("Bad value '{value}' for option '--tabs': {err}");
                }));
            }

            "--crlf" => toppings = toppings.newline(tortilla::Newline::CRLF),

            "--salsa" => sauce = Sauce::Salsa,
            "--guacamole" => sauce = Sauce::Guacamole,

            "-h" | "--help" => {
                io::stderr().lock().write_all(HELP.as_bytes())?;
                std::process::exit(0);
            }

            other => exit!("Unexpected argument '{other}'"),
        }
    }

    Ok((sauce, toppings))
}

fn main() -> io::Result<()> {
    let (sauce, toppings) = order()?;

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    #[cfg(unix)]
    let mut mouth = {
        // This is ~50% faster than io::stdout() on macOS when processing
        // large (several MB) files.
        use std::os::unix::io::FromRawFd;
        io::BufWriter::new(unsafe { std::fs::File::from_raw_fd(1) })
    };

    #[cfg(not(unix))]
    let mut mouth = io::stdout().lock();

    match sauce {
        Sauce::Salsa => {
            for bite in tortilla::wrap::<Salsa>(&input, toppings) {
                mouth.write_all(bite.as_bytes())?;
            }
        }
        Sauce::Guacamole => {
            for bite in tortilla::wrap::<Guacamole>(&input, toppings) {
                mouth.write_all(bite.as_bytes())?;
            }
        }
    }

    mouth.flush() // Stay hydrated.
}
