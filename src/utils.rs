use std::convert::TryFrom;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

use crate::error::Error;

pub(crate) mod math {
    use super::*;

    pub(crate) fn fact(mut n: usize) -> Result<usize, Error> {
        let orig = n;
        let mut answer = 1usize;
        loop {
            answer = match answer.checked_mul(n) {
                Some(val) => val,
                None => bail!("Factorial of {} overflows usize.", orig),
            };
            if (n - 1) == 1 {
                break;
            } else {
                n -= 1;
            }
        }
        Ok(answer)
    }

    pub(crate) fn gcf(a: u64, b: u64) -> Result<u64, Error> {
        if a == 0 || b == 0 {
            bail!("gcf function only works with positive inputs.");
        }
        let (mut smaller, mut larger) = if a > b { (b, a) } else { (a, b) };
        loop {
            let rem = larger % smaller;
            if rem == 0 {
                return Ok(smaller);
            }
            larger = smaller;
            smaller = rem;
        }
    }

    pub(crate) fn lcm(a: u64, b: u64) -> Result<u64, Error> {
        Ok(a * b / gcf(a, b)?)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_gcf() {
            assert_eq!(5, gcf(5, 5).unwrap());
            assert_eq!(5, gcf(5, 10).unwrap());
            assert_eq!(3, gcf(15, 21).unwrap());
            assert!(gcf(1, 0).is_err());
        }
    }
}

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

// TODO: Verify that this is kosher.
#[allow(clippy::derive_hash_xor_eq)]
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

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
mod simd {
    use std::mem;

    use super::*;

    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    impl From<__m256i> for Vec3<i64> {
        fn from(v: __m256i) -> Self {
            // safety: This is safe because the call to _mm256_storeu_si256 will write
            // values to the uninitialized array so we won't be tying to access junk memory.
            let mut a: [i64; 4] = unsafe { mem::MaybeUninit::uninit().assume_init() };
            #[allow(clippy::cast_ptr_alignment)]
            unsafe {
                _mm256_storeu_si256(&mut a as *mut _ as *mut __m256i, v)
            };
            Vec3::new(a[3], a[2], a[1])
        }
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
