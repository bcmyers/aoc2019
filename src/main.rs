use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::exit;

use structopt::StructOpt;

use aoc2019::{self, bail, Error, Reader};

#[derive(Debug, StructOpt)]
struct Opt {
    /// Day
    day: usize,

    /// Optional path to input file; if not supplied will read from stdin
    input: Option<PathBuf>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        let mut e: &dyn std::error::Error = &e;
        while let Some(source) = e.source() {
            eprintln!("  - caused by: {}", source);
            e = source;
        }
        exit(1);
    }
}

fn run() -> Result<(), Error> {
    let opt = Opt::from_args();

    let stdin = io::stdin();

    let input = match opt.input {
        Some(path) => {
            let file = fs::File::open(path)?;
            let reader = io::BufReader::new(file);
            Reader::File(reader)
        }
        None => {
            let guard = stdin.lock();
            Reader::Stdin(guard)
        }
    };

    match opt.day {
        1 => aoc2019::day01::run(input)?,
        2 => aoc2019::day02::run(input)?,
        3 => aoc2019::day03::run(input)?,
        n if n > 0 && n < 26 => bail!("Day {} is not yet implemented.", n),
        _ => bail!("Day must be between 1 and 25, inclusive."),
    };

    Ok(())
}
