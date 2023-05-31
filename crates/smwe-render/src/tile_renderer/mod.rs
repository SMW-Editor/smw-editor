use emath::*;
use glow::*;

use crate::{
    basic_renderer::{BasicRenderer, BindUniforms, GlVertexAttribute, ShaderSources},
    gfx_buffers::GfxBuffers,
};

const VERTEX_SHADER_SRC: &str = include_str!("tile.vs.glsl");
const GEOMETRY_SHADER_SRC: &str = include_str!("tile.gs.glsl");
const FRAGMENT_SHADER_SRC: &str = include_str!("tile.fs.glsl");

#[derive(Debug)]
pub struct TileRenderer {
    renderer: BasicRenderer,
}

#[derive(Debug)]
pub struct TileUniforms {
    pub gfx_bufs:    GfxBuffers,
    pub screen_size: Vec2,
    pub offset:      Vec2,
    pub zoom:        f32,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Tile(pub [u32; 4]);

impl TileRenderer {
    pub fn new(gl: &Context) -> Self {
        let shader_sources = ShaderSources {
            vertex_shader:   VERTEX_SHADER_SRC,
            geometry_shader: Some(GEOMETRY_SHADER_SRC),
            fragment_shader: FRAGMENT_SHADER_SRC,
        };
        let vertex_attribute =
            GlVertexAttribute { index: 0, size: 4, data_type: INT, stride: 0, offset: 0 };
        let renderer = BasicRenderer::new(gl, shader_sources, vertex_attribute);
        Self { renderer }
    }

    pub fn destroy(&mut self, gl: &Context) {
        self.renderer.destroy(gl);
    }

    pub fn paint(&self, gl: &Context, uniforms: &TileUniforms) {
        self.renderer.paint(gl, uniforms);
    }

    pub fn set_tiles(&mut self, gl: &Context, tiles: Vec<Tile>) {
        self.renderer.set_vertices(gl, tiles);
    }
}

impl BindUniforms for TileUniforms {
    unsafe fn bind_uniforms(&self, gl: &Context, shader_program: Program) {
        let u = gl.get_uniform_location(shader_program, "offset");
        gl.uniform_2_f32(u.as_ref(), self.offset.x, self.offset.y);

        let u = gl.get_uniform_location(shader_program, "screen_size");
        gl.uniform_2_f32(u.as_ref(), self.screen_size.x, self.screen_size.y);

        let u = gl.get_uniform_location(shader_program, "zoom");
        gl.uniform_1_f32(u.as_ref(), self.zoom);

        gl.bind_buffer_base(UNIFORM_BUFFER, 0, Some(self.gfx_bufs.palette_buf));
        let palette_block =
            gl.get_uniform_block_index(shader_program, "Color").expect("Failed to get uniform block 'Color'");
        gl.uniform_block_binding(shader_program, palette_block, 0);

        gl.bind_buffer_base(UNIFORM_BUFFER, 1, Some(self.gfx_bufs.vram_buf));
        let vram_block =
            gl.get_uniform_block_index(shader_program, "Graphics").expect("Failed to get uniform block 'Graphics'");
        gl.uniform_block_binding(shader_program, vram_block, 1);
    }
}

impl Tile {
    pub fn pos(self) -> Pos2 {
        pos2(self.0[0] as f32, self.0[1] as f32)
    }

    pub fn tile_num(self) -> u32 {
        self.0[2]
    }

    pub fn scale(self) -> u32 {
        self.0[3] & 0xFF
    }

    pub fn move_by(&mut self, offset: Vec2) {
        self.0[0] = (self.0[0] as i32 + offset.x as i32) as u32;
        self.0[1] = (self.0[1] as i32 + offset.y as i32) as u32;
    }

    pub fn snap_to_grid(&mut self, cell_size: u32, cell_origin: Vec2) {
        let cell_size = cell_size as f32;
        self.0[0] = {
            let origin_pos = self.0[0] as f32 + cell_origin.x;
            let unscaled = origin_pos / cell_size;
            let cell_coord = unscaled.floor();
            (cell_coord * cell_size) as u32
        };
        self.0[1] = {
            let origin_pos = self.0[1] as f32 + cell_origin.y;
            let unscaled = origin_pos / cell_size;
            let cell_coord = unscaled.floor();
            (cell_coord * cell_size) as u32
        };
    }
}
