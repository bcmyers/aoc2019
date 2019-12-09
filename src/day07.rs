use std::io;
use std::sync::{Arc, Barrier};
use std::thread;

use crossbeam::channel;
use itertools::Itertools;

use crate::computer::{Channel, Computer, Rom};
use crate::error::Error;

fn fact(mut n: usize) -> Result<usize, Error> {
    let orig = n;
    let mut answer = 1usize;
    loop {
        answer = match answer.checked_mul(n) {
            Some(val) => val,
            None => bail!("Factorial of {} overflows usize.", orig),
        };
        if (n - 1) == 0 {
            break;
        } else {
            n = n - 1;
        }
    }
    Ok(answer)
}

pub fn run<R>(reader: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    let rom = Rom::from_reader(reader)?;
    let ncomputers = 5;

    let (tx_output, rx_output) = channel::bounded(fact(ncomputers)?);

    let mut handles = Vec::new();
    let mut senders = Vec::new();
    let barrier = Arc::new(Barrier::new(ncomputers));
    for i in 0..ncomputers {
        let (tx_input, rx_input) = channel::bounded(fact(ncomputers)?);
        senders.push(tx_input);

        let barrier = barrier.clone();
        let rom = rom.clone();
        let tx_output = tx_output.clone();

        let handle = thread::spawn(move || {
            loop {
                let (part, phase_setting, input, output) = match rx_input.recv() {
                    Ok(data) => data,
                    Err(_) => break,
                };
                let mut computer = Computer::new(input, output);

                computer.input_mut().push_back(phase_setting);
                if i == 0 {
                    computer.input_mut().push_back(0);
                }

                barrier.wait();
                computer.execute(&rom, None)?;
                barrier.wait();

                if i == 4 {
                    let answer = computer.output_mut().pop_front()?;
                    tx_output.send((part, answer)).unwrap();
                }
            }
            Ok::<_, Error>(())
        });
        handles.push(handle);
    }

    for (part, range) in (&[(0..5), (5..10)]).iter().cloned().enumerate() {
        for phase_settings in range.map(|i| i as i64).permutations(ncomputers) {
            let channels = (0..ncomputers)
                .map(|_| Channel::default())
                .collect::<Vec<_>>();
            let mut outputs = (0..ncomputers).map(|i| channels[i].clone());
            let mut inputs =
                (0..ncomputers).map(|i| channels[(i + ncomputers - 1) % ncomputers].clone());
            for i in 0..ncomputers {
                let output = outputs.next().unwrap();
                let input = inputs.next().unwrap();
                senders[i]
                    .send((part, phase_settings[i], input, output))
                    .unwrap();
            }
        }
    }

    drop(senders);
    drop(tx_output);

    let (mut answer1, mut answer2) = (0, 0);

    let mut iter = rx_output.iter();
    while let Some((part, output)) = iter.next() {
        match part {
            0 => {
                if output > answer1 {
                    answer1 = output;
                }
            }
            1 => {
                if output > answer2 {
                    answer2 = output;
                }
            }
            _ => unreachable!(),
        }
    }

    for handle in handles {
        handle.join().unwrap()?;
    }

    Ok((answer1.to_string(), answer2.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils;

    #[test]
    fn test_07() {
        utils::tests::test_full_problem(7, run, "43812", "59597414");
    }
}
