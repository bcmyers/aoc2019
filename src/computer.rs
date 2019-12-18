use std::convert::TryFrom;
use std::io;
use std::time::Duration;

use crossbeam::channel::{self, Receiver, Sender};

use crate::error::Error;

#[derive(Clone, Debug)]
pub(crate) struct Rom(Vec<i64>);

impl Rom {
    pub(crate) fn from_reader<R>(mut reader: R) -> Result<Self, Error>
    where
        R: io::BufRead,
    {
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        let vec = buf
            .trim()
            .split(',')
            .map(|s| s.trim().parse::<i64>().map_err(Error::from))
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Rom(vec))
    }
}

impl AsRef<[i64]> for Rom {
    fn as_ref(&self) -> &[i64] {
        &self.0
    }
}

impl std::ops::Deref for Rom {
    type Target = [i64];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Rom {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Channel<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

impl<T> Default for Channel<T> {
    fn default() -> Self {
        let (sender, receiver) = channel::bounded(1024);
        Self { sender, receiver }
    }
}

impl<T> Channel<T> {
    pub(crate) fn into_parts(self) -> (Sender<T>, Receiver<T>) {
        (self.sender, self.receiver)
    }

    pub(crate) fn push_back(&mut self, val: T) {
        self.sender.send(val).unwrap()
    }

    pub(crate) fn pop_front(&mut self) -> Result<T, Error> {
        use crossbeam::channel::RecvTimeoutError;
        match self.receiver.recv_timeout(Duration::from_secs(5)) {
            Ok(val) => Ok(val),
            Err(e) => match e {
                RecvTimeoutError::Timeout => {
                    bail!("Attempted to pop value off channel, but timed out.")
                }
                RecvTimeoutError::Disconnected => unreachable!(),
            },
        }
    }

    pub(crate) fn try_clear(&mut self) {
        let mut iter = self.receiver.try_iter();
        while iter.next().is_some() {}
    }

    pub(crate) fn try_iter<'a>(&'a mut self) -> impl Iterator<Item = T> + 'a {
        self.receiver.try_iter()
    }
}

pub(crate) struct Computer {
    ram: Vec<i64>,
    input: Channel<i64>,
    output: Channel<i64>,
    rb: i64, // Relative base
    pc: u64, // Program counter
}

impl Default for Computer {
    fn default() -> Self {
        Self {
            ram: Vec::new(),
            input: Channel::default(),
            output: Channel::default(),
            rb: 0,
            pc: 0,
        }
    }
}

impl Computer {
    pub(crate) fn with_io(input: Channel<i64>, output: Channel<i64>) -> Self {
        Self {
            ram: Vec::new(),
            input,
            output,
            rb: 0,
            pc: 0,
        }
    }

    pub(crate) fn execute<R>(
        &mut self,
        rom: R,
        noun_and_verb: Option<(i64, i64)>,
    ) -> Result<i64, Error>
    where
        R: AsRef<[i64]>,
    {
        // reset state
        self.pc = 0;
        self.ram = rom.as_ref().to_vec();
        self.rb = 0;

        // set inputs
        if let Some((noun, verb)) = noun_and_verb {
            self.ram.write(1, noun)?;
            self.ram.write(2, verb)?;
        }

        // main loop
        loop {
            let instruction = self.read_instruction()?;
            if self.execute_instruction(instruction)?.is_none() {
                break;
            };
        }

        Ok(self.ram[0])
    }

    fn read_instruction(&mut self) -> Result<Instruction, Error> {
        let (opcode, mut modes) = self.ram.read_opcode(&mut self.pc)?;
        let instruction = match opcode {
            1 => Instruction::Add {
                a: self.ram.read_signed(&mut modes, self.rb, &mut self.pc)?,
                b: self.ram.read_signed(&mut modes, self.rb, &mut self.pc)?,
                w: self.ram.read_ptr(&mut modes, self.rb, &mut self.pc)?,
            },
            2 => Instruction::Multiply {
                a: self.ram.read_signed(&mut modes, self.rb, &mut self.pc)?,
                b: self.ram.read_signed(&mut modes, self.rb, &mut self.pc)?,
                w: self.ram.read_ptr(&mut modes, self.rb, &mut self.pc)?,
            },
            3 => Instruction::Input {
                w: self.ram.read_ptr(&mut modes, self.rb, &mut self.pc)?,
            },
            4 => Instruction::Output {
                a: self.ram.read_signed(&mut modes, self.rb, &mut self.pc)?,
            },
            5 => Instruction::JumpIfTrue {
                a: self.ram.read_signed(&mut modes, self.rb, &mut self.pc)?,
                p: self.ram.read_unsigned(&mut modes, self.rb, &mut self.pc)?,
            },
            6 => Instruction::JumpIfFalse {
                a: self.ram.read_signed(&mut modes, self.rb, &mut self.pc)?,
                p: self.ram.read_unsigned(&mut modes, self.rb, &mut self.pc)?,
            },
            7 => Instruction::LessThan {
                a: self.ram.read_signed(&mut modes, self.rb, &mut self.pc)?,
                b: self.ram.read_signed(&mut modes, self.rb, &mut self.pc)?,
                w: self.ram.read_ptr(&mut modes, self.rb, &mut self.pc)?,
            },
            8 => Instruction::Equals {
                a: self.ram.read_signed(&mut modes, self.rb, &mut self.pc)?,
                b: self.ram.read_signed(&mut modes, self.rb, &mut self.pc)?,
                w: self.ram.read_ptr(&mut modes, self.rb, &mut self.pc)?,
            },
            9 => Instruction::RelativeBase {
                a: self.ram.read_signed(&mut modes, self.rb, &mut self.pc)?,
            },
            99 => Instruction::Halt,
            _ => bail!("Unrecognized opcode {}", opcode),
        };
        Ok(instruction)
    }

    fn execute_instruction(&mut self, instruction: Instruction) -> Result<Option<()>, Error> {
        match instruction {
            Instruction::Add { a, b, w } => {
                self.ram.write(w, a + b)?;
            }
            Instruction::Multiply { a, b, w } => {
                self.ram.write(w, a * b)?;
            }
            Instruction::Input { w } => {
                let val = self.input.pop_front()?;
                self.ram.write(w, val)?;
            }
            Instruction::Output { a } => {
                self.output.push_back(a);
            }
            Instruction::JumpIfTrue { a, p } => {
                if a != 0 {
                    self.pc = p;
                }
            }
            Instruction::JumpIfFalse { a, p } => {
                if a == 0 {
                    self.pc = p;
                }
            }
            Instruction::LessThan { a, b, w } => {
                if a < b {
                    self.ram.write(w, 1)?;
                } else {
                    self.ram.write(w, 0)?;
                }
            }
            Instruction::Equals { a, b, w } => {
                if a == b {
                    self.ram.write(w, 1)?;
                } else {
                    self.ram.write(w, 0)?;
                }
            }
            Instruction::RelativeBase { a } => {
                self.rb += a;
            }
            Instruction::Halt => {
                return Ok(None);
            }
        }
        Ok(Some(()))
    }

    pub(crate) fn input_mut(&mut self) -> &mut Channel<i64> {
        &mut self.input
    }

    pub(crate) fn output_mut(&mut self) -> &mut Channel<i64> {
        &mut self.output
    }

    #[allow(unused)]
    pub(crate) fn ram(&self) -> &[i64] {
        &self.ram
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Mode {
    Position,
    Immediate,
    Relative,
}

impl TryFrom<u64> for Mode {
    type Error = Error;
    fn try_from(n: u64) -> Result<Self, Self::Error> {
        let output = match n {
            0 => Mode::Position,
            1 => Mode::Immediate,
            2 => Mode::Relative,
            _ => bail!("Unrecognized addressing mode {}", n),
        };
        Ok(output)
    }
}

struct Modes(u64);

impl Iterator for Modes {
    type Item = Result<Mode, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            0 => Some(Ok(Mode::Position)),
            _ => {
                let mode = match Mode::try_from(self.0 % 10) {
                    Ok(mode) => mode,
                    Err(e) => return Some(Err(e)),
                };
                self.0 /= 10;
                Some(Ok(mode))
            }
        }
    }
}

trait Memory {
    fn read(&mut self, ptr: u64) -> Result<i64, Error>;
    fn write(&mut self, ptr: u64, val: i64) -> Result<(), Error>;

    fn read_opcode(&mut self, pc: &mut u64) -> Result<(u64, Modes), Error> {
        let n = self.read(*pc)?;
        *pc += 1;
        if n < 0 {
            bail!("Read negative opcode {}, which is not allowed.", n);
        }
        let mut n = n as u64;
        let opcode = n % 100;
        n /= 100;
        let modes = Modes(n);
        Ok((opcode, modes))
    }

    fn read_signed(&mut self, modes: &mut Modes, rb: i64, pc: &mut u64) -> Result<i64, Error> {
        let mut val = self.read(*pc)?;
        *pc += 1;

        let mode = modes.next().unwrap()?;
        match mode {
            Mode::Immediate => Ok(val),
            Mode::Position | Mode::Relative => {
                if mode == Mode::Relative {
                    val += rb;
                }
                if val < 0 {
                    bail!(
                        "Encountered negative pointer {}, which is not allowed.",
                        val
                    );
                }
                let val2 = self.read(val as u64)?;
                Ok(val2)
            }
        }
    }
    fn read_unsigned(&mut self, modes: &mut Modes, rb: i64, pc: &mut u64) -> Result<u64, Error> {
        let val = self.read_signed(modes, rb, pc)?;
        if val < 0 {
            bail!("Reading unsigned integer but found negative value {}.", val);
        }
        Ok(val as u64)
    }

    fn read_ptr(&mut self, modes: &mut Modes, rb: i64, pc: &mut u64) -> Result<u64, Error> {
        let val = self.read(*pc)?;
        *pc += 1;

        let mode = modes.next().unwrap()?;
        let val2 = match mode {
            Mode::Immediate | Mode::Position => val,
            Mode::Relative => val + rb,
        };

        if val2 < 0 {
            bail!(
                "Encountered negative pointer {}, which is not allowed.",
                val2
            );
        }
        Ok(val2 as u64)
    }
}

impl Memory for Vec<i64> {
    fn read(&mut self, ptr: u64) -> Result<i64, Error> {
        if ptr as usize >= self.len() {
            self.resize(ptr as usize + 1, 0);
        }
        let value = *self.get(ptr as usize).unwrap();
        Ok(value)
    }

    fn write(&mut self, ptr: u64, val: i64) -> Result<(), Error> {
        if ptr as usize >= self.len() {
            self.resize(ptr as usize + 1, 0);
        }
        let reference = self.get_mut(ptr as usize).unwrap();
        *reference = val;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum Instruction {
    Add { a: i64, b: i64, w: u64 },
    Multiply { a: i64, b: i64, w: u64 },
    Input { w: u64 },
    Output { a: i64 },
    JumpIfTrue { a: i64, p: u64 },
    JumpIfFalse { a: i64, p: u64 },
    LessThan { a: i64, b: i64, w: u64 },
    Equals { a: i64, b: i64, w: u64 },
    RelativeBase { a: i64 },
    Halt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_computer() {
        let test_cases = &[
            // (input, noun, verb, expected_ram)
            ("1,0,0,0,99", 0, 0, "2,0,0,0,99"),
            ("2,3,0,3,99", 3, 0, "2,3,0,6,99"),
            ("2,4,4,5,99,0", 4, 4, "2,4,4,5,99,9801"),
            ("1,1,1,4,99,5,6,0,99", 1, 1, "30,1,1,4,2,5,6,0,99"),
        ];

        for (input, noun, verb, expected_ram) in test_cases {
            let reader = io::BufReader::new(input.as_bytes());
            let rom = Rom::from_reader(reader).unwrap();
            let mut computer = Computer::default();
            let _ = computer.execute(&rom, Some((*noun, *verb))).unwrap();
            let expected_ram = expected_ram
                .split(",")
                .map(|s| s.trim().parse::<i64>().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(computer.ram(), &expected_ram[..]);
        }
    }
}
