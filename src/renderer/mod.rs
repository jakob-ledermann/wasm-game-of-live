use crate::Universe;

pub trait Renderer {
    type Context;

    fn init(universe: &Universe, ctx: &Self::Context) -> Self;

    fn render_to_canvas(&mut self, universe: &Universe, ctx: &Self::Context);
}

pub mod canvas_renderer;
pub mod webgl_renderer;
