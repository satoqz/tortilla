# tortilla

Somewhat syntax-aware text wrapping for source code and plain text documents.

## What?

This is both a Rust crate and a corresponding command line tool for text
wrapping, with some assumptions that are specific to code comments and plain
text documents (e.g., markdown documentation or emails).

In particular, the following features are supported:

* Preservation of indentation (both tabs and spaces)
* Preservation of comment tokens (`//`, `#`, etc.)
* Bulleted/numbered list alignment of subsequent lines (for `-`, `*`, `1.`,
  etc.)
* Multiple paragraphs (delimited by at least two line breaks or a change in
  indentation/comment token)
* Customizable tab width for line width calculation (`--tabs`)
* Line breaking algorithms:
  * Optimal-fit (`--salsa`), this is the default
  * First-fit (`--guacamole`) as a less resource-intensive alternative

## Why?

I like to hard wrap my code comments and plain text documents. It's a
rough reality, but it makes them easier to read, review and maintain, and
soft-wrapping is a highly subpar alternative. In many cases I want an automated
initial pass at hard wrapping, and then follow up with any (rare) manual edits.

Text editors tend to provide hard wrapping functionality to some degree,
but usually have limited understanding of indentation, comment tokens or
bulleted/numbered lists. This project was born out of my frustrations wrapping
text in the [Helix editor](https://helix-editor.com) via the `:reflow` command,
which uses the popular [textwrap](https://crates.io/crates/textwrap) crate.

## Usage

### Installation

Via cargo:

```sh
cargo install tortilla
```

### Command line

```shell-session
$ tortilla --help
Usage: tortilla [-h, --help] [--width <WIDTH>] [--tabs <TABS>] [--crlf] [--salsa] [--guacamole]
```

Input is read from stdin, output is written to stdout. For example:

```shell-session
$ echo "foo bar baz" | tortilla --width 8
foo bar
baz
```

Tortilla wraps to 80 characters with LF (`\n`) newlines by default, counting tab
indents as 4 spaces.

### Text editors

For [Helix](https://helix-editor.com), I suggest the following:

```toml
keys.normal."=" = ":pipe tortilla"
```

### Rust crate

Add the `tortilla` crate to your Rust project:

```sh
cargo add tortilla
```

See [docs.rs/tortilla](https://docs.rs/tortilla) for documentation.

### License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT
License](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
