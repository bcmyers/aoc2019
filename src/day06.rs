use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::io;
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};

use crate::error::Error;

type Ids = HashMap<String, usize>;

pub fn run<R>(input: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    let (directed, undirected, ids) = parse_input(input)?;
    let id_com = *ids.get("COM").ok_or_else(|| error!("COM node missing!"))?;
    let nconnections = directed.nconnections(&id_com);

    let id_you = *ids.get("YOU").ok_or_else(|| error!("YOU node missing!"))?;
    let id_san = *ids.get("SAN").ok_or_else(|| error!("SAN node missing!"))?;
    let shortest_distance = undirected
        .shortest_distance(&id_you, &id_san)
        .ok_or_else(|| error!("Could not find a path from us to Santa :( :("))?;
    let answer2 = shortest_distance - 2;

    Ok((nconnections.to_string(), answer2.to_string()))
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Graph<N>(HashMap<N, Vec<N>>)
where
    N: Eq + Hash;

impl<N> Deref for Graph<N>
where
    N: Eq + Hash,
{
    type Target = HashMap<N, Vec<N>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<N> DerefMut for Graph<N>
where
    N: Eq + Hash,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<N> FromIterator<(N, Vec<N>)> for Graph<N>
where
    N: Eq + Hash,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (N, Vec<N>)>,
    {
        Graph(HashMap::from_iter(iter))
    }
}

impl<N> Graph<N>
where
    N: Eq + Hash,
{
    fn shortest_distance(&self, a: &N, b: &N) -> Option<usize> {
        let mut levels: HashMap<&N, usize> = HashMap::new();
        let mut queue: VecDeque<&N> = VecDeque::new();
        let mut visited: HashSet<&N> = HashSet::new();

        levels.insert(a, 0);
        queue.push_back(a);
        visited.insert(a);

        while let Some(ref node) = queue.pop_front() {
            if let Some(children) = self.0.get(node) {
                for child in children {
                    if !visited.contains(child) {
                        let level = levels.get(node).unwrap() + 1;
                        if child == b {
                            return Some(level);
                        }

                        levels.insert(child, level);

                        visited.insert(child);
                        queue.push_back(child);
                    }
                }
            }
        }
        None
    }

    fn nconnections(&self, start: &N) -> usize {
        let mut nconnections = 0;

        let mut levels = HashMap::new();
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        levels.insert(start, 0);
        queue.push_back(start);
        visited.insert(start);

        while let Some(ref node) = queue.pop_front() {
            if let Some(children) = self.0.get(node) {
                for child in children {
                    if !visited.contains(child) {
                        let level = levels.get(node).unwrap() + 1;
                        levels.insert(child, level);

                        nconnections += level;

                        visited.insert(child);
                        queue.push_back(child);
                    }
                }
            }
        }

        nconnections
    }
}

fn parse_input<R>(mut reader: R) -> Result<(Graph<usize>, Graph<usize>, Ids), Error>
where
    R: io::BufRead,
{
    let mut directed = Graph::default();
    let mut undirected = Graph::default();

    let mut buffer = String::new();
    let mut id = 0;
    let mut ids: HashMap<String, usize> = HashMap::new();
    loop {
        if reader.read_line(&mut buffer)? == 0 {
            break;
        }

        let line = buffer.trim();
        let mut iter = line.split(")").map(|s| s.trim().to_string());
        let parent = iter
            .next()
            .ok_or_else(|| error!("Unable to parse input line {}", line))?;
        let child = iter
            .next()
            .ok_or_else(|| error!("Unable to parse input line {}", line))?;

        let id_parent = *ids.entry(parent).or_insert_with(|| id);
        let id_child = *ids.entry(child).or_insert_with(|| id + 1);
        id += 2;

        directed
            .entry(id_parent)
            .or_insert_with(|| Vec::new())
            .push(id_child);

        undirected
            .entry(id_parent)
            .or_insert_with(|| Vec::new())
            .push(id_child);
        undirected
            .entry(id_child)
            .or_insert_with(|| Vec::new())
            .push(id_parent);

        buffer.clear();
    }

    Ok((directed, undirected, ids))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_06() {
        {
            // Part 2
            let input = "COM)B\nB)C\nC)D\nD)E\nE)F\nB)G\nG)H\nD)I\nE)J\nJ)K\nK)L";
            let reader = io::BufReader::new(input.as_bytes());
            let (directed, _, ids) = parse_input(reader).unwrap();

            let id_com = ids.get("COM").unwrap();
            let nconnections = directed.nconnections(id_com);
            assert_eq!(nconnections, 42);
        }

        {
            // Part 2
            let input = "COM)B\nB)C\nC)D\nD)E\nE)F\nB)G\nG)H\nD)I\nE)J\nJ)K\nK)L\nK)YOU\nI)SAN\n";
            let reader = io::BufReader::new(input.as_bytes());
            let (_, undirected, ids) = parse_input(reader).unwrap();

            let id_you = ids.get("YOU").unwrap();
            let id_san = ids.get("SAN").unwrap();
            let dist = undirected.shortest_distance(id_you, id_san).unwrap();
            assert_eq!(dist, 6);
        }
    }
}
