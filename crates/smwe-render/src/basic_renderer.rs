use glow::*;
use itertools::Itertools;

#[derive(Clone, Debug)]
pub struct BasicRenderer {
    shader_program: Program,
    vao:            VertexArray,
    vbo:            Buffer,
    vertex_count:   usize,
    primitive_type: u32,
    destroyed:      bool,
}

#[derive(Copy, Clone, Debug)]
pub struct ShaderSources {
    pub vertex_shader:   &'static str,
    pub geometry_shader: Option<&'static str>,
    pub fragment_shader: &'static str,
}

#[derive(Copy, Clone, Debug)]
pub struct GlVertexAttribute {
    pub index:     u32,
    pub size:      i32,
    pub data_type: u32,
    pub stride:    i32,
    pub offset:    i32,
}

pub trait BindUniforms {
    /// # Safety
    /// Calls to unsafe glow functions.
    unsafe fn bind_uniforms(&self, gl: &Context, shader_program: Program);
}

impl BasicRenderer {
    pub fn new(
        gl: &Context, shader_sources: ShaderSources, vertex_attribute: GlVertexAttribute, primitive_type: u32,
    ) -> Self {
        let shader_program = unsafe { gl.create_program().expect("Failed to create shader program") };

        let mut shader_infos =
            vec![(VERTEX_SHADER, shader_sources.vertex_shader), (FRAGMENT_SHADER, shader_sources.fragment_shader)];
        if let Some(geometry_shader) = shader_sources.geometry_shader {
            shader_infos.push((GEOMETRY_SHADER, geometry_shader));
        }

        let shaders = shader_infos
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

        let vao = unsafe { gl.create_vertex_array().expect("Failed to create vertex array for TileRenderer") };

        let vbo = unsafe {
            let buf = gl.create_buffer().expect("Failed to create vertex buffer for TileRenderer");
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(ARRAY_BUFFER, Some(buf));
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_i32(
                vertex_attribute.index,
                vertex_attribute.size,
                vertex_attribute.data_type,
                vertex_attribute.stride,
                vertex_attribute.offset,
            );
            buf
        };

        Self { shader_program, vao, vbo, vertex_count: 0, primitive_type, destroyed: false }
    }

    pub fn destroy(&mut self, gl: &Context) {
        if self.destroyed {
            log::warn!("Attempted to destroy BasicRenderer after it was already destroyed");
            return;
        }
        unsafe {
            gl.delete_program(self.shader_program);
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
        }
        self.destroyed = true;
    }

    pub fn paint(&self, gl: &Context, uniforms: &impl BindUniforms) {
        if self.destroyed {
            log::warn!("Attempted to paint BasicRenderer after it was already destroyed");
            return;
        }
        unsafe {
            gl.use_program(Some(self.shader_program));
            uniforms.bind_uniforms(gl, self.shader_program);
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(ARRAY_BUFFER, Some(self.vbo));
            gl.draw_arrays(self.primitive_type, 0, self.vertex_count as i32);
        }
    }

    pub fn set_vertices<Vertex>(&mut self, gl: &Context, vertices: Vec<Vertex>) {
        if self.destroyed {
            log::warn!("Attempted to set vertices in BasicRenderer after it was already destroyed");
            return;
        }
        self.vertex_count = vertices.len();
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(ARRAY_BUFFER, vertices.align_to().1, DYNAMIC_DRAW);
        }
    }
}
