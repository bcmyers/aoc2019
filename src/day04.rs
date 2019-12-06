use std::io;

use crate::error::Error;

pub fn run<R>(input: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    let (low, high) = read_input(input)?;

    let (mut answer1, mut answer2) = (0, 0);
    for n in low..=high {
        let is_valid = is_valid(n)?;
        if is_valid.0 {
            answer1 += 1;
        }
        if is_valid.1 {
            answer2 += 1;
        }
    }

    Ok((answer1.to_string(), answer2.to_string()))
}

fn is_valid(n: usize) -> Result<(bool, bool), Error> {
    let mut is_valid = (false, false);
    let mut min = 0;
    let mut previous_digits = PreviousDigits::None;

    for (i, digit) in parse_digits(n)?.iter().enumerate() {
        let digit = *digit;
        if digit < min {
            return Ok((false, false));
        }
        min = digit;

        match previous_digits {
            PreviousDigits::None => {
                previous_digits = PreviousDigits::One(digit);
            }
            PreviousDigits::One(d) => {
                if digit == d {
                    is_valid.0 = true;
                    if i == 5 {
                        is_valid.1 = true;
                    }
                    previous_digits = PreviousDigits::Two(digit);
                } else {
                    previous_digits = PreviousDigits::One(digit);
                }
            }
            PreviousDigits::Two(d) => {
                if digit == d {
                    previous_digits = PreviousDigits::ThreeOrMore(digit);
                } else {
                    is_valid.1 = true;
                    previous_digits = PreviousDigits::One(digit);
                }
            }
            PreviousDigits::ThreeOrMore(d) => {
                if digit == d {
                } else {
                    previous_digits = PreviousDigits::One(digit);
                }
            }
        }
    }

    Ok(is_valid)
}

fn read_input<R>(mut reader: R) -> Result<(usize, usize), Error>
where
    R: io::BufRead,
{
    let parse = |s: &str| s.trim().parse::<usize>();
    let error = || error!("Invalid input.");

    let mut s = String::new();
    reader.read_to_string(&mut s)?;

    let mut iter = s.split("-");
    let low = iter.next().map(parse).ok_or_else(error)??;
    let high = iter.next().map(parse).ok_or_else(error)??;

    if iter.next().is_some() {
        bail!("Invalid input.")
    }

    Ok((low, high))
}

fn parse_digits(mut n: usize) -> Result<[u8; 6], Error> {
    if n < 100_000 || n > 999_999 {
        bail!("Input must be a 6 digit number.")
    }
    let mut output = [0u8; 6];
    let mut i = 5;
    loop {
        output[i] = (n % 10) as u8;
        n /= 10;
        if i == 0 {
            break;
        }
        i -= 1;
    }
    Ok(output)
}

enum PreviousDigits {
    None,
    One(u8),
    Two(u8),
    ThreeOrMore(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_04() {
        let test_cases = &[
            (111111, true, false),
            (223450, false, false),
            (123789, false, false),
            (112233, true, true),
            (123444, true, false),
            (111122, true, true),
        ];

        for (n, expected1, expected2) in test_cases {
            let (actual1, actual2) = is_valid(*n).unwrap();
            assert_eq!(actual1, *expected1);
            assert_eq!(actual2, *expected2);
        }
    }
}
