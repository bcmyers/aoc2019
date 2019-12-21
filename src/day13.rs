use std::io;

use crate::computer::{ComputerST, Queue, Rom, State};
use crate::error::Error;

const ROWS: usize = 26;
const COLS: usize = 40;

pub fn run<R>(reader: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    let rom = Rom::from_reader(reader)?;
    let mut game = Game::new(&rom);
    game.run()?;
    Ok((game.nblocks.to_string(), game.score.to_string()))
}

pub struct Game {
    computer: ComputerST,
    display: Vec<u8>,

    x: Option<i64>,
    y: Option<i64>,
    id: Option<i64>,
    score: i64,
    ball: i64,
    paddle: i64,
    nblocks: usize,
    first: bool,
}

impl Game {
    pub fn new<R>(rom: R) -> Self
    where
        R: AsRef<[i64]>,
    {
        let mut computer = ComputerST::new(rom.as_ref());
        computer.write(0, 2);
        Self {
            computer,
            display: vec![0; COLS * ROWS],

            x: None,
            y: None,
            id: None,
            score: 0,
            ball: 0,
            paddle: 0,
            nblocks: 0,
            first: true,
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            match self.step()? {
                Some(next_move) => self.computer.input_mut().enqueue(next_move),
                None => break,
            }
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<Option<i64>, Error> {
        loop {
            match self.computer.step()? {
                State::Done => return Ok(None),
                State::NeedsInput => {
                    if self.first {
                        self.nblocks = bytecount::count(&self.display[..], 2);
                        self.first = false;
                    }
                    if self.paddle < self.ball {
                        return Ok(Some(1));
                    } else if self.paddle > self.ball {
                        return Ok(Some(-1));
                    } else {
                        return Ok(Some(0));
                    }
                }
                State::HasOutput => {
                    if self.x.is_none() {
                        self.x = Some(self.computer.output_mut().dequeue().unwrap());
                        continue;
                    }
                    if self.y.is_none() {
                        self.y = Some(self.computer.output_mut().dequeue().unwrap());
                        continue;
                    }
                    if self.id.is_none() {
                        self.id = Some(self.computer.output_mut().dequeue().unwrap());
                    }
                    if self.x.is_some() && self.y.is_some() && self.id.is_some() {
                        let x = self.x.unwrap();
                        let y = self.y.unwrap();
                        let id = self.id.unwrap();
                        if x == -1 && y == 0 {
                            self.score = id;
                            self.x = None;
                            self.y = None;
                            self.id = None;
                            continue;
                        }
                        match id {
                            0 | 1 | 2 => (),
                            3 => self.paddle = x,
                            4 => self.ball = x,
                            _ => bail!("Received invalid id: {}", id),
                        }
                        self.display[(y as usize) * COLS + x as usize] = id as u8;
                        self.x = None;
                        self.y = None;
                        self.id = None;
                    }
                }
            };
        }
    }

    pub const fn rows(&self) -> usize {
        ROWS
    }

    pub const fn cols(&self) -> usize {
        COLS
    }

    pub fn display(&self) -> &[u8] {
        &self.display
    }

    pub fn input(&mut self, val: i64) {
        self.computer.input_mut().enqueue(val)
    }

    pub fn score(&self) -> i64 {
        self.score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils;

    #[test]
    fn test_13() {
        utils::tests::test_full_problem(13, run, "432", "22225")
    }
}
