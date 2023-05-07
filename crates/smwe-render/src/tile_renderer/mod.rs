use emath::Vec2;
use glow::*;
use itertools::Itertools;

use crate::gfx_buffers::GfxBuffers;

const VERTEX_SHADER_SRC: &str = include_str!("tile.vs.glsl");
const GEOMETRY_SHADER_SRC: &str = include_str!("tile.gs.glsl");
const FRAGMENT_SHADER_SRC: &str = include_str!("tile.fs.glsl");

#[derive(Copy, Clone, Debug, Default)]
pub struct Tile(pub [u32; 4]);

#[derive(Debug)]
pub struct TileRenderer {
    shader_program: Program,
    vao:            VertexArray,
    vbo:            Buffer,
    tiles_count:    usize,
    destroyed:      bool,
}

impl TileRenderer {
    pub fn new(gl: &Context) -> Self {
        let shader_program =
            unsafe { gl.create_program().expect("Failed to create shader program for background layer") };

        let shader_sources = [
            (VERTEX_SHADER, VERTEX_SHADER_SRC),
            (GEOMETRY_SHADER, GEOMETRY_SHADER_SRC),
            (FRAGMENT_SHADER, FRAGMENT_SHADER_SRC),
        ];

        let shaders = shader_sources
            .into_iter()
            .map(|(shader_type, shader_source)| unsafe {
                let shader = gl.create_shader(shader_type).expect("Failed to create shader");
                gl.shader_source(shader, shader_source);
                gl.compile_shader(shader);

                debug_assert!(
                    gl.get_shader_compile_status(shader),
                    "Failed to compile {shader_type}: {}",
                    gl.get_shader_info_log(shader),
                );

                gl.attach_shader(shader_program, shader);
                shader
            })
            .collect_vec();

        unsafe {
            gl.link_program(shader_program);
            assert!(gl.get_program_link_status(shader_program), "{}", gl.get_program_info_log(shader_program));
        }

        shaders.into_iter().for_each(|shader| unsafe {
            gl.detach_shader(shader_program, shader);
            gl.delete_shader(shader);
        });

        let vao = unsafe { gl.create_vertex_array().expect("Failed to create vertex array for background layer") };

        let vbo = unsafe {
            let buf = gl.create_buffer().expect("Failed to create vertex buffer for background layer");
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(ARRAY_BUFFER, Some(buf));
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_i32(0, 4, INT, 0, 0);
            buf
        };

        Self { shader_program, vao, vbo, tiles_count: 0, destroyed: false }
    }

    pub fn destroy(&mut self, gl: &Context) {
        if self.destroyed {
            return;
        }
        unsafe {
            gl.delete_program(self.shader_program);
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
        }
        self.destroyed = true;
    }

    pub fn paint(&self, gl: &Context, gfx_bufs: GfxBuffers, screen_size: Vec2, offset: Vec2) {
        if self.destroyed {
            return;
        }
        unsafe {
            gl.use_program(Some(self.shader_program));

            let u = gl.get_uniform_location(self.shader_program, "offset");
            gl.uniform_2_f32(u.as_ref(), offset.x, offset.y);

            let u = gl.get_uniform_location(self.shader_program, "screen_size");
            gl.uniform_2_f32(u.as_ref(), screen_size.x, screen_size.y);

            gl.bind_buffer_base(ARRAY_BUFFER, 0, Some(gfx_bufs.palette_buf));
            let palette_block =
                gl.get_uniform_block_index(self.shader_program, "Color").expect("Failed to get 'Color' block");
            gl.uniform_block_binding(self.shader_program, palette_block, 0);

            gl.bind_buffer_base(ARRAY_BUFFER, 1, Some(gfx_bufs.vram_buf));
            let vram_block =
                gl.get_uniform_block_index(self.shader_program, "Graphics").expect("Failed to get 'Graphics' block");
            gl.uniform_block_binding(self.shader_program, vram_block, 1);

            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(ARRAY_BUFFER, Some(self.vbo));

            gl.draw_arrays(POINTS, 0, self.tiles_count as i32);
        }
    }

    pub fn set_tiles(&mut self, gl: &Context, tiles: Vec<Tile>) {
        if self.destroyed {
            return;
        }
        self.tiles_count = tiles.len();
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(ARRAY_BUFFER, tiles.align_to().1, DYNAMIC_DRAW);
        }
    }
}
