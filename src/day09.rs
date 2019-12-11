use std::io;

use crate::error::Error;

pub fn run<R>(input: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    let _ = input;
    Ok(("foo".to_string(), "bar".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils;

    #[test]
    #[ignore]
    fn test_09() {
        utils::tests::test_full_problem(9, run, "3460311188", "42202");
    }
}
