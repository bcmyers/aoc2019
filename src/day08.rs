use std::io;

use crate::error::Error;

const ROWS: usize = 6;
const COLS: usize = 25;

pub fn run<R>(mut reader: R) -> Result<(String, String), Error>
where
    R: io::BufRead,
{
    // Parse input
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    buf.pop();
    buf.iter_mut().for_each(|b| *b -= 48);

    // Part 1
    let answer1 = match buf
        .chunks(ROWS * COLS)
        .fold((std::usize::MAX, None), |mut state, layer| {
            let nzeros = bytecount::count(layer, 0);
            if nzeros < state.0 {
                state = (nzeros, Some(layer));
            }
            state
        }) {
        (_, Some(layer)) => {
            let nones = bytecount::count(layer, 1);
            let ntwos = bytecount::count(layer, 2);
            nones * ntwos
        }
        (_, None) => bail!("Error"),
    };

    // Part 2
    let image = buf
        .chunks(ROWS * COLS)
        .fold([2u8; ROWS * COLS], |mut state, layer| {
            state.iter_mut().enumerate().for_each(|(i, b)| {
                if *b == 2 {
                    *b = layer[i];
                }
            });
            state
        });

    let mut iter = image.iter();
    let mut answer2 = String::new();
    for _row in 0..ROWS {
        for _col in 0..COLS {
            match iter.next() {
                Some(0) => answer2.push('\u{2585}'),
                Some(1) => answer2.push(' '),
                Some(_) => bail!("Bad input: Found digit that is neither 0 nor 1"),
                None => bail!("Bad input. Must contain {} rows and {} columns", ROWS, COLS),
            }
        }
        answer2.push('\n');
    }

    Ok((answer1.to_string(), answer2))
}
