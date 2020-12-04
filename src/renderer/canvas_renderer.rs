use super::Renderer;
use crate::{Cell, Universe};
use wasm_bindgen::JsValue;

pub struct CanvasRenderer;

impl CanvasRenderer {
    fn draw_grid(universe: &Universe, ctx: &web_sys::CanvasRenderingContext2d) -> () {
        let grid_color: JsValue = JsValue::from_str("#CCCCCC");
        let width = universe.size.width as f64;
        let height = universe.size.height as f64;
        ctx.begin_path();
        ctx.set_stroke_style(&grid_color);

        // Vertical Lines
        for i in 0..universe.size.width {
            let x = i as f64;
            ctx.move_to(x, 0.0);
            ctx.line_to(x, height)
        }

        for j in 0..universe.size.height {
            let y = j as f64;
            ctx.move_to(0.0, y);
            ctx.line_to(width, y);
        }

        ctx.stroke();
    }

    fn draw_cells(universe: &Universe, ctx: &web_sys::CanvasRenderingContext2d) -> () {
        let alive_color = JsValue::from_str("#000000");
        let dead_color = JsValue::from_str("#FFFFFF");

        ctx.begin_path();

        for (idx, cell) in universe.get_cells().iter().enumerate() {
            let (y, x) = universe.size.get_address(idx);
            let color = match cell {
                Cell::Alive => &alive_color,
                Cell::Dead => &dead_color,
            };
            ctx.set_fill_style(color);
            ctx.fill_rect(x as f64, y as f64, 0.9, 0.9);
        }
    }
}

impl Renderer for CanvasRenderer {
    type Context = web_sys::CanvasRenderingContext2d;

    fn init(_: &Self::Context) -> Self {
        CanvasRenderer {}
    }

    fn render_to_canvas(&mut self, universe: &crate::Universe, ctx: &Self::Context) {
        ctx.clear_rect(
            0.0,
            0.0,
            universe.size.width as f64,
            universe.size.height as f64,
        );
        Self::draw_grid(&universe, ctx);
        Self::draw_cells(&universe, ctx)
    }
}
