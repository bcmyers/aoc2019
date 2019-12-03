use std::io;

use crate::error::Error;

pub fn run<R>(input: R) -> Result<(), Error>
where
    R: io::BufRead,
{
    let (answer1, answer2) = run2(input)?;
    println!("{}", answer1);
    println!("{}", answer2);
    Ok(())
}

pub fn run2<R>(mut input: R) -> Result<(usize, usize), Error>
where
    R: io::BufRead,
{
    let mut buffer = String::new();
    let mut total1 = 0;
    let mut total2 = 0;

    loop {
        if input.read_line(&mut buffer)? == 0 {
            break;
        }

        let n = buffer.trim().parse::<usize>()?;

        total1 += part_one(n);
        total2 += part_two(n);

        buffer.clear();
    }

    Ok((total1, total2))
}

fn part_one(n: usize) -> usize {
    match (n / 3).checked_sub(2) {
        Some(m) => m,
        None => 0,
    }
}

fn part_two(mut n: usize) -> usize {
    let mut total = 0;
    loop {
        let m = match (n / 3).checked_sub(2) {
            Some(m) => m,
            None => break total,
        };
        total += m;
        n = m;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_01() {
        let test_cases = &[
            // (input, expected1, expected2)
            ("12", 2, 2),
            ("14", 2, 2),
            ("1969", 654, 966),
            ("100756", 33583, 50346),
        ];

        for (input, expected1, expected2) in test_cases {
            let reader = io::BufReader::new(input.as_bytes());
            let (actual1, actual2) = run2(reader).unwrap();
            assert_eq!(*expected1, actual1);
            assert_eq!(*expected2, actual2);
        }
    }
}
