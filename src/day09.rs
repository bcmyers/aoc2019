use std::io;

use crate::computer::{Channel, Computer, Rom};
use crate::error::Error;

pub fn run<R>(reader: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    let rom = Rom::from_reader(reader)?;
    let mut computer = Computer::new(Channel::default(), Channel::default());

    computer.input_mut().push_back(1);
    computer.execute(&rom, None)?;
    let answer1 = computer.output_mut().pop_front()?;

    computer.input_mut().try_clear();
    computer.output_mut().try_clear();

    computer.input_mut().push_back(2);
    computer.execute(&rom, None)?;
    let answer2 = computer.output_mut().pop_front()?;

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
            let mut computer = Computer::new(Channel::default(), Channel::default());
            computer.execute(&rom, None).unwrap();
            let actual = computer.output_mut().try_iter().collect::<Vec<_>>();
            assert_eq!(*expected, &actual[..]);
        }

        utils::tests::test_full_problem(9, run, "3460311188", "42202");
    }
}
