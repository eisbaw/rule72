//! Command-line interface for rule72 commit message formatter.
//!
//! Reads commit messages from stdin and outputs formatted text to stdout,
//! making it suitable for Git hooks, editor integration, and batch processing.

use anyhow::Result;
use clap::{Arg, Command};
use rule72::{reflow, Options};
use std::io::{self, Read};

/// Main entry point for the rule72 CLI application.
///
/// Parses command-line arguments, reads from stdin, applies text reflow,
/// and outputs the result to stdout.
fn main() -> Result<()> {
    let matches = Command::new("rule72")
        .version("0.2.2")
        .about("Git commit message formatter")
        .arg(
            Arg::new("width")
                .short('w')
                .long("width")
                .value_name("N")
                .help("Set body wrap width")
                .default_value("72"),
        )
        .arg(
            Arg::new("headline-width")
                .long("headline-width")
                .value_name("N")
                .help("Advisory headline width")
                .default_value("50"),
        )
        .arg(
            Arg::new("no-ansi")
                .long("no-ansi")
                .help("Strip ANSI color codes before measuring width")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("debug-svg")
                .long("debug-svg")
                .value_name("PATH")
                .help("Output SVG visualization of parsing/classification"),
        )
        .arg(
            Arg::new("debug-trace")
                .long("debug-trace")
                .help("Output detailed trace of parsing pipeline")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let width: usize = matches.get_one::<String>("width").unwrap().parse()?;
    let headline_width: usize = matches
        .get_one::<String>("headline-width")
        .unwrap()
        .parse()?;
    let strip_ansi = matches.get_flag("no-ansi");
    let debug_svg = matches.get_one::<String>("debug-svg").cloned();
    let debug_trace = matches.get_flag("debug-trace");

    let opts = Options {
        width,
        headline_width,
        strip_ansi,
        debug_svg,
        debug_trace,
    };

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let output = reflow(&input, &opts);
    print!("{}", output);

    Ok(())
}
