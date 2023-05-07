use egui::{Rgba, Vec2};
use glow::*;
use itertools::Itertools;
use smwe_emu::Cpu;
use smwe_render::tile_renderer::{Tile, TileRenderer};
use smwe_rom::graphics::color::Abgr1555;

pub(super) struct LevelRenderer {
    layer1:  TileRenderer,
    layer2:  TileRenderer,
    sprites: TileRenderer,

    palette_buf: Buffer,
    vram_buf:    Buffer,

    offset:    Vec2,
    destroyed: bool,
}

impl LevelRenderer {
    pub(super) fn new(gl: &Context) -> Self {
        let layer1 = TileRenderer::new(gl);
        let layer2 = TileRenderer::new(gl);
        let sprites = TileRenderer::new(gl);

        let palette_buf = make_buffer(gl, 256 * 16, 0);
        let vram_buf = make_buffer(gl, 0x2000, 1);

        Self { layer1, layer2, sprites, palette_buf, vram_buf, offset: Vec2::splat(0.), destroyed: false }
    }

    pub(super) fn destroy(&mut self, gl: &Context) {
        unsafe {
            gl.delete_buffer(self.vram_buf);
            gl.delete_buffer(self.palette_buf);
        }
        self.layer1.destroy(gl);
        self.layer2.destroy(gl);
        self.destroyed = true;
    }

    pub(super) fn paint(&self, gl: &Context, screen_size: Vec2) {
        if self.destroyed {
            return;
        }
        self.layer2.paint(gl, self.palette_buf, self.vram_buf, screen_size, self.offset);
        self.layer1.paint(gl, self.palette_buf, self.vram_buf, screen_size, self.offset);
        self.sprites.paint(gl, self.palette_buf, self.vram_buf, screen_size, self.offset);
    }

    pub(super) fn upload_gfx(&self, gl: &Context, data: &[u8]) {
        if self.destroyed {
            return;
        }
        unsafe {
            gl.bind_buffer(ARRAY_BUFFER, Some(self.vram_buf));
            gl.buffer_data_u8_slice(ARRAY_BUFFER, data, DYNAMIC_DRAW);
        }
    }

    pub(super) fn upload_palette(&self, gl: &Context, data: &[u8]) {
        if self.destroyed {
            return;
        }
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
        if self.destroyed {
            return;
        }
        self.load_layer(gl, cpu, false);
        self.load_layer(gl, cpu, true);
    }

    pub(super) fn upload_sprites(&mut self, gl: &Context, cpu: &mut Cpu) {
        if self.destroyed {
            return;
        }
        self.load_sprites(gl, cpu);
    }

    pub(super) fn set_offset(&mut self, offset: Vec2) {
        if self.destroyed {
            return;
        }
        self.offset = offset;
    }

    fn load_sprites(&mut self, gl: &Context, cpu: &mut Cpu) {
        let mut tiles = Vec::new();
        for spr in (0..64).rev() {
            let mut x = cpu.mem.load_u8(0x300 + spr * 4) as u32;
            let mut y = cpu.mem.load_u8(0x301 + spr * 4) as u32;
            if y >= 0xE0 {
                continue;
            }
            x += cpu.mem.load_u16(0x1A) as u32;
            y += cpu.mem.load_u16(0x1C) as u32;
            let tile = cpu.mem.load_u16(0x302 + spr * 4);
            let size = cpu.mem.load_u8(0x460 + spr);
            if size & 0x01 != 0 {
                if x > 0x80 {
                    x = x.wrapping_sub(256);
                } else {
                    x = x.wrapping_add(256);
                }
            }
            if size & 0x02 != 0 {
                let (xn, xf) = if tile & 0x4000 == 0 { (0, 8) } else { (8, 0) };
                let (yn, yf) = if tile & 0x8000 == 0 { (0, 8) } else { (8, 0) };
                tiles.push(sp_tile(x.wrapping_add(xn), y.wrapping_add(yn), tile));
                tiles.push(sp_tile(x.wrapping_add(xf), y.wrapping_add(yn), tile + 1));
                tiles.push(sp_tile(x.wrapping_add(xn), y.wrapping_add(yf), tile + 16));
                tiles.push(sp_tile(x.wrapping_add(xf), y.wrapping_add(yf), tile + 17));
                //Self::draw_tile_sp(cpu, &mut new_image, tile + 1, x + xf, y + yn);
                //Self::draw_tile_sp(cpu, &mut new_image, tile + 16, x + xn, y + yf);
                //Self::draw_tile_sp(cpu, &mut new_image, tile + 17, x + xf, y + yf);
            } else {
                tiles.push(sp_tile(x, y, tile));
            }
        }
        self.sprites.set_tiles(gl, tiles);
    }

    fn load_layer(&mut self, gl: &Context, cpu: &mut Cpu, bg: bool) {
        let mut tiles = Vec::new();
        let map16_bank = cpu.mem.cart.resolve("Map16Common").expect("Cannot resolve Map16Common") & 0xFF0000;
        let map16_bg = cpu.mem.cart.resolve("Map16BGTiles").expect("Cannot resolve Map16BGTiles");
        let vertical = cpu.mem.load_u8(0x5B) & if bg { 2 } else { 1 } != 0;
        let has_layer2 = {
            let mode = cpu.mem.load_u8(0x1925);
            let renderer_table = cpu.mem.cart.resolve("CODE_058955").unwrap() + 9;
            let renderer = cpu.mem.load_u24(renderer_table + (mode as u32) * 3);
            let l2_renderers = [cpu.mem.cart.resolve("CODE_058B8D"), cpu.mem.cart.resolve("CODE_058C71")];
            l2_renderers.contains(&Some(renderer))
        };
        let scr_len = match (vertical, has_layer2) {
            (false, false) => 0x20,
            (true, false) => 0x1C,
            (false, true) => 0x10,
            (true, true) => 0x0E,
        };
        let scr_size = if vertical { 16 * 32 } else { 16 * 27 };
        let (blocks_lo_addr, blocks_hi_addr) = match (bg, has_layer2) {
            (true, true) => {
                let offset = scr_len * scr_size;
                (0x7EC800 + offset, 0x7FC800 + offset)
            }
            (true, false) => (0x7EB900, 0x7EBD00),
            (false, _) => (0x7EC800, 0x7FC800),
        };
        let len = if has_layer2 { 256 * 27 } else { 512 * 27 };
        for idx in 0..len {
            let (block_x, block_y) = if vertical {
                let (screen, sidx) = (idx / (16 * 16), idx % (16 * 16));
                let (row, column) = (sidx / 16, sidx % 16);
                let (sub_y, sub_x) = (screen / 2, screen % 2);
                (column * 16 + sub_x * 256, row * 16 + sub_y * 256)
            } else {
                let (screen, sidx) = (idx / (16 * 27), idx % (16 * 27));
                let (row, column) = (sidx / 16, sidx % 16);
                (column * 16 + screen * 256, row * 16)
            };
            let idx = if bg & !has_layer2 { idx % (16 * 27 * 2) } else { idx };
            let block_id =
                cpu.mem.load_u8(blocks_lo_addr + idx) as u16 | ((cpu.mem.load_u8(blocks_hi_addr + idx) as u16) << 8);
            let block_ptr = if bg & !has_layer2 {
                block_id as u32 * 8 + map16_bg
            } else {
                cpu.mem.load_u16(0x0FBE + block_id as u32 * 2) as u32 + map16_bank
            };
            for (tile_id, (off_x, off_y)) in (0..4).zip([(0, 0), (0, 8), (8, 0), (8, 8)].into_iter()) {
                let tile_id = cpu.mem.load_u16(block_ptr + tile_id * 2);
                tiles.push(bg_tile(block_x + off_x, block_y + off_y, tile_id));
            }
        }
        if bg {
            self.layer2.set_tiles(gl, tiles);
        } else {
            self.layer1.set_tiles(gl, tiles);
        }
    }
}

fn bg_tile(x: u32, y: u32, t: u16) -> Tile {
    let t = t as u32;
    let tile = t & 0x3FF;
    let scale = 8;
    let pal = (t >> 10) & 0x7;
    let params = scale | (pal << 8) | (t & 0xC000);
    Tile([x, y, tile, params])
}

fn sp_tile(x: u32, y: u32, t: u16) -> Tile {
    let t = t as u32;
    let tile = (t & 0x1FF) + 0x600;
    let scale = 8;
    let pal = ((t >> 9) & 0x7) + 8;
    let params = scale | (pal << 8) | (t & 0xC000);
    Tile([x, y, tile, params])
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
