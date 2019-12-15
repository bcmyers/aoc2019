use std::io;

use crate::error::Error;

pub fn run<R>(reader: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    let _ = reader;
    Ok(("foo".to_string(), "bar".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils;

    #[test]
    #[ignore]
    fn test_14() {
        utils::tests::test_full_problem(14, run, "612880", "2509120");
    }
}
