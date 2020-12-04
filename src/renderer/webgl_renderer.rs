use super::Renderer;
use crate::{Cell, Size, Universe};
use js_sys::{Float32Array, Int32Array};
use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlShader};

pub struct WebGLRenderer {}

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

struct GridRenderProgram {
    program: WebGlProgram,
    buffer: WebGlBuffer,
    bufferlen: usize,
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
        buffer: grid_buffer,
        bufferlen: grid_lines.len(),
    }
}

fn render_grid(size: Size, ctx: &WebGlRenderingContext) {
    let grid = initialize_grid(size, &ctx);
    ctx.use_program(Some(&grid.program));

    if let Some(size_location) = ctx.get_uniform_location(&grid.program, "size") {
        ctx.uniform2i(Some(&size_location), size.width as i32, size.height as i32);
    }

    ctx.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&grid.buffer));
    ctx.vertex_attrib_pointer_with_i32(0, 2, WebGlRenderingContext::FLOAT, false, 0, 0);
    ctx.enable_vertex_attrib_array(0);

    ctx.draw_arrays(WebGlRenderingContext::LINES, 0, (grid.bufferlen) as i32);
}

// Safety: should be safe to do, as Vec2 is declared as #[repr(C)] so we are guaranteed two fields are layed out adjacent
unsafe fn as_flat_slice<T>(slice: &[Vec2<T>]) -> &[T] {
    let raw_ptr = slice.as_ptr() as *const T;
    let len = slice.len() * 2;

    std::slice::from_raw_parts(raw_ptr, len)
}

impl Renderer for WebGLRenderer {
    type Context = WebGlRenderingContext;

    fn init(_: &Self::Context) -> WebGLRenderer {
        WebGLRenderer {}
    }

    fn render_to_canvas(&mut self, universe: &Universe, ctx: &Self::Context) {
        ctx.clear_color(1.0, 1.0, 1.0, 1.0);
        ctx.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
        render_grid(universe.size, ctx);
        return;
        let vertex_shader = compile_shader(
            ctx,
            WebGlRenderingContext::VERTEX_SHADER,
            r#"
            attribute vec2 position;
            attribute bool alive;
            void main() {
                gl_Position = position;
            }
        "#,
        )
        .unwrap();

        let frag_shader = compile_shader(
            &ctx,
            WebGlRenderingContext::FRAGMENT_SHADER,
            r#"
            void main() {
                gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
            }
        "#,
        )
        .unwrap();

        let program = link_program(&ctx, &vertex_shader, &frag_shader).unwrap();
        ctx.use_program(Some(&program));

        let vertices: [f32; 9] = [-0.7, -0.7, 0.0, 0.7, -0.7, 0.0, 0.0, 0.7, 0.0];

        let mut coordinates: Vec<(i32, i32)> = universe
            .get_cells()
            .iter()
            .enumerate()
            .map(|(idx, _)| universe.size.get_address(idx))
            .map(|address| (address.1 as i32, address.0 as i32))
            .collect();

        let coordinate_buffer = ctx.create_buffer().unwrap();
        ctx.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&coordinate_buffer),
        );

        // Note that `Float32Array::view` is somewhat dangerous (hence the
        // `unsafe`!). This is creating a raw view into our module's
        // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
        // (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the `Float32Array` to be invalid.
        //
        // As a result, after `Float32Array::view` we have to be very careful not to
        // do any memory allocations before it's dropped.
        unsafe {
            let vert_array = js_sys::Int32Array::view_mut_raw(
                coordinates.as_mut_ptr() as *mut i32,
                coordinates.len() * 2,
            );

            ctx.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGlRenderingContext::STATIC_DRAW,
            );
        }

        let cell_buffer = ctx.create_buffer().unwrap();
        ctx.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&cell_buffer));

        // Note that `Float32Array::view` is somewhat dangerous (hence the
        // `unsafe`!). This is creating a raw view into our module's
        // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
        // (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the `Float32Array` to be invalid.
        //
        // As a result, after `Float32Array::view` we have to be very careful not to
        // do any memory allocations before it's dropped.
        unsafe {
            let vert_array = js_sys::Int8Array::view_mut_raw(
                universe.cells() as *mut Cell as *mut u8 as *mut i8,
                universe.get_cells().len(),
            );

            ctx.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGlRenderingContext::DYNAMIC_DRAW,
            );
        }

        ctx.vertex_attrib_pointer_with_i32(0, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
        ctx.enable_vertex_attrib_array(0);

        ctx.clear_color(0.0, 0.0, 0.0, 1.0);
        ctx.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        ctx.draw_arrays(
            WebGlRenderingContext::TRIANGLES,
            0,
            (vertices.len() / 3) as i32,
        );
    }
}
