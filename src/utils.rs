use std::convert::TryFrom;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

use crate::error::Error;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct F64(f64);

impl TryFrom<f64> for F64 {
    type Error = Error;

    fn try_from(f: f64) -> Result<Self, Self::Error> {
        if f.is_nan() {
            bail!("Cannot convert {} into F64", f);
        }
        Ok(F64(f))
    }
}

impl Deref for F64 {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Eq for F64 {}

impl Hash for F64 {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.0.to_bits().hash(state);
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct Vec2<T>(T, T);

impl<T> Vec2<T> {
    pub(crate) const fn new(x: T, y: T) -> Self {
        Self(x, y)
    }
}

impl<T> Vec2<T>
where
    T: Copy,
{
    pub(crate) fn x(&self) -> T {
        self.0
    }

    pub(crate) fn y(&self) -> T {
        self.1
    }
}

impl<T> From<(T, T)> for Vec2<T> {
    fn from(tup: (T, T)) -> Self {
        Self(tup.0, tup.1)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct Vec3<T>([T; 3]);

impl<T> Default for Vec3<T>
where
    T: Copy + Default,
{
    fn default() -> Self {
        Self([T::default(); 3])
    }
}

impl<T> Vec3<T> {
    #[allow(unused)]
    pub(crate) const fn new(x: T, y: T, z: T) -> Self {
        Self([x, y, z])
    }
}

impl<T> Vec3<T>
where
    T: Copy,
{
    #[allow(unused)]
    pub(crate) fn x(&self) -> T {
        self.0[0]
    }

    #[allow(unused)]
    pub(crate) fn y(&self) -> T {
        self.0[1]
    }

    #[allow(unused)]
    pub(crate) fn z(&self) -> T {
        self.0[2]
    }
}

impl<T> Deref for Vec3<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Vec3<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<(T, T, T)> for Vec3<T> {
    fn from(tup: (T, T, T)) -> Self {
        Self([tup.0, tup.1, tup.2])
    }
}

impl<T> From<[T; 3]> for Vec3<T> {
    fn from(array: [T; 3]) -> Self {
        Self(array)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::fs;
    use std::io;

    use crate::error::Error;

    pub(crate) fn test_full_problem<F>(day: usize, run_func: F, expected1: &str, expected2: &str)
    where
        F: Fn(io::BufReader<fs::File>) -> Result<(String, String), Error>,
    {
        let path = format!("data/{:02}.txt", day);
        let file = std::fs::File::open(path).unwrap();
        let reader = std::io::BufReader::new(file);
        let (actual1, actual2) = run_func(reader).unwrap();
        assert_eq!(&actual1, expected1);
        assert_eq!(&actual2, expected2);
    }
}
