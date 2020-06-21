use std::collections::VecDeque;
use std::convert::TryFrom;
use std::io;
use std::time::Duration;

use crossbeam::channel::{self, Receiver, Sender};

use crate::error::Error;

pub type ComputerST = Computer<VecDeque<i64>>;
pub type ComputerMT = Computer<Channel<i64>>;

#[derive(Clone, Debug)]
pub struct Computer<Q> {
    pc: u64, // Program counter
    rb: i64, // Relative base
    ram: Vec<i64>,
    state: StateInternal,
    input: Q,
    output: Q,
}

impl Computer<VecDeque<i64>> {
    pub fn new<R>(rom: R) -> Computer<VecDeque<i64>>
    where
        R: AsRef<[i64]>,
    {
        Self {
            pc: 0,
            rb: 0,
            ram: rom.as_ref().to_vec(),
            state: StateInternal::Executing,
            input: VecDeque::default(),
            output: VecDeque::default(),
        }
    }
}

impl Computer<Channel<i64>> {
    pub fn new<R>(rom: R, input: Channel<i64>, output: Channel<i64>) -> Computer<Channel<i64>>
    where
        R: AsRef<[i64]>,
    {
        Self {
            pc: 0,
            rb: 0,
            ram: rom.as_ref().to_vec(),
            state: StateInternal::Executing,
            input,
            output,
        }
    }
}

impl<Q> Computer<Q>
where
    Q: Queue,
{
    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            match self.step()? {
                State::Done => return Ok(()),
                State::HasOutput => (),
                State::NeedsInput => bail!("Needs input."),
            }
        }
    }

    pub fn step(&mut self) -> Result<State, Error> {
        loop {
            match self.state {
                StateInternal::Done => return Ok(State::Done),
                StateInternal::Executing => {
                    let instruction = self.read_instruction()?;
                    self.execute_instruction(instruction);
                }
                StateInternal::NeedsInput { w } => match self.input.dequeue() {
                    Ok(val) => {
                        self.ram.write(w, val);
                        self.state = StateInternal::Executing;
                    }
                    Err(_) => return Ok(State::NeedsInput),
                },
                StateInternal::HasOutput => {
                    self.state = StateInternal::Executing;
                    return Ok(State::HasOutput);
                }
            }
        }
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

    fn execute_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Add { a, b, w } => {
                self.ram.write(w, a + b);
            }
            Instruction::Multiply { a, b, w } => {
                self.ram.write(w, a * b);
            }
            Instruction::Input { w } => {
                self.state = StateInternal::NeedsInput { w };
                return;
            }
            Instruction::Output { a } => {
                self.output.enqueue(a);
                self.state = StateInternal::HasOutput;
                return;
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
                    self.ram.write(w, 1);
                } else {
                    self.ram.write(w, 0);
                }
            }
            Instruction::Equals { a, b, w } => {
                if a == b {
                    self.ram.write(w, 1);
                } else {
                    self.ram.write(w, 0);
                }
            }
            Instruction::RelativeBase { a } => {
                self.rb += a;
            }
            Instruction::Halt => {
                self.state = StateInternal::Done;
                return;
            }
        }
        self.state = StateInternal::Executing;
    }

    pub fn input_mut(&mut self) -> &mut Q {
        &mut self.input
    }

    pub fn output_mut(&mut self) -> &mut Q {
        &mut self.output
    }

    #[cfg(test)]
    pub(crate) fn ram(&self) -> &[i64] {
        &self.ram
    }

    pub fn read(&mut self, ptr: u64) -> i64 {
        self.ram.read(ptr)
    }

    pub fn write(&mut self, ptr: u64, val: i64) {
        self.ram.write(ptr, val)
    }
}

#[derive(Clone, Debug)]
pub struct Channel<T> {
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
    pub fn into_parts(self) -> (Sender<T>, Receiver<T>) {
        (self.sender, self.receiver)
    }
}

impl Queue for Channel<i64> {
    fn enqueue(&mut self, val: i64) {
        self.sender.send(val).unwrap()
    }

    fn dequeue(&mut self) -> Result<i64, Error> {
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
    fn read(&mut self, ptr: u64) -> i64;
    fn write(&mut self, ptr: u64, val: i64);

    fn read_opcode(&mut self, pc: &mut u64) -> Result<(u64, Modes), Error> {
        let n = self.read(*pc);
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
        let mut val = self.read(*pc);
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
                let val2 = self.read(val as u64);
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
        let val = self.read(*pc);
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
    fn read(&mut self, ptr: u64) -> i64 {
        if ptr as usize >= self.len() {
            self.resize(ptr as usize + 1, 0);
        }
        *self.get(ptr as usize).unwrap()
    }

    fn write(&mut self, ptr: u64, val: i64) {
        if ptr as usize >= self.len() {
            self.resize(ptr as usize + 1, 0);
        }
        let reference = self.get_mut(ptr as usize).unwrap();
        *reference = val;
    }
}

#[derive(Clone, Debug)]
pub struct Rom(Vec<i64>);

impl Rom {
    pub fn from_reader<R>(mut reader: R) -> Result<Self, Error>
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

pub trait Queue {
    fn dequeue(&mut self) -> Result<i64, Error>;
    fn enqueue(&mut self, val: i64);
}

impl Queue for VecDeque<i64> {
    fn dequeue(&mut self) -> Result<i64, Error> {
        self.pop_front()
            .ok_or_else(|| error!("Attempted to pop something off the queue, but queue was empty"))
    }
    fn enqueue(&mut self, val: i64) {
        self.push_back(val);
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum State {
    Done,
    NeedsInput,
    HasOutput,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum StateInternal {
    Done,
    Executing,
    NeedsInput { w: u64 },
    HasOutput,
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
            let mut computer = ComputerST::new(&rom);
            computer.write(1, *noun);
            computer.write(2, *verb);
            computer.run().unwrap();
            let expected_ram = expected_ram
                .split(",")
                .map(|s| s.trim().parse::<i64>().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(computer.ram(), &expected_ram[..]);
        }
    }
}
