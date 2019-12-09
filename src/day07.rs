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
    use super::*;

    use crate::utils;

    #[test]
    #[ignore]
    fn test_07() {
        utils::tests::test_full_problem(7, run, "43812", "59597414");
    }
}
