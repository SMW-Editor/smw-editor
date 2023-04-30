use egui::{Rgba, Vec2};
use glow::*;
use itertools::Itertools;
use smwe_emu::Cpu;
use smwe_rom::graphics::color::Abgr1555;

use crate::ui::editor_prototypes::level_editor::shaders::{TILE_FS_SRC, TILE_GS_SRC, TILE_VS_SRC};

struct BackgroundLayer {
    shader_program: Program,
    vao:            VertexArray,
    vbo:            Buffer,
    tiles_count:    usize,
}

pub(super) struct LevelRenderer {
    layer1: BackgroundLayer,

    palette_buf: Buffer,
    vram_buf:    Buffer,
}

impl BackgroundLayer {
    fn new(gl: &Context) -> Self {
        let shader_program =
            unsafe { gl.create_program().expect("Failed to create shader program for background layer") };

        let shader_sources =
            [(VERTEX_SHADER, TILE_VS_SRC), (GEOMETRY_SHADER, TILE_GS_SRC), (FRAGMENT_SHADER, TILE_FS_SRC)];

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
            gl.bind_buffer(ARRAY_BUFFER, Some(buf));
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_i32(0, 4, INT, 0, 0);
            buf
        };

        Self { shader_program, vao, vbo, tiles_count: 0 }
    }

    fn destroy(&self, gl: &Context) {
        unsafe {
            gl.delete_program(self.shader_program);
            gl.delete_vertex_array(self.vao);
        }
    }

    fn paint(&self, gl: &Context, palette_buf: Buffer, vram_buf: Buffer, screen_size: Vec2) {
        unsafe {
            gl.use_program(Some(self.shader_program));

            let u = gl.get_uniform_location(self.shader_program, "offset");
            gl.uniform_2_f32_slice(u.as_ref(), &[0., 0.]);

            let u = gl.get_uniform_location(self.shader_program, "screen_size");
            gl.uniform_2_f32(u.as_ref(), screen_size.x, screen_size.y);

            gl.bind_buffer_base(ARRAY_BUFFER, 0, Some(palette_buf));
            let palette_block =
                gl.get_uniform_block_index(self.shader_program, "Color").expect("Failed to get 'Color' block");
            gl.uniform_block_binding(self.shader_program, palette_block, 0);

            gl.bind_buffer_base(ARRAY_BUFFER, 1, Some(vram_buf));
            let vram_block =
                gl.get_uniform_block_index(self.shader_program, "Graphics").expect("Failed to get 'Graphics' block");
            gl.uniform_block_binding(self.shader_program, vram_block, 1);

            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(ARRAY_BUFFER, Some(self.vbo));

            gl.draw_arrays(POINTS, 0, self.tiles_count as i32)
        }
    }

    fn load_layer(&mut self, gl: &Context, cpu: &mut Cpu) {
        let mut tiles = Vec::new();
        for idx in 0..512 * 27 {
            let (screen, sidx) = (idx / (16 * 27), idx % (16 * 27));
            let (row, column) = (sidx / 16, sidx % 16);
            let (block_x, block_y) = (column * 16 + screen * 256, row * 16);
            let block_id =
                cpu.mem.load_u8(0x7EC800 + idx as u32) as u16 | ((cpu.mem.load_u8(0x7FC800 + idx as u32) as u16) << 8);
            let block_ptr = cpu.mem.load_u16(0x0FBE + block_id as u32 * 2) as u32 + 0x0D0000;
            for (tile_id, (off_x, off_y)) in (0..4).zip([(0, 0), (0, 8), (8, 0), (8, 8)].into_iter()) {
                let tile_id = cpu.mem.load_u16(block_ptr + tile_id * 2) as i32;
                tiles.push([block_x + off_x, block_y + off_y, tile_id, 0]);
            }
        }
        self.tiles_count = tiles.len();
        unsafe {
            gl.bind_buffer(ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(ARRAY_BUFFER, tiles.align_to().1, DYNAMIC_DRAW);
        }
    }
}

impl LevelRenderer {
    pub(super) fn new(gl: &Context) -> Self {
        let layer1 = BackgroundLayer::new(gl);

        let palette_buf = make_buffer(gl, 256 * 16, 0);
        let vram_buf = make_buffer(gl, 0x2000, 1);

        Self { layer1, palette_buf, vram_buf }
    }

    pub(super) fn destroy(&self, gl: &Context) {
        unsafe {
            gl.delete_buffer(self.vram_buf);
            gl.delete_buffer(self.palette_buf);
        }
        self.layer1.destroy(gl);
    }

    pub(super) fn paint(&self, gl: &Context, screen_size: Vec2) {
        self.layer1.paint(gl, self.palette_buf, self.vram_buf, screen_size);
    }

    pub(super) fn upload_gfx(&self, gl: &Context, data: &[u8]) {
        unsafe {
            gl.bind_buffer(ARRAY_BUFFER, Some(self.vram_buf));
            gl.buffer_data_u8_slice(ARRAY_BUFFER, data, DYNAMIC_DRAW);
        }
    }

    pub(super) fn upload_palette(&self, gl: &Context, data: &[u8]) {
        let colors = data
            .iter()
            .tuples::<(&u8, &u8)>()
            .map(|(b1, b2)| u16::from_le_bytes([*b1, *b2]))
            .map(Abgr1555)
            .map(Rgba::from)
            .flat_map(|color| color.to_array())
            .collect_vec();
        unsafe {
            gl.bind_buffer(ARRAY_BUFFER, Some(self.palette_buf));
            gl.buffer_data_u8_slice(ARRAY_BUFFER, colors.align_to().1, DYNAMIC_DRAW);
        }
    }

    pub(super) fn upload_level(&mut self, gl: &Context, cpu: &mut Cpu) {
        self.layer1.load_layer(gl, cpu);
    }
}

fn make_buffer(gl: &Context, size: i32, index: u32) -> Buffer {
    unsafe {
        let buf = gl.create_buffer().expect("Failed to create buffer");
        gl.bind_buffer(ARRAY_BUFFER, Some(buf));
        gl.buffer_data_size(ARRAY_BUFFER, size, DYNAMIC_DRAW);
        gl.bind_buffer_base(UNIFORM_BUFFER, index, Some(buf));
        buf
    }
}
