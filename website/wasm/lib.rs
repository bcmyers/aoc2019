use std::io;

use aoc2019::{self, Rom};
use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct Game(aoc2019::Game);

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Game, JsValue> {
        console_error_panic_hook::set_once();
        let bytes = include_bytes!("../../data/13.txt");
        let reader = io::BufReader::new(&bytes[..]);
        let rom = Rom::from_reader(reader).map_err(|e| e.to_string())?;
        let game = aoc2019::Game::new(&rom);
        Ok(Self(game))
    }

    pub fn step(&mut self) -> Result<Option<i64>, JsValue> {
        let next_move = self.0.step().map_err(|e| e.to_string())?;
        Ok(next_move)
    }

    pub fn display(&self) -> *const u8 {
        self.0.display().as_ptr()
    }

    pub fn display_len(&self) -> usize {
        self.0.display().len()
    }

    pub fn rows(&self) -> usize {
        self.0.rows()
    }

    pub fn cols(&self) -> usize {
        self.0.cols()
    }

    pub fn input(&mut self, val: i64) {
        self.0.input(val)
    }

    pub fn score(&self) -> i64 {
        self.0.score()
    }
}
