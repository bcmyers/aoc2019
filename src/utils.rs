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
