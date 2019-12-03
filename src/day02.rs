use std::io;

use crate::error::Error;

pub fn run<R>(_input: R) -> Result<(), Error>
where
    R: io::BufRead,
{
    unimplemented!()
}
