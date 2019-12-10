use std::io;

use crate::error::Error;

pub fn run<R>(reader: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    let _ = reader;
    Ok(("foo".to_string(), "bar".to_string()))
}
