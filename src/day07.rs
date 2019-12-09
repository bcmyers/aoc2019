use std::io;

use crate::error::Error;

pub fn run<R>(_input: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    Ok(("foo".to_string(), "bar".to_string()))
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_07() {}
}
