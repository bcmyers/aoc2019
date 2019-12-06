use std::convert::TryFrom;
use std::io;

use crate::error::Error;

pub(crate) struct Computer {
    rom: Vec<i64>,
    ram: Vec<i64>,

    input: Vec<i64>,
    output: Vec<i64>,

    pc: u64,
}

impl Computer {
    pub(crate) fn new<R>(mut rom_reader: R) -> Result<Self, Error>
    where
        R: io::BufRead,
    {
        let mut buffer = String::new();
        rom_reader.read_to_string(&mut buffer)?;
        let rom = buffer
            .trim()
            .split(",")
            .map(|s| Ok(s.parse::<i64>()?))
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Self {
            rom,
            ram: Vec::new(),
            input: Vec::new(),
            output: Vec::new(),
            pc: 0,
        })
    }

    pub(crate) fn execute(
        &mut self,
        input: Option<Vec<i64>>,
        noun_and_verb: Option<(i64, i64)>,
    ) -> Result<i64, Error> {
        // reset state
        self.output = Vec::new();
        self.pc = 0;
        self.ram = self.rom.clone();

        // set inputs
        if let Some(input) = input {
            self.input = input;
        }
        if let Some((noun, verb)) = noun_and_verb {
            self.ram[1] = noun;
            self.ram[2] = verb;
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
                a: self.ram.read_signed(&mut modes, &mut self.pc)?,
                b: self.ram.read_signed(&mut modes, &mut self.pc)?,
                w: self.ram.read_ptr(&mut self.pc)?,
            },
            2 => Instruction::Multiply {
                a: self.ram.read_signed(&mut modes, &mut self.pc)?,
                b: self.ram.read_signed(&mut modes, &mut self.pc)?,
                w: self.ram.read_ptr(&mut self.pc)?,
            },
            3 => Instruction::Input {
                w: self.ram.read_ptr(&mut self.pc)?,
            },
            4 => Instruction::Output {
                a: self.ram.read_signed(&mut modes, &mut self.pc)?,
            },
            5 => Instruction::JumpIfTrue {
                a: self.ram.read_signed(&mut modes, &mut self.pc)?,
                p: self.ram.read_unsigned(&mut modes, &mut self.pc)?,
            },
            6 => Instruction::JumpIfFalse {
                a: self.ram.read_signed(&mut modes, &mut self.pc)?,
                p: self.ram.read_unsigned(&mut modes, &mut self.pc)?,
            },
            7 => Instruction::LessThan {
                a: self.ram.read_signed(&mut modes, &mut self.pc)?,
                b: self.ram.read_signed(&mut modes, &mut self.pc)?,
                w: self.ram.read_ptr(&mut self.pc)?,
            },
            8 => Instruction::Equals {
                a: self.ram.read_signed(&mut modes, &mut self.pc)?,
                b: self.ram.read_signed(&mut modes, &mut self.pc)?,
                w: self.ram.read_ptr(&mut self.pc)?,
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
                let val = self.input.pop().ok_or_else(|| {
                    error!("Attempted to access input, but input vector was empty.")
                })?;
                self.ram.write(w, val)?;
            }
            Instruction::Output { a } => {
                self.output.push(a);
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
            Instruction::Halt => {
                return Ok(None);
            }
        }
        Ok(Some(()))
    }

    #[allow(unused)]
    pub(crate) fn input(&self) -> &[i64] {
        &self.input
    }

    pub(crate) fn output(&self) -> &[i64] {
        &self.output
    }

    #[allow(unused)]
    pub(crate) fn ram(&self) -> &[i64] {
        &self.ram
    }
}

enum Mode {
    Pointer,
    Immediate,
}

impl TryFrom<u64> for Mode {
    type Error = Error;
    fn try_from(n: u64) -> Result<Self, Self::Error> {
        let output = match n {
            0 => Mode::Pointer,
            1 => Mode::Immediate,
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
            0 => return Some(Ok(Mode::Pointer)),
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
    fn read(&self, ptr: u64) -> Result<i64, Error>;
    fn write(&mut self, ptr: u64, val: i64) -> Result<(), Error>;

    fn read_opcode(&self, pc: &mut u64) -> Result<(u64, Modes), Error> {
        let n = self.read(*pc)?;
        *pc += 1;
        if n < 0 {
            bail!("Read negative opcode {}, which is not allowed.", n);
        }
        let mut n = n as u64;
        let ones = n % 10;
        n /= 10;
        let tens = n % 10;
        n /= 10;
        let opcode = tens * 10 + ones;
        let modes = Modes(n);
        Ok((opcode, modes))
    }

    fn read_signed(&self, modes: &mut Modes, pc: &mut u64) -> Result<i64, Error> {
        let val = self.read(*pc)?;
        *pc += 1;

        let mode = modes.next().unwrap()?;
        match mode {
            Mode::Immediate => Ok(val),
            Mode::Pointer => {
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
    fn read_unsigned(&self, modes: &mut Modes, pc: &mut u64) -> Result<u64, Error> {
        let val = self.read_signed(modes, pc)?;
        if val < 0 {
            bail!("Reading unsigned integer but found negative value {}.", val);
        }
        Ok(val as u64)
    }

    fn read_ptr(&self, pc: &mut u64) -> Result<u64, Error> {
        let val = self.read(*pc)?;
        if val < 0 {
            bail!(
                "Encountered negative pointer {}, which is not allowed.",
                val
            );
        }
        *pc += 1;
        Ok(val as u64)
    }
}

impl Memory for Vec<i64> {
    fn read(&self, ptr: u64) -> Result<i64, Error> {
        let value = *self.get(ptr as usize).ok_or_else(|| {
            error!(
                "Out of bounds error. Unable to read memory at location {}",
                ptr
            )
        })?;
        Ok(value)
    }

    fn write(&mut self, ptr: u64, val: i64) -> Result<(), Error> {
        let reference = self.get_mut(ptr as usize).ok_or_else(|| {
            error!(
                "Out of bounds error. Unable to write to memory location {}",
                ptr
            )
        })?;
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
    Halt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_02() {
        let test_cases = &[
            // (input, noun, verb, expected_ram)
            ("1,0,0,0,99", 0, 0, "2,0,0,0,99"),
            ("2,3,0,3,99", 3, 0, "2,3,0,6,99"),
            ("2,4,4,5,99,0", 4, 4, "2,4,4,5,99,9801"),
            ("1,1,1,4,99,5,6,0,99", 1, 1, "30,1,1,4,2,5,6,0,99"),
        ];

        for (input, noun, verb, expected_ram) in test_cases {
            let reader = io::BufReader::new(input.as_bytes());
            let mut computer = Computer::new(reader).unwrap();
            let _ = computer.execute(None, Some((*noun, *verb))).unwrap();
            let expected_ram = expected_ram
                .split(",")
                .map(|s| s.trim().parse::<i64>().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(computer.ram(), &expected_ram[..]);
        }
    }
}
