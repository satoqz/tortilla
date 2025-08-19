use std::io::{self, Read, Write};

const HELP: &str = "Usage: tortilla [-h, --help] [--width <WIDTH>] [--tabs <TABS>] [--crlf]\n";

fn main() -> io::Result<()> {
    let toppings = order()?;

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    #[cfg(unix)]
    let mut stdout = {
        // This is ~50% faster than io::stdout() on macOS when processing
        // large (several MB) files.
        use std::os::unix::io::FromRawFd;
        io::BufWriter::new(unsafe { std::fs::File::from_raw_fd(1) })
    };

    #[cfg(not(unix))]
    let mut stdout = io::stdout().lock();

    for token in tortilla::wrap(&input, toppings) {
        stdout.write_all(token.as_bytes())?;
    }

    stdout.flush()
}

fn order() -> io::Result<tortilla::Toppings> {
    let mut args = std::env::args().skip(1);
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

            "-h" | "--help" => {
                io::stderr().lock().write_all(HELP.as_bytes())?;
                std::process::exit(0);
            }

            other => exit!("Unexpected argument '{other}'"),
        }
    }

    Ok(toppings)
}
