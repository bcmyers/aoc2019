use std::io;

use crate::computer::{ComputerST, Queue, Rom};
use crate::error::Error;

pub fn run<R>(reader: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    let rom = Rom::from_reader(reader)?;

    let mut computer = ComputerST::new(&rom);
    computer.input_mut().enqueue(1);
    computer.run()?;
    let answer1 = computer.output_mut().dequeue()?;

    let mut computer = ComputerST::new(&rom);
    computer.input_mut().enqueue(2);
    computer.run()?;
    let answer2 = computer.output_mut().dequeue()?;

    Ok((answer1.to_string(), answer2.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils;

    #[test]
    fn test_09() {
        let test_cases = &[
            (
                "109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99",
                &[
                    109i64, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
                ][..],
            ),
            (
                "1102,34915192,34915192,7,4,7,99,0",
                &[34915192 * 34915192][..],
            ),
            ("104,1125899906842624,99", &[1125899906842624][..]),
        ];

        for (input, expected) in test_cases {
            let reader = io::BufReader::new(input.as_bytes());
            let rom = Rom::from_reader(reader).unwrap();
            let mut computer = ComputerST::new(&rom);
            computer.run().unwrap();
            let actual = computer.output_mut().iter().cloned().collect::<Vec<i64>>();
            assert_eq!(*expected, &actual[..]);
        }

        utils::tests::test_full_problem(9, run, "3460311188", "42202");
    }
}
