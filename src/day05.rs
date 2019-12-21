use std::io;

use crate::computer::{ComputerST, Queue, Rom};
use crate::error::Error;

pub fn run<R>(input: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    let rom = Rom::from_reader(input)?;

    // Part 1
    let mut computer = ComputerST::new(&rom);
    computer.input_mut().enqueue(1);
    computer.run()?;
    let answer1 = computer
        .output_mut()
        .pop_back()
        .ok_or_else(|| error!("Nothing in output channel for part 1."))?;

    // Part 2
    let mut computer = ComputerST::new(&rom);
    computer.input_mut().enqueue(5);
    computer.run()?;
    let answer2 = computer
        .output_mut()
        .pop_back()
        .ok_or_else(|| error!("Nothing in output channel for part 2."))?;

    Ok((answer1.to_string(), answer2.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils;

    #[test]
    fn test_05() {
        utils::tests::test_full_problem(5, run, "2845163", "9436229");
    }
}
