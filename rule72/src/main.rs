use anyhow::Result;
use clap::{Arg, Command};
use rule72::{reflow, Options};
use std::io::{self, Read};

fn main() -> Result<()> {
    let matches = Command::new("rule72")
        .version("0.1.0")
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
        .get_matches();

    let width: usize = matches.get_one::<String>("width").unwrap().parse()?;
    let headline_width: usize = matches.get_one::<String>("headline-width").unwrap().parse()?;
    let strip_ansi = matches.get_flag("no-ansi");
    let debug_svg = matches.get_one::<String>("debug-svg").cloned();

    let opts = Options {
        width,
        headline_width,
        strip_ansi,
        debug_svg,
    };

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let output = reflow(&input, &opts);
    print!("{}", output);

    Ok(())
}
