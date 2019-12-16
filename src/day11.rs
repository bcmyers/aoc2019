use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::io;

use crossbeam::channel::{Receiver, Sender};
use crossbeam::thread;

use crate::computer::{Channel, Computer, Rom};
use crate::error::Error;
use crate::utils::Vec2;

type Point = Vec2<i64>;

pub fn run<R>(reader: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    let rom = Rom::from_reader(reader)?;

    // Part 1
    let robot = Robot::run(&rom, Color::Black)?;
    let answer1 = robot.grid.keys().count();

    // Part 2
    let robot = Robot::run(&rom, Color::White)?;
    let answer2 = robot.to_string();

    Ok((answer1.to_string(), answer2))
}

struct Robot {
    grid: HashMap<Point, Color>,
    location: Location,
}

impl Robot {
    fn run(rom: &Rom, color: Color) -> Result<Self, Error> {
        thread::scope(|s| {
            let mut robot = Self {
                grid: HashMap::default(),
                location: Location {
                    point: Point::new(0, 0),
                    direction: Direction::North,
                },
            };
            robot.grid.insert(robot.location.point, color);

            let (input, output) = (Channel::default(), Channel::default());
            let (sender, _) = input.clone().into_parts();
            let (_, receiver) = output.clone().into_parts();
            let handle = s.spawn(move |_| {
                let mut computer = Computer::with_io(input, output);
                computer.execute(rom, None)?;
                Ok::<_, Error>(())
            });

            loop {
                if robot.step(&sender, &receiver)?.is_none() {
                    break;
                }
            }

            handle.join().unwrap()?;
            Ok(robot)
        })
        .unwrap()
    }

    fn step(
        &mut self,
        sender: &Sender<i64>,
        receiver: &Receiver<i64>,
    ) -> Result<Option<()>, Error> {
        let color = *self.grid.get(&self.location.point).unwrap_or(&Color::Black);

        if sender.send(color as i64).is_err() {
            return Ok(None);
        };

        let new_color = match receiver.recv() {
            Ok(i) => Color::try_from(i)?,
            Err(_) => return Ok(None),
        };
        self.grid.insert(self.location.point, new_color);

        let turn = match receiver.recv() {
            Ok(i) => Turn::try_from(i)?,
            Err(_) => return Ok(None),
        };
        self.location = self.location.next(turn);

        Ok(Some(()))
    }
}

impl fmt::Display for Robot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let min_x = self.grid.keys().map(|p| p.x()).min().unwrap();
        let max_x = self.grid.keys().map(|p| p.x()).max().unwrap();
        let min_y = self.grid.keys().map(|p| p.y()).min().unwrap();
        let max_y = self.grid.keys().map(|p| p.y()).max().unwrap();

        let rows = (max_y - min_y) as usize + 1;
        let cols = (max_x - min_x) as usize + 1;

        let len = rows * (cols + 1);

        let mut buf = vec![' ' as u8; len];
        for point in self
            .grid
            .iter()
            .filter(|(_, color)| **color == Color::White)
            .map(|(point, _)| point)
        {
            let x = (point.x() - min_x) as usize;
            let y = (point.y() - min_y) as usize;
            buf[(rows - 1 - y) * cols + x] = '#' as u8;
        }

        for row in 0..(rows - 1) {
            buf[row * cols + cols - 1] = '\n' as u8;
        }

        let s = String::from_utf8(buf).unwrap();
        write!(f, "{}", s)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
enum Color {
    Black = 0,
    White = 1,
}

impl TryFrom<i64> for Color {
    type Error = Error;

    fn try_from(i: i64) -> Result<Self, Self::Error> {
        let color = match i {
            0 => Color::Black,
            1 => Color::White,
            _ => bail!("Cannot parse {} into a Color.", i),
        };
        Ok(color)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn next(&self, turn: Turn) -> Direction {
        use self::Direction::*;
        use self::Turn::*;
        match (self, turn) {
            (North, Left) => West,
            (North, Right) => East,
            (South, Left) => East,
            (South, Right) => West,
            (East, Left) => North,
            (East, Right) => South,
            (West, Left) => South,
            (West, Right) => North,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct Location {
    point: Point,
    direction: Direction,
}

impl Location {
    fn next(&self, turn: Turn) -> Location {
        use self::Direction::*;
        let (x, y) = (self.point.x(), self.point.y());
        let next_direction = self.direction.next(turn);
        let next_point = match next_direction {
            North => (x, y + 1),
            South => (x, y - 1),
            East => (x + 1, y),
            West => (x - 1, y),
        }
        .into();
        Self {
            direction: next_direction,
            point: next_point,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
enum Turn {
    Left = 0,
    Right = 1,
}

impl TryFrom<i64> for Turn {
    type Error = Error;

    fn try_from(i: i64) -> Result<Self, Self::Error> {
        let color = match i {
            0 => Turn::Left,
            1 => Turn::Right,
            _ => bail!("Cannot parse {} into a Turn.", i),
        };
        Ok(color)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_11() {
        let file = fs::File::open("data/11.txt").unwrap();
        let reader = io::BufReader::new(file);
        let rom = Rom::from_reader(reader).unwrap();
        let robot = Robot::run(&rom, Color::Black).unwrap();
        let actual = robot.grid.keys().count();
        assert_eq!(actual, 2293);
    }
}
