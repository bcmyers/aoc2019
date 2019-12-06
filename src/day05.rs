use std::io;

use crate::computer::Computer;
use crate::error::Error;

pub fn run<R>(input: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    let mut computer = Computer::new(input)?;
    computer.execute(Some(vec![1]), None)?;
    let answer1 = match computer.output().last() {
        Some(val) => *val,
        None => bail!("Did not get any output."),
    };

    computer.execute(Some(vec![5]), None)?;
    let answer2 = match computer.output().last() {
        Some(val) => *val,
        None => bail!("Did not get any output."),
    };

    Ok((answer1.to_string(), answer2.to_string()))
}
