use std::io;

use crate::computer::Computer;
use crate::error::Error;

pub fn run<R>(input: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    let mut computer = Computer::new(input)?;
    let answer1 = computer.execute(None, Some((12, 2)))?;

    let mut answer2 = Err(error!(
        "Invalid input. Unable to find noun/verb combination that outputs 19690720."
    ));
    for noun in 0..=99 {
        for verb in 0..=99 {
            if computer.execute(None, Some((noun, verb)))? == 19690720 {
                answer2 = Ok(100 * noun + verb);
            }
        }
    }

    Ok((answer1.to_string(), answer2?.to_string()))
}
