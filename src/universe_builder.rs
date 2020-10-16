use wasm_bindgen::prelude::*;

use crate::{Cell, Universe, utils};

#[wasm_bindgen]
pub struct EmptyUniverse;

#[wasm_bindgen]
pub struct UniverseWithWidth {
    width: usize
}

#[wasm_bindgen]
pub struct UniverseWithHeight {
    height: usize
}

#[wasm_bindgen]
pub struct ScaledUniverse {
    width: usize,
    height: usize,
}

#[wasm_bindgen]
impl EmptyUniverse {
    pub fn with_width(self, width: usize) -> UniverseWithWidth {
        utils::set_panic_hook();
        let state = UniverseWithWidth {
            width
        };
        state
    }

    pub fn with_height(self, height: usize) -> UniverseWithHeight {
        utils::set_panic_hook();
        let state = UniverseWithHeight {
            height
        };
        state
    }
}

#[wasm_bindgen]
impl UniverseWithWidth {
    pub fn with_height(self, height: usize) -> ScaledUniverse {
        ScaledUniverse {
            width: self.width,
            height,
        }
    }
}

#[wasm_bindgen]
impl UniverseWithHeight {
    pub fn with_width(self, width: usize) -> ScaledUniverse {
        ScaledUniverse {
            height: self.height,
            width,
        }
    }
}

#[wasm_bindgen]
impl ScaledUniverse {
    pub fn random(self) -> Universe {
        use js_sys::Math::random;

        let width = self.width;
        let height = self.height;
        let mut cells = vec![Cell::Dead; width * height];
        for index in 0..cells.len()
        {
            let random = random();
            cells[index] = if random < 0.5 { Cell::Alive } else { Cell::Dead }
        }

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn empty(self) -> Universe {
        let width = self.width;
        let height = self.height;
        let cells = (0..width * height)
            .map(|_| Cell::Dead)
            .collect();

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn default(self) -> Universe {
        let width = self.width;
        let height = self.height;

        let mut cells = vec![Cell::Dead; width * height];
        for i in 0..cells.len() {
            cells[i] = if i % 2 == 0 || i % 7 == 0 { Cell::Alive } else { Cell::Dead };
        }

        Universe {
            width,
            height,
            cells,
        }
    }
}