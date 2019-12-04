use std::cmp;
use std::convert::TryFrom;
use std::io;
use std::ops::{Add, Deref, DerefMut};

use crate::error::Error;

const ORIGIN: Point = Point { x: 0, y: 0 };

pub fn run<R>(input: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    let paths = parse_input(input)?;

    let mut intersections = Vec::new();
    for segment0 in paths[0].iter() {
        for segment1 in paths[1].iter() {
            if let Some(intersection) = segment0.intersection(segment1) {
                intersections.push(intersection);
            }
        }
    }

    if intersections.is_empty() {
        bail!("Unable to find any intersections.")
    }

    let (mut min_dist, mut min_steps) = (std::u64::MAX, std::u64::MAX);
    for intersection in intersections {
        let point = intersection.point;
        let steps = intersection.steps;

        // Part 1
        let dist = manhattan_distance(point, ORIGIN);
        if dist < min_dist {
            min_dist = dist;
        }

        // Part 2
        if steps < min_steps {
            min_steps = steps;
        }
    }

    Ok((format!("{}", min_dist), format!("{}", min_steps)))
}

fn manhattan_distance(a: Point, b: Point) -> u64 {
    ((a.x - b.x).abs() + (a.y - b.y).abs()) as u64
}

fn parse_input<R>(mut reader: R) -> Result<[Path; 2], Error>
where
    R: io::BufRead,
{
    let mut buffer = String::new();
    let mut id = 0;
    let mut paths = [Path(Vec::new()), Path(Vec::new())];

    loop {
        if reader.read_line(&mut buffer)? == 0 {
            break;
        }

        if id > 1 {
            bail!("Invalid input. Input must be comprise of exactly 2 paths.")
        }

        let mut origin = Point { x: 0, y: 0 };
        let mut steps = 0;

        for s in buffer.trim().split(",").map(|s| s.trim()) {
            let instruction = {
                let bytes = s.as_bytes();
                let c = bytes[0] as char;
                let dir = Direction::try_from(c)?;
                let dist = atoi::atoi::<u64>(&bytes[1..])
                    .ok_or_else(|| error!("Unable to parse {} into an instruction.", s))?;
                Instruction { dir, dist }
            };
            let destination = origin + instruction;
            let segment = Segment::new(steps, origin, destination)?;
            steps += instruction.dist;
            paths[id].push(segment);
            origin = destination;
        }

        id += 1;
        buffer.clear();
    }

    Ok(paths)
}

#[derive(Copy, Clone, Debug)]
enum Direction {
    U,
    D,
    L,
    R,
}

impl TryFrom<char> for Direction {
    type Error = Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        let d = match c {
            'U' => Self::U,
            'D' => Self::D,
            'L' => Self::L,
            'R' => Self::R,
            _ => bail!("Invalid direction {}.", c),
        };
        Ok(d)
    }
}

#[derive(Copy, Clone, Debug)]
struct Instruction {
    dir: Direction,
    dist: u64,
}

impl Add<Instruction> for Point {
    type Output = Point;

    fn add(self, rhs: Instruction) -> Self::Output {
        match rhs.dir {
            Direction::U => Point {
                x: self.x,
                y: self.y + rhs.dist as i64,
            },
            Direction::D => Point {
                x: self.x,
                y: self.y - rhs.dist as i64,
            },
            Direction::L => Point {
                x: self.x - rhs.dist as i64,
                y: self.y,
            },
            Direction::R => Point {
                x: self.x + rhs.dist as i64,
                y: self.y,
            },
        }
    }
}

struct Intersection {
    point: Point,
    steps: u64,
}

#[derive(Copy, Clone, Debug)]
pub struct Segment {
    o: Point,
    d: Point,
    steps: u64,
}

#[derive(Copy, Clone, Debug)]
pub enum SegmentKind {
    Horizontal { x: (i64, i64), y: i64 },
    Vertical { x: i64, y: (i64, i64) },
}

impl Segment {
    fn new(steps: u64, o: Point, d: Point) -> Result<Self, Error> {
        if o.x != d.x && o.y != d.y {
            bail!("Invalid line segment from {:?} to {:?}. Line segments must be horizontal or vertical.", o, d);
        }
        Ok(Self { o, d, steps })
    }

    fn steps_to(&self, p: Point) -> u64 {
        let new_steps = manhattan_distance(self.o, p);
        self.steps + new_steps
    }

    fn intersection(&self, other: &Segment) -> Option<Intersection> {
        match self.kind() {
            SegmentKind::Horizontal { x, y } => {
                let (xh, yh) = (x, y);
                match other.kind() {
                    SegmentKind::Vertical { x, y } => {
                        let (xv, yv) = (x, y);
                        if (xh.0..=xh.1).contains(&xv) {
                            if (yv.0..=yv.1).contains(&yh) {
                                let point = Point { x: xv, y: yh };
                                if point != ORIGIN {
                                    let steps = self.steps_to(point) + other.steps_to(point);
                                    return Some(Intersection { point, steps });
                                }
                            }
                        }
                        None
                    }
                    _ => None,
                }
            }
            SegmentKind::Vertical { x, y } => {
                let (xv, yv) = (x, y);
                match other.kind() {
                    SegmentKind::Horizontal { x, y } => {
                        let (xh, yh) = (x, y);
                        if (xh.0..=xh.1).contains(&xv) {
                            if (yv.0..=yv.1).contains(&yh) {
                                let point = Point { x: xv, y: yh };
                                if point != ORIGIN {
                                    let steps = self.steps_to(point) + other.steps_to(point);
                                    return Some(Intersection { point, steps });
                                }
                            }
                        }
                        None
                    }
                    _ => None,
                }
            }
        }
    }

    fn kind(&self) -> SegmentKind {
        if self.o.x == self.d.x {
            SegmentKind::Vertical {
                x: self.o.x,
                y: (cmp::min(self.o.y, self.d.y), cmp::max(self.o.y, self.d.y)),
            }
        } else if self.o.y == self.d.y {
            SegmentKind::Horizontal {
                x: (cmp::min(self.o.x, self.d.x), cmp::max(self.o.x, self.d.x)),
                y: self.o.y,
            }
        } else {
            unreachable!()
        }
    }
}

#[derive(Clone, Debug)]
struct Path(Vec<Segment>);

impl Deref for Path {
    type Target = Vec<Segment>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Path {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct Point {
    x: i64,
    y: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_03() {
        let test_cases = &[
            // (input, expected1, expected2)
            ("R8,U5,L5,D3\nU7,R6,D4,L4", "6", "30"),
            (
                "R75,D30,R83,U83,L12,D49,R71,U7,L72\nU62,R66,U55,R34,D71,R55,D58,R83",
                "159",
                "610",
            ),
            (
                "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51\nU98,R91,D20,R16,D67,R40,U7,R15,U6,R7",
                "135",
                "410",
            ),
        ];

        for (input, expected1, expected2) in test_cases {
            let reader = io::BufReader::new(input.as_bytes());
            let (actual1, actual2) = run(reader).unwrap();
            assert_eq!(&actual1, expected1);
            assert_eq!(&actual2, expected2);
        }
    }
}
