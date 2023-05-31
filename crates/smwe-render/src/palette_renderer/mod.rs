use glow::*;

use crate::basic_renderer::{BasicRenderer, BindUniforms, GlVertexAttribute, ShaderSources};

const VERTEX_SHADER_SRC: &str = include_str!("palette.vs.glsl");
const FRAGMENT_SHADER_SRC: &str = include_str!("palette.fs.glsl");

#[derive(Debug)]
pub struct PaletteRenderer {
    renderer: BasicRenderer,
}

#[derive(Debug)]
pub struct PaletteUniforms {
    pub palette_buf: Buffer,
}

impl PaletteRenderer {
    pub fn new(gl: &Context) -> Self {
        let shader_sources = ShaderSources {
            vertex_shader:   VERTEX_SHADER_SRC,
            geometry_shader: None,
            fragment_shader: FRAGMENT_SHADER_SRC,
        };
        let vertex_attribute =
            GlVertexAttribute { index: 0, size: 1, data_type: INT, stride: 0, offset: 0 };
        let mut renderer = BasicRenderer::new(gl, shader_sources, vertex_attribute, TRIANGLE_STRIP);

        let vertices = vec![0b00, 0b10, 0b01, 0b11];
        renderer.set_vertices(gl, vertices);

        Self { renderer }
    }

    pub fn destroy(&mut self, gl: &Context) {
        self.renderer.destroy(gl);
    }

    pub fn paint(&self, gl: &Context, uniforms: &PaletteUniforms) {
        self.renderer.paint(gl, uniforms);
    }
}

impl BindUniforms for PaletteUniforms {
    unsafe fn bind_uniforms(&self, gl: &Context, shader_program: Program) {
        gl.bind_buffer_base(UNIFORM_BUFFER, 0, Some(self.palette_buf));
        let palette_block =
            gl.get_uniform_block_index(shader_program, "Color").expect("Failed to get uniform block 'Color'");
        gl.uniform_block_binding(shader_program, palette_block, 0);
    }
}
