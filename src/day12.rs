use std::cell::RefCell;
use std::io;
use std::mem;
use std::ops::{Deref, DerefMut};

use crate::error::Error;
use crate::utils::Vec3;

#[cfg(not(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
)))]
use self::normal::Moon;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
use self::simd::Moon;

pub fn run<R>(reader: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    let mut moons = parse_input(reader)?;
    for _ in 0..1_000 {
        moons.step();
    }
    let answer1 = moons.energy();
    Ok((answer1.to_string(), "TODO".to_string()))
}

fn parse_input<R>(reader: R) -> Result<Moons, Error>
where
    R: io::BufRead,
{
    // safety: TODO
    let mut moons: [RefCell<Moon>; 4] = unsafe { mem::MaybeUninit::uninit().assume_init() };
    let mut i = 0;
    for res in reader.lines() {
        if i > 3 {
            bail!("Can only support exactly 4 moons.");
        }
        let line = res?;
        let line = line.trim();
        let mut pos: [i64; 3] = [0i64; 3];
        let mut j = 0;
        for part in line.split(',') {
            let coord = part
                .split('=')
                .nth(1)
                .ok_or_else(|| error!("TODO"))?
                .chars()
                .take_while(|&c| c != '>')
                .collect::<String>()
                .parse::<i64>()?;
            pos[j] = coord;
            j += 1;
        }
        if j != 3 {
            bail!("Found too many coordinates.");
        }
        let moon = Moon::new(pos, Vec3::default());
        moons[i] = RefCell::new(moon);
        i += 1;
    }
    if i != 4 {
        bail!("Can only support exactly 4 moons");
    }

    Ok(Moons(moons))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Moons([RefCell<Moon>; 4]);

impl Moons {
    fn energy(&self) -> u64 {
        let mut total = 0;
        for moon in self.iter() {
            let moon = moon.borrow();
            let mut potential = 0;
            let mut kinetic = 0;
            for k in 0..3 {
                potential += moon.pos()[k].abs() as u64;
                kinetic += moon.vel()[k].abs() as u64;
            }
            total += potential * kinetic;
        }
        total
    }
}

impl Deref for Moons {
    type Target = [RefCell<Moon>];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Moons {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(not(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
)))]
mod normal {
    use super::*;

    impl Moons {
        pub(crate) fn step(&mut self) {
            for moon_i in &self.0 {
                for moon_j in &self.0 {
                    if moon_i == moon_j {
                        continue;
                    }
                    for k in 0..3 {
                        let pos_i = moon_i.borrow().pos()[k];
                        let pos_j = moon_j.borrow().pos()[k];
                        if pos_i < pos_j {
                            moon_i.borrow_mut().vel_mut()[k] += 1;
                        } else if pos_i > pos_j {
                            moon_i.borrow_mut().vel_mut()[k] -= 1;
                        }
                    }
                }
            }
            for moon in self.iter_mut() {
                for k in 0..3 {
                    let vel = { moon.borrow().vel()[k] };
                    moon.borrow_mut().pos_mut()[k] += vel;
                }
            }
        }
    }

    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
    pub(crate) struct Moon {
        pos: Vec3<i64>,
        vel: Vec3<i64>,
    }

    impl Moon {
        pub(crate) fn new<V, U>(pos: V, vel: U) -> Self
        where
            V: Into<Vec3<i64>>,
            U: Into<Vec3<i64>>,
        {
            Self {
                pos: pos.into(),
                vel: vel.into(),
            }
        }

        pub(crate) fn pos(&self) -> &Vec3<i64> {
            &self.pos
        }

        pub(crate) fn pos_mut(&mut self) -> &mut Vec3<i64> {
            &mut self.pos
        }

        pub(crate) fn vel(&self) -> &Vec3<i64> {
            &self.vel
        }

        pub(crate) fn vel_mut(&mut self) -> &mut Vec3<i64> {
            &mut self.vel
        }
    }
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
mod simd {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    use lazy_static::lazy_static;

    use super::*;

    lazy_static! {
        static ref ONE: __m256i = unsafe { _mm256_set_epi64x(1, 1, 1, 0) };
        static ref NEGATIVE_ONE: __m256i = unsafe { _mm256_set_epi64x(-1, -1, -1, 0) };
    }

    impl Moons {
        pub(crate) fn step(&mut self) {
            for moon_i in &self.0 {
                for moon_j in &self.0 {
                    if moon_i == moon_j {
                        continue;
                    }

                    let pos_i = moon_i.borrow().pos;
                    let pos_j = moon_j.borrow().pos;

                    // Adding
                    let mask_gt = unsafe { _mm256_cmpgt_epi64(pos_i, pos_j) };
                    let operand_add = unsafe { _mm256_and_si256(mask_gt, *NEGATIVE_ONE) };

                    // Subtracting
                    let mask_lt = unsafe { _mm256_cmpgt_epi64(pos_j, pos_i) };
                    let operand_sub = unsafe { _mm256_and_si256(mask_lt, *ONE) };

                    let operand = unsafe { _mm256_or_si256(operand_add, operand_sub) };

                    let mut moon_ref = moon_i.borrow_mut();
                    let vel_ref = moon_ref.vel_mut();
                    *vel_ref = unsafe { _mm256_add_epi64(*vel_ref, operand) };
                }
            }
            for moon in self.iter_mut() {
                let new_pos = {
                    let moon = moon.borrow();
                    unsafe { _mm256_add_epi64(moon.pos, moon.vel) }
                };
                let mut moon = moon.borrow_mut();
                *moon.pos_mut() = new_pos;
            }
        }
    }

    #[derive(Copy, Clone, Debug)]
    pub(crate) struct Moon {
        pos: __m256i,
        vel: __m256i,
    }

    impl PartialEq for Moon {
        fn eq(&self, other: &Moon) -> bool {
            self.pos() == other.pos() && self.vel() == other.vel()
        }
    }

    impl Eq for Moon {}

    impl Moon {
        pub(crate) fn new<V, U>(pos: V, vel: U) -> Self
        where
            V: Into<Vec3<i64>>,
            U: Into<Vec3<i64>>,
        {
            let pos = {
                let pos = pos.into();
                unsafe { _mm256_set_epi64x(pos.x(), pos.y(), pos.z(), 0) }
            };
            let vel = {
                let vel = vel.into();
                unsafe { _mm256_set_epi64x(vel.x(), vel.y(), vel.z(), 0) }
            };
            Self { pos, vel }
        }

        pub(crate) fn pos(&self) -> Vec3<i64> {
            let mut a: [i64; 4] = unsafe { mem::MaybeUninit::uninit().assume_init() };
            unsafe { _mm256_storeu_si256(&mut a as *mut _ as *mut _, self.pos) };
            Vec3::new(a[3], a[2], a[1])
        }

        pub(crate) fn pos_mut(&mut self) -> &mut __m256i {
            &mut self.pos
        }

        pub(crate) fn vel(&self) -> Vec3<i64> {
            let mut a: [i64; 4] = unsafe { mem::MaybeUninit::uninit().assume_init() };
            unsafe { _mm256_storeu_si256(&mut a as *mut _ as *mut _, self.vel) };
            Vec3::new(a[3], a[2], a[1])
        }

        pub(crate) fn vel_mut(&mut self) -> &mut __m256i {
            &mut self.vel
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils;

    #[test]
    fn test_12() {
        utils::tests::test_full_problem(12, run, "7722", "TODO");
    }
}
