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

    let (answer1, answer2) = match opt.day {
        1 => aoc2019::day01::run(input)?,
        2 => aoc2019::day02::run(input)?,
        3 => aoc2019::day03::run(input)?,
        4 => aoc2019::day04::run(input)?,
        5 => aoc2019::day05::run(input)?,
        6 => aoc2019::day06::run(input)?,
        7 => aoc2019::day07::run(input)?,
        8 => aoc2019::day08::run(input)?,
        9 => aoc2019::day09::run(input)?,
        10 => aoc2019::day10::run(input)?,
        11 => aoc2019::day11::run(input)?,
        12 => aoc2019::day12::run(input)?,
        13 => aoc2019::day13::run(input)?,
        14 => aoc2019::day14::run(input)?,
        15 => aoc2019::day15::run(input)?,
        n if n > 0 && n < 26 => bail!("Day {} is not yet implemented.", n),
        _ => bail!("Day must be between 1 and 25, inclusive."),
    };

    println!("{}", answer1);
    println!("{}", answer2);

    Ok(())
}
