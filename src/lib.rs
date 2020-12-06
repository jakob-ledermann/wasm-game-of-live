mod utils;

use wasm_bindgen::prelude::*;
use fixedbitset::FixedBitSet;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    width: usize,
    height: usize,
    cells: FixedBitSet,
}

impl Universe {
    fn get_index(&self, row: usize, column: usize) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: usize, column: usize) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }
}

use std::fmt;

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize / 8) {
            for &cells in line {
                for bit_index in (0u8..32u8) {
                    let mask = 1 << bit_index;
                    let cell = &cells & mask == mask;
                    let symbol = if cell { '◻' } else { '◼' };
                    write!(f, "{}", symbol)?;
                }
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                next.set(idx, match (cell, live_neighbors) {
                    (true, x) if x < 2 => false,
                    (true, 2) | (true, 3) => true,
                    (true, x) if x > 3 => false,
                    (false, 3) => true,
                    (otherwise, _) => otherwise
                });
            }
        }

        self.cells = next;
    }

    pub fn new() -> Universe {
        utils::set_panic_hook();
        let width = 64;
        let height = 64;

        let mut cells = FixedBitSet::with_capacity(width * height);
        for i in 0..cells.len() {
            cells.set(i, i % 2 == 0 || i % 7 == 0);
        }

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn new_spaceship() -> Universe {
        let width = 64;
        let height = 64;
        let cells = FixedBitSet::with_capacity(width * height);

        Universe {
            width,
            height,
            cells
        }
    }

    pub fn generate_random() -> Universe {
        use js_sys::Math::random;

        let width = 64;
        let height = 64;
        let mut cells = FixedBitSet::with_capacity(width * height);
        for index in 0..cells.len()
        {
            let random = random();
            cells.set(index, random < 0.5)
        }

        Universe {
            width,
            height,
            cells
        }
    }

    pub fn width(&self) -> u32 {
        self.width as u32
    }

    pub fn height(&self) -> u32 {
        self.height as u32
    }

    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }

    pub fn render(&self) -> String {
        self.to_string()
    }
}