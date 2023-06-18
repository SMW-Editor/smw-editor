use emath::*;
use glow::*;
use serde::{Deserialize, Serialize};
use smwe_math::space::{OnCanvas, OnScreen};
use thiserror::Error;

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TileJson {
    x:         u32,
    y:         u32,
    tile_id:   u32,
    scale:     u8,
    color_row: u8,
    flip_x:    bool,
    flip_y:    bool,
}

#[derive(Debug, Error)]
#[error("Could not deserialize tile")]
pub struct TileDeserializeError;

impl TileRenderer {
    pub fn new(gl: &Context) -> Self {
        let shader_sources = ShaderSources {
            vertex_shader:   VERTEX_SHADER_SRC,
            geometry_shader: Some(GEOMETRY_SHADER_SRC),
            fragment_shader: FRAGMENT_SHADER_SRC,
        };
        let vertex_attribute =
            GlVertexAttribute { index: 0, size: 4, data_type: INT, stride: 0, offset: 0 };
        let renderer = BasicRenderer::new(gl, shader_sources, vertex_attribute, POINTS);
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
    #[inline]
    pub fn pos(self) -> OnCanvas<Pos2> {
        OnCanvas(pos2(self.0[0] as f32, self.0[1] as f32))
    }

    #[inline]
    pub fn tile_num(self) -> u32 {
        self.0[2]
    }

    #[inline]
    pub fn scale(self) -> u32 {
        self.0[3] & 0xFF
    }

    #[inline]
    pub fn color_row(self) -> u32 {
        (self.0[3] >> 8) & 0xF
    }

    #[inline]
    pub fn flip_x(self) -> bool {
        (self.0[3] & 0x4000) != 0
    }

    #[inline]
    pub fn flip_y(self) -> bool {
        (self.0[3] & 0x8000) != 0
    }

    #[inline]
    pub fn move_by(&mut self, offset: OnCanvas<Vec2>) {
        self.0[0] = (self.0[0] as i32 + offset.0.x as i32) as u32;
        self.0[1] = (self.0[1] as i32 + offset.0.y as i32) as u32;
    }

    #[inline]
    pub fn snap_to_grid(&mut self, cell_size: u32, cell_origin: OnScreen<Vec2>) {
        let cell_size = cell_size as f32;
        self.0[0] = {
            let origin_pos = self.0[0] as f32 + cell_origin.0.x;
            let unscaled = origin_pos / cell_size;
            let cell_coord = unscaled.floor();
            (cell_coord * cell_size) as u32
        };
        self.0[1] = {
            let origin_pos = self.0[1] as f32 + cell_origin.0.y;
            let unscaled = origin_pos / cell_size;
            let cell_coord = unscaled.floor();
            (cell_coord * cell_size) as u32
        };
    }

    #[inline]
    pub fn contains_point(self, point: OnCanvas<Pos2>) -> bool {
        let min = self.pos().0;
        let size = Vec2::splat(self.scale() as f32);
        Rect::from_min_size(min, size).contains(point.0)
    }

    #[inline]
    pub fn intersects_rect(self, rect: OnCanvas<Rect>) -> bool {
        let min = self.pos().0;
        let size = Vec2::splat(self.scale() as f32);
        Rect::from_min_size(min, size).intersects(rect.0)
    }
}

impl From<Tile> for TileJson {
    fn from(value: Tile) -> Self {
        Self {
            x:         value.0[0],
            y:         value.0[1],
            tile_id:   value.tile_num(),
            scale:     value.scale() as u8,
            color_row: value.color_row() as u8,
            flip_x:    value.flip_x(),
            flip_y:    value.flip_y(),
        }
    }
}

impl From<TileJson> for Tile {
    fn from(value: TileJson) -> Self {
        Self([
            value.x,
            value.y,
            value.tile_id,
            value.scale as u32
                | ((value.color_row as u32 & 0xF) << 8)
                | (0x4000 * value.flip_x as u32)
                | (0x8000 * value.flip_x as u32),
        ])
    }
}
