use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::TryFrom;
use std::io;

use crate::error::Error;
use crate::utils::{Point, F64};

pub fn run<R>(reader: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    // Parse input
    let points = parse_input(reader)?;

    // Part 1
    let (answer1, laser) = part1(&points)?;

    // Part 2
    let mut asteroids = part2(laser, &points)?;
    let asteroid = asteroids
        .nth(199)
        .ok_or_else(|| error!("Could not find 200th asteroid"))?;
    let answer2 = asteroid.point.x() * 100 + asteroid.point.y();

    Ok((answer1.to_string(), answer2.to_string()))
}

fn parse_input<R>(mut reader: R) -> Result<Vec<Point>, Error>
where
    R: io::BufRead,
{
    let mut points = Vec::new();
    let mut buf = String::new();
    let mut y = 0;
    loop {
        if reader.read_line(&mut buf)? == 0 {
            break;
        }

        buf.trim().chars().enumerate().for_each(|(x, c)| {
            if c == '#' {
                let point = Point::new(x as i64, y as i64);
                points.push(point);
            }
        });

        y += 1;
        buf.clear();
    }

    Ok(points)
}

fn part1(points: &[Point]) -> Result<(usize, Point), Error> {
    let mut map: HashMap<Point, HashSet<Direction>> = HashMap::new();
    for origin in points {
        for other in points {
            let direction = Direction::new(*origin, *other)?;
            map.entry(*origin)
                .or_insert_with(|| HashSet::new())
                .insert(direction);
        }
    }

    let (max, point) = map.iter().fold((0, None), |mut state, (point, set)| {
        let count = set.len();
        if count > state.0 {
            state = (count, Some(point));
        }
        state
    });

    let point = *point.ok_or_else(|| error!("TODO"))?;

    Ok((max, point))
}

fn part2(laser: Point, points: &[Point]) -> Result<Asteroids, Error> {
    let vec = points
        .iter()
        .filter(|point| **point != laser)
        .map(|point| Asteroid::new(laser, *point))
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(Asteroids::new(vec))
}

fn polar_coordinates_transformation(angle: f64) -> f64 {
    match angle + std::f64::consts::FRAC_PI_2 {
        f if f < 0.0 => f + 2.0 * std::f64::consts::PI,
        f => f,
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Asteroid {
    point: Point,
    angle: F64,
    distance_squared: u64,
}

impl Asteroid {
    fn new(laser: Point, point: Point) -> Result<Self, Error> {
        let (x, y) = (point.x() - laser.x(), point.y() - laser.y());
        let angle = (y as f64).atan2(x as f64);
        let angle = polar_coordinates_transformation(angle);
        let distance_squared = (x * x + y * y) as u64;
        Ok(Self {
            point,
            angle: F64::try_from(angle)?,
            distance_squared,
        })
    }
}

impl PartialOrd for Asteroid {
    fn partial_cmp(&self, other: &Asteroid) -> Option<Ordering> {
        match self.angle.partial_cmp(&other.angle) {
            Some(Ordering::Less) => Some(Ordering::Less),
            Some(Ordering::Greater) => Some(Ordering::Greater),
            Some(Ordering::Equal) => Some(self.distance_squared.cmp(&other.distance_squared)),
            None => unreachable!(),
        }
    }
}

impl Ord for Asteroid {
    fn cmp(&self, other: &Asteroid) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

struct Asteroids {
    previous_angle: Option<F64>,
    queue: VecDeque<Asteroid>,
}

impl Asteroids {
    fn new(mut vec: Vec<Asteroid>) -> Self {
        vec.sort_unstable();
        Self {
            previous_angle: None,
            queue: VecDeque::from(vec),
        }
    }
}

impl Iterator for Asteroids {
    type Item = Asteroid;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let asteroid = self.queue.pop_front()?;
            if let Some(previous_angle) = self.previous_angle {
                if asteroid.angle == previous_angle {
                    self.queue.push_back(asteroid);
                    continue;
                }
            }
            self.previous_angle = Some(asteroid.angle);
            return Some(asteroid);
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct Direction(F64);

impl Direction {
    fn new(origin: Point, other: Point) -> Result<Self, Error> {
        let (x, y) = (other.x() - origin.x(), other.y() - origin.y());
        let angle = (y as f64).atan2(x as f64);
        Ok(Direction(F64::try_from(angle)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils;

    #[test]
    fn test_angle() {
        use std::f64::consts::*;
        let test_cases = &[
            ((0i64, -1i64), 0.0),
            ((1, -1), 1.0 * FRAC_PI_4),
            ((1, 0), 2.0 * FRAC_PI_4),
            ((1, 1), 3.0 * FRAC_PI_4),
            ((0, 1), 4.0 * FRAC_PI_4),
            ((-1, 1), 5.0 * FRAC_PI_4),
            ((-1, 0), 6.0 * FRAC_PI_4),
            ((-1, -1), 7.0 * FRAC_PI_4),
        ];

        for ((x, y), expected) in test_cases {
            let angle = (*y as f64).atan2(*x as f64);
            let actual = polar_coordinates_transformation(angle);
            assert_eq!(actual, *expected);
        }
    }

    #[test]
    fn test_10() {
        utils::tests::test_full_problem(10, run, "260", "608");
    }
}
