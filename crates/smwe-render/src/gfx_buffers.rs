use epaint::Rgba;
use glow::*;
use itertools::Itertools;

use crate::color::Abgr1555;

#[derive(Copy, Clone, Debug)]
pub struct GfxBuffers {
    pub palette_buf: Buffer,
    pub vram_buf:    Buffer,
}

impl GfxBuffers {
    pub fn new(gl: &Context) -> Self {
        let make_buffer = |size, index| unsafe {
            let buf = gl.create_buffer().expect("Failed to create buffer");
            gl.bind_buffer(ARRAY_BUFFER, Some(buf));
            gl.buffer_data_size(ARRAY_BUFFER, size, DYNAMIC_DRAW);
            gl.bind_buffer_base(UNIFORM_BUFFER, index, Some(buf));
            buf
        };
        let palette_buf = make_buffer(256 * 16, 0);
        let vram_buf = make_buffer(0x2000, 1);
        Self::from_buffers(palette_buf, vram_buf)
    }

    pub fn from_buffers(palette_buf: Buffer, vram_buf: Buffer) -> Self {
        Self { palette_buf, vram_buf }
    }

    pub fn destroy(&self, gl: &Context) {
        unsafe {
            gl.delete_buffer(self.vram_buf);
            gl.delete_buffer(self.palette_buf);
        }
    }

    pub fn upload_vram(&self, gl: &Context, data: &[u8]) {
        unsafe {
            gl.bind_buffer(ARRAY_BUFFER, Some(self.vram_buf));
            gl.buffer_data_u8_slice(ARRAY_BUFFER, data, DYNAMIC_DRAW);
        }
    }

    pub fn upload_palette(&self, gl: &Context, data: &[u8]) {
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
}
