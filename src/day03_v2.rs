use std::cmp;
use std::convert::TryFrom;
use std::io;
use std::ops::{Add, Deref, DerefMut};

use crate::error::Error;

const ORIGIN: Point = Point { x: 0, y: 0 };

pub fn run<R>(input: R) -> Result<(), Error>
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

    println!("{}", min_dist);
    println!("{}", min_steps);

    Ok(())
}

fn manhattan_distance(a: Point, b: Point) -> u64 {
    ((a.x - b.x).abs() + (a.y - b.y).abs()) as u64
}

fn parse_input<R>(mut reader: R) -> Result<[Path; 2], Error>
where
    R: io::BufRead,
{
    let mut buffer = String::new();
    let mut pass = 0;
    let mut paths = [Path(Vec::new()), Path(Vec::new())];

    loop {
        if reader.read_line(&mut buffer)? == 0 {
            break;
        }

        if pass > 1 {
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
            paths[pass].push(segment);
            origin = destination;
        }

        pass += 1;
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
