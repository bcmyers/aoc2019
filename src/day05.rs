use std::io;

use crate::computer::{Channel, Computer, Rom};
use crate::error::Error;

pub fn run<R>(input: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    let rom = Rom::from_reader(input)?;
    let mut computer = Computer::new(Channel::default(), Channel::default());

    // Part 1
    computer.input_mut().push_back(1);
    computer.execute(&rom, None)?;
    let answer1 = computer
        .output_mut()
        .try_iter()
        .last()
        .ok_or_else(|| error!("Nothing in output channel for part 1."))?;

    // Resetting I/O state
    computer.input_mut().try_clear();
    computer.output_mut().try_clear();

    // Part 2
    computer.input_mut().push_back(5);
    computer.execute(&rom, None)?;
    let answer2 = computer
        .output_mut()
        .try_iter()
        .last()
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
