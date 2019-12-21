use std::io;

use crate::computer::{ComputerST, Rom};
use crate::error::Error;

pub fn run<R>(input: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    let rom = Rom::from_reader(input)?;

    let mut computer = ComputerST::new(&rom);
    computer.write(1, 12);
    computer.write(2, 2);
    computer.run()?;
    let answer1 = computer.read(0);

    let mut answer2 = Err(error!(
        "Invalid input. Unable to find noun/verb combination that outputs 19690720."
    ));
    'outer: for noun in 0..=99 {
        for verb in 0..=99 {
            let mut computer = ComputerST::new(&rom);
            computer.write(1, noun);
            computer.write(2, verb);
            computer.run()?;
            if computer.read(0) == 19_690_720 {
                answer2 = Ok(100 * noun + verb);
                break 'outer;
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
