use super::Renderer;
use crate::{Cell, Size, Universe};
use js_sys::{Float32Array, Int32Array};
use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlShader};

pub struct WebGLRenderer {
    grid: GridRenderProgram,
    cells: CellRenderProgram,
    size: Size,
}

impl WebGLRenderer {
    fn render_grid(&self, size: Size, ctx: &WebGlRenderingContext) {
        let grid = &self.grid;
        ctx.use_program(Some(&grid.program));

        if let Some(size_location) = ctx.get_uniform_location(&grid.program, "size") {
            ctx.uniform2i(Some(&size_location), size.width as i32, size.height as i32);
        }

        ctx.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&grid.buffer.buffer),
        );
        ctx.vertex_attrib_pointer_with_i32(0, 2, WebGlRenderingContext::FLOAT, false, 0, 0);
        ctx.enable_vertex_attrib_array(0);

        ctx.draw_arrays(
            WebGlRenderingContext::LINES,
            0,
            (grid.buffer.vertex_count) as i32,
        );
    }

    fn render_cells(&self, universe: &Universe, ctx: &WebGlRenderingContext) {
        let size = universe.size;
        assert_eq!(size, self.size);
        let cells = &self.cells;
        let alive_data: Vec<f32> = universe
            .get_cells()
            .iter()
            .flat_map(|cell| {
                vec![
                    match cell {
                        Cell::Alive => 1.0f32,
                        Cell::Dead => 0.0f32,
                    };
                    6
                ]
                .into_iter()
            })
            .collect();

        ctx.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&cells.alive.buffer),
        );

        unsafe {
            let vert_array = js_sys::Float32Array::view(&alive_data);

            ctx.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGlRenderingContext::DYNAMIC_DRAW,
            );
        }

        ctx.use_program(Some(&cells.program));

        if let Some(size_location) = ctx.get_uniform_location(&cells.program, "size") {
            ctx.uniform2i(Some(&size_location), size.width as i32, size.height as i32);
        }

        ctx.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&cells.rects.buffer),
        );
        let position_attrib_idx = ctx.get_attrib_location(&cells.program, "position") as u32;
        ctx.vertex_attrib_pointer_with_i32(
            position_attrib_idx,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        ctx.enable_vertex_attrib_array(position_attrib_idx);

        let alive_attrib_idx = ctx.get_attrib_location(&cells.program, "alive");
        if alive_attrib_idx >= 0 {
            let alive_attrib_idx = alive_attrib_idx as u32;
            ctx.bind_buffer(
                WebGlRenderingContext::ARRAY_BUFFER,
                Some(&cells.alive.buffer),
            );
            ctx.vertex_attrib_pointer_with_i32(
                alive_attrib_idx,
                1,
                WebGlRenderingContext::FLOAT,
                false,
                0,
                0,
            );
            ctx.enable_vertex_attrib_array(alive_attrib_idx);
        }

        ctx.draw_arrays(
            WebGlRenderingContext::TRIANGLES,
            0,
            cells.rects.vertex_count as i32,
        );
    }
}

#[repr(C)]
struct Vec2<T> {
    x: T,
    y: T,
}

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

fn compile_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

fn link_program(
    context: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

struct WebGlBufferedData {
    buffer: WebGlBuffer,
    vertex_count: usize,
}

impl WebGlBufferedData {
    fn len(&self) -> usize {
        self.vertex_count
    }
}

struct GridRenderProgram {
    program: WebGlProgram,
    buffer: WebGlBufferedData,
}

fn initialize_grid(size: Size, ctx: &WebGlRenderingContext) -> GridRenderProgram {
    let vertex_shader = compile_shader(
        ctx,
        WebGlRenderingContext::VERTEX_SHADER,
        include_str!("grid_vertex_shader.glsl"),
    )
    .unwrap();

    let frag_shader = compile_shader(
        ctx,
        WebGlRenderingContext::FRAGMENT_SHADER,
        r#"
    void main() {
        gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
    }
    "#,
    )
    .unwrap();
    let grid_program = link_program(ctx, &vertex_shader, &frag_shader).unwrap();

    let mut grid_lines: Vec<Vec2<f32>> =
        Vec::with_capacity((size.height + 1) * 2 + (size.width + 1) * 2);

    for x in 0..=size.width as i32 {
        grid_lines.push(Vec2::new(x as f32, 0f32));
        grid_lines.push(Vec2::new(x as f32, (size.height + 1) as f32));
    }

    for y in 0..=size.height as i32 {
        grid_lines.push(Vec2::new(0f32, y as f32));
        grid_lines.push(Vec2::new((size.width + 1) as f32, y as f32));
    }

    let grid_buffer = ctx.create_buffer().unwrap();

    ctx.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&grid_buffer));

    // Note that `Float32Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Float32Array` to be invalid.
    //
    // As a result, after `Float32Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe {
        let vertex_array = Float32Array::view(as_flat_slice(&grid_lines));
        ctx.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &vertex_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }

    GridRenderProgram {
        program: grid_program,
        buffer: WebGlBufferedData {
            buffer: grid_buffer,
            vertex_count: grid_lines.len(),
        },
    }
}

fn get_rect(x: f32, y: f32) -> [Vec2<f32>; 6] {
    let half_margin = 0.05;
    [
        Vec2::new(x + half_margin, y + half_margin), // top left
        Vec2::new(x + half_margin, y + 1.0 - half_margin), // top right
        Vec2::new(x + 1.0 - half_margin, y + half_margin), // bottom left
        Vec2::new(x + half_margin, y + 1f32 - half_margin), // top right
        Vec2::new(x + 1f32 - half_margin, y + half_margin), // bottom left
        Vec2::new(x + 1f32 - half_margin, y + 1f32 - half_margin), // bottom right
    ]
}

struct CellRenderProgram {
    program: WebGlProgram,
    rects: WebGlBufferedData,
    alive: WebGlBufferedData,
}

fn initialize_cells(size: Size, ctx: &WebGlRenderingContext) -> CellRenderProgram {
    let vertex_shader = compile_shader(
        ctx,
        WebGlRenderingContext::VERTEX_SHADER,
        include_str!("cells_vertex_shader.glsl"),
    )
    .unwrap();

    let frag_shader = compile_shader(
        &ctx,
        WebGlRenderingContext::FRAGMENT_SHADER,
        r#"
        varying lowp float frag_alive;
        void main() {
            highp vec3 color = vec3(1.0, 1.0, 1.0);
            color = (1.0 - frag_alive) * color;            
            gl_FragColor = vec4(color, 1.0);
        }
    "#,
    )
    .unwrap();

    let program = link_program(&ctx, &vertex_shader, &frag_shader).unwrap();

    let mut cell_rects = Vec::with_capacity(size.height * size.width);

    for y in 0..size.height {
        for x in 0..size.width {
            let rect: [Vec2<f32>; 6] = get_rect(x as f32, y as f32);

            cell_rects.push(rect);
        }
    }

    let rect_buffer = ctx.create_buffer().unwrap();
    ctx.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&rect_buffer));

    // Note that `Float32Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Float32Array` to be invalid.
    //
    // As a result, after `Float32Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe {
        let vert_array = js_sys::Float32Array::view(array_to_flat_slice(cell_rects.as_slice()));

        ctx.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }

    let alive_buffer = ctx.create_buffer().unwrap();
    ctx.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&alive_buffer));

    CellRenderProgram {
        program: program,
        rects: WebGlBufferedData {
            buffer: rect_buffer,
            vertex_count: cell_rects.len() * 6,
        },
        alive: WebGlBufferedData {
            buffer: alive_buffer,
            vertex_count: cell_rects.len() * 6,
        },
    }
}

// Safety: should be safe to do, as Vec2 is declared as #[repr(C)] so we are guaranteed two fields are layed out adjacent
unsafe fn as_flat_slice<T>(slice: &[Vec2<T>]) -> &[T] {
    let raw_ptr = slice.as_ptr() as *const T;
    let len = slice.len() * 2;

    std::slice::from_raw_parts(raw_ptr, len)
}

unsafe fn array_to_flat_slice<T>(slice: &[[Vec2<T>; 6]]) -> &[T] {
    let raw_ptr = slice.as_ptr() as *const T;
    let len = slice.len() * 6 * 2;

    std::slice::from_raw_parts(raw_ptr, len)
}

impl Renderer for WebGLRenderer {
    type Context = WebGlRenderingContext;

    fn init(universe: &Universe, context: &Self::Context) -> WebGLRenderer {
        WebGLRenderer {
            grid: initialize_grid(universe.size, context),
            cells: initialize_cells(universe.size, context),
            size: universe.size,
        }
    }

    fn render_to_canvas(&mut self, universe: &Universe, ctx: &Self::Context) {
        ctx.clear_color(1.0, 1.0, 1.0, 1.0);
        ctx.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
        self.render_grid(universe.size, ctx);
        self.render_cells(universe, ctx);
    }
}
