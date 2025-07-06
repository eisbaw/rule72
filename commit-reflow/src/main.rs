use std::io::{self, Read};

use clap::Parser;

use commit_reflow::{reflow, Options};

/// Reflow git commit messages.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Wrap width for body prose
    #[arg(short = 'w', long, default_value_t = 72)]
    width: usize,

    /// Advisory width for headline (subject) line
    #[arg(long, default_value_t = 50)]
    headline_width: usize,

    /// Strip ANSI color codes before measuring width
    #[arg(long)]
    no_ansi: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;

    let opts = Options {
        width: args.width,
        headline_width: args.headline_width,
        strip_ansi: args.no_ansi,
    };

    let output = reflow(&buf, &opts);
    print!("{}", output);
    Ok(())
}
