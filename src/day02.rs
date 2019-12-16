use std::io;

use crate::computer::{Computer, Rom};
use crate::error::Error;

pub fn run<R>(input: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    let rom = Rom::from_reader(input)?;
    let mut computer = Computer::default();
    let answer1 = computer.execute(&rom, Some((12, 2)))?;

    let mut answer2 = Err(error!(
        "Invalid input. Unable to find noun/verb combination that outputs 19690720."
    ));
    for noun in 0..=99 {
        for verb in 0..=99 {
            if computer.execute(&rom, Some((noun, verb)))? == 19690720 {
                answer2 = Ok(100 * noun + verb);
            }
        }
    }

    Ok((answer1.to_string(), answer2?.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils;

    #[test]
    fn test_02() {
        utils::tests::test_full_problem(2, run, "3267740", "7870");
    }
}
