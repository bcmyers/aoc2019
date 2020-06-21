use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::{TryFrom, TryInto};
use std::io;

use crate::computer::{ComputerST, Queue, Rom, State};
use crate::error::Error;
use crate::utils::Vec2;

type Point = Vec2<i64>;

const ORIGIN: Point = Point::new(0, 0);

pub fn run<R>(reader: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    let rom = Rom::from_reader(reader)?;
    let mut droid = Droid::new(rom);
    let (oxygen, answer1) = droid.run()?;
    let answer2 = largest_layer(oxygen, &droid.graph);

    Ok((answer1.to_string(), answer2.to_string()))
}

fn largest_layer(start: Point, graph: &HashMap<Point, HashSet<Point>>) -> usize {
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut layers = HashMap::new();

    queue.push_back(start);
    visited.insert(start);
    layers.insert(start, 0usize);

    while let Some(parent) = queue.pop_front() {
        visited.insert(parent);

        for neighbor in graph.get(&parent).unwrap() {
            if !visited.contains(neighbor) {
                let layer = *layers.get(&parent).unwrap() + 1;
                layers.insert(*neighbor, layer);
                queue.push_back(*neighbor);
            }
        }
    }
    *layers.values().max().unwrap()
}

struct Droid {
    computer: ComputerST,

    queue: VecDeque<(Point, ComputerST)>,
    visited: HashSet<Point>,
    layers: HashMap<Point, usize>,
    graph: HashMap<Point, HashSet<Point>>,
}

impl Droid {
    fn new<R>(rom: R) -> Self
    where
        R: AsRef<[i64]>,
    {
        let computer = ComputerST::new(rom);
        let queue = {
            let mut queue = VecDeque::new();
            queue.push_back((ORIGIN, computer.clone()));
            queue
        };
        let visited = {
            let mut visited = HashSet::new();
            visited.insert(ORIGIN);
            visited
        };
        let layers = {
            let mut map = HashMap::new();
            map.insert(ORIGIN, 0);
            map
        };
        Self {
            computer,
            queue,
            visited,
            layers,
            graph: HashMap::new(),
        }
    }

    fn run(&mut self) -> Result<(Point, usize), Error> {
        let mut output = None;
        while let Some((parent, computer)) = self.queue.pop_front() {
            self.visited.insert(parent);
            self.computer = computer;
            for (point, command) in surrounding_points(parent).iter() {
                if !self.visited.contains(point) {
                    self.computer.input_mut().enqueue(*command as i64);
                    if self.computer.step()? != State::HasOutput {
                        bail!("Invalid computer program. Expected output.");
                    };
                    let response: Response = self.computer.output_mut().dequeue()?.try_into()?;
                    match response {
                        Response::Wall => {}
                        Response::Move | Response::Oxygen => {
                            self.graph
                                .entry(parent)
                                .or_insert_with(HashSet::new)
                                .insert(*point);
                            self.graph
                                .entry(*point)
                                .or_insert_with(HashSet::new)
                                .insert(parent);

                            let layer = self.layers.get(&parent).unwrap() + 1;
                            self.layers.insert(*point, layer);
                            self.queue.push_back((*point, self.computer.clone()));
                            if response == Response::Oxygen {
                                output = Some((*point, layer));
                            }
                            self.computer.input_mut().enqueue(command.opposite() as i64);
                            if self.computer.step()? != State::HasOutput {
                                bail!("Invalid computer program. Expected output.");
                            };
                            let response: Response =
                                self.computer.output_mut().dequeue()?.try_into()?;
                            assert_eq!(response, Response::Move);
                        }
                    }
                }
            }
        }
        output.ok_or_else(|| error!("Could not find oxygen"))
    }
}

fn surrounding_points(point: Point) -> [(Point, Command); 4] {
    [
        (Point::new(point.x(), point.y() + 1), Command::North),
        (Point::new(point.x(), point.y() - 1), Command::South),
        (Point::new(point.x() - 1, point.y()), Command::West),
        (Point::new(point.x() + 1, point.y()), Command::East),
    ]
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i64)]
enum Command {
    North = 1,
    South = 2,
    West = 3,
    East = 4,
}

impl Command {
    fn opposite(self) -> Command {
        use self::Command::*;
        match self {
            North => South,
            South => North,
            West => East,
            East => West,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i64)]
enum Response {
    Wall = 0,
    Move = 1,
    Oxygen = 2,
}

impl TryFrom<i64> for Response {
    type Error = Error;

    fn try_from(i: i64) -> Result<Self, Self::Error> {
        use self::Response::*;
        let output = match i {
            0 => Wall,
            1 => Move,
            2 => Oxygen,
            _ => bail!("Could not parse {} into a Response.", i),
        };
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils;

    #[test]
    fn test_15() {
        utils::tests::test_full_problem(15, run, "248", "382");
    }
}
