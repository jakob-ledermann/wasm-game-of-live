use std::fmt;

use renderer::{canvas_renderer, webgl_renderer, Renderer};
use wasm_bindgen::prelude::*;

use crate::timer::Timer;
use crate::universe_builder::*;
use crate::Cell::{Alive, Dead};

#[macro_use]
mod utils;
mod renderer;

pub mod timer;
pub mod universe_builder;

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
    size: Size,
    active: usize,
    cells: [Vec<Cell>; 2],
}

#[derive(Copy, Clone, Debug, Default)]
struct Size {
    width: usize,
    height: usize,
}

impl Size {
    fn get_index(&self, row: usize, column: usize) -> usize {
        (row * self.width + column) as usize
    }

    #[inline]
    fn get_address(&self, index: usize) -> (usize, usize) {
        let row = index / self.width;
        let col = index % self.width;
        (row, col)
    }
}

impl Universe {
    fn live_neighbor_count(size: &Size, active_cells: &[Cell], row: usize, column: usize) -> u8 {
        let north = if row == 0 { size.height - 1 } else { row - 1 };

        let south = if row == size.height - 1 { 0 } else { row + 1 };

        let west = if column == 0 {
            size.width - 1
        } else {
            column - 1
        };

        let east = if column == size.width - 1 {
            0
        } else {
            column + 1
        };

        let neighbors = [
            (north, west),
            (north, column),
            (north, east),
            (row, west),
            (row, east),
            (south, west),
            (south, column),
            (south, east),
        ];

        let mut count = 0;
        for (r, c) in neighbors.iter() {
            let index = size.get_index(*r, *c);
            count += unsafe { *active_cells.get_unchecked(index) } as u8
        }

        count
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.get_cells().chunks(self.size.width as usize / 8) {
            for &cell in line {
                let symbol = match cell {
                    Alive => '◼',
                    Dead => '◻',
                };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn build() -> EmptyUniverse {
        EmptyUniverse {}
    }

    pub fn tick(&mut self) {
        let _timer = Timer::new("Universe::tick");

        let next_buffer = match self.active {
            x if x < self.cells.len() - 1 => x + 1,
            x if x == self.cells.len() - 1 => 0,
            _ => panic!("only one buffer is available"),
        };

        {
            let buffers = &mut self.cells;
            let cells = buffers.split_at_mut(1);
            let size = &self.size;

            let (active_cells, next) = match self.active {
                0 => (&cells.0[0], &mut cells.1[0]),
                1 => (&cells.1[0], &mut cells.0[0]),
                _ => panic!("there should not be more than 2 buffers"),
            };

            let _timer = Timer::new("new generation");
            let mut row = 0usize;
            let mut col = 0usize;
            for (idx, cell) in active_cells.iter().enumerate() {
                let live_neighbors = Universe::live_neighbor_count(size, active_cells, row, col);

                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (Cell::Dead, 3) => Cell::Alive,
                    // All other cells remain in the same state.
                    (&otherwise, _) => otherwise,
                };
                /*
                                log!("    it becomes {:?}", next_cell);
                */
                next[idx] = next_cell;
            }

            self.active = next_buffer
        }
    }

    pub fn width(&self) -> u32 {
        self.size.width as u32
    }

    pub fn height(&self) -> u32 {
        self.size.height as u32
    }

    pub fn cells(&self) -> *const Cell {
        self.get_cells().as_ptr()
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn render_to_canvas_webgl(&self, context: web_sys::WebGlRenderingContext) -> () {
        let mut renderer = webgl_renderer::WebGLRenderer::init(&context);
        renderer.render_to_canvas(self, &context);
    }

    pub fn render_to_canvas_2d(&self, context: web_sys::CanvasRenderingContext2d) -> () {
        let mut renderer = canvas_renderer::CanvasRenderer::init(&context);
        renderer.render_to_canvas(self, &context)
    }

    pub fn toggle_cell(&mut self, row: u32, col: u32) {
        let idx = self.size.get_index(row as usize, col as usize);
        self.get_cells()[idx].toggle();
    }
}

impl Universe {
    pub fn get_cells(&self) -> &[Cell] {
        &self.cells[self.active]
    }

    pub fn set_cells(&mut self, cells: &[(usize, usize)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.size.get_index(row, col);
            self.get_cells()[idx] = Cell::Alive;
        }
    }
}

impl Cell {
    pub fn toggle(&mut self) {
        *self = match *self {
            Dead => Alive,
            Alive => Dead,
        }
    }
}
