use crate::{Universe, Cell, utils};

pub struct UniverseBuilder<T> {
   state: T
}

pub struct EmptyUniverse;

pub struct UniverseWithWidth {
    width: usize
}

pub struct UniverseWithHeight  {
    height: usize
}

pub struct ScaledUniverse {
    width: usize,
    height: usize
}

impl UniverseBuilder<EmptyUniverse> {
    pub fn with_width(width: usize) -> UniverseBuilder<UniverseWithWidth> {
        utils::set_panic_hook();
        let state = UniverseWithWidth {
            width
        };

        UniverseBuilder {
            state
        }
    }

    pub fn with_height(height: usize) -> UniverseBuilder<UniverseWithHeight> {
        utils::set_panic_hook();
        let state = UniverseWithHeight {
            height
        };

        UniverseBuilder {
            state
        }
    }
}

impl UniverseBuilder<UniverseWithWidth> {
    pub fn with_height(self, height: usize) -> UniverseBuilder<ScaledUniverse> {
        UniverseBuilder {
            state: ScaledUniverse {
                width: self.state.width,
                height
            }
        }
    }
}

impl UniverseBuilder<UniverseWithHeight> {
    pub fn with_width(self, width: usize) -> UniverseBuilder<ScaledUniverse> {
        UniverseBuilder {
            state: ScaledUniverse {
                height: self.state.height,
                width
            }
        }
    }
}

impl UniverseBuilder<ScaledUniverse> {
    pub fn random(self) -> Universe {
        use js_sys::Math::random;

        let width = self.state.width;
        let height = self.state.height;
        let mut cells = vec![Cell::Dead; width * height];
        for index in 0..cells.len()
        {
            let random = random();
            cells[index] = if random < 0.5 { Cell::Alive } else { Cell::Dead }
        }

        Universe {
            width,
            height,
            cells
        }
    }

    pub fn empty(self) -> Universe {
        let width = self.state.width;
        let height = self.state.height;
        let cells = (0..width * height)
            .map(|_| Cell::Dead)
            .collect();

        Universe {
            width,
            height,
            cells
        }
    }

    pub fn default(self) -> Universe {
        let width = self.state.width;
        let height = self.state.height;

        let mut cells = Vec::with_capacity(width * height);
        for i in 0..cells.len() {
            cells[i] = if i % 2 == 0 || i % 7 == 0 { Cell::Alive } else { Cell::Dead };
        }

        Universe {
            width,
            height,
            cells
        }
    }
}