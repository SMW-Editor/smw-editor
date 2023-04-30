mod level_renderer;
mod shaders;

use std::sync::{Arc, Mutex};

use egui::{Button, CentralPanel, DragValue, SidePanel, Ui, WidgetText, *};
use egui_glow::CallbackFn;
use smwe_emu::Cpu;
use smwe_rom::graphics::{color::Abgr1555, gfx_file::Tile};
use smwe_widgets::value_switcher::{ValueSwitcher, ValueSwitcherButtons};

use crate::ui::{
    editor_prototypes::level_editor::level_renderer::LevelRenderer,
    tool::DockableEditorTool,
    EditorState,
};

pub const N_TILES_IN_ROW: usize = 16;

pub struct UiLevelEditor {
    gl:               Arc<glow::Context>,
    level_renderer:   Arc<Mutex<LevelRenderer>>,
    initialized:      bool,
    spr_image_handle: Option<TextureHandle>,
    curr_image_size:  (usize, usize),
    level_num:        u16,
    blue_pswitch:     bool,
    silver_pswitch:   bool,
    on_off_switch:    bool,
    palette_line:     u8,
    anim_frame:       u8,
}

impl UiLevelEditor {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        let level_renderer = Arc::new(Mutex::new(LevelRenderer::new(&gl)));
        Self {
            gl,
            level_renderer,
            initialized: false,
            spr_image_handle: None,
            curr_image_size: (0, 0),
            level_num: 0,
            blue_pswitch: false,
            silver_pswitch: false,
            on_off_switch: false,
            palette_line: 0,
            anim_frame: 0,
        }
    }
}

impl DockableEditorTool for UiLevelEditor {
    fn update(&mut self, ui: &mut Ui, state: &mut EditorState) {
        if !self.initialized {
            self.update_cpu(state);
            self.update_cpu_sprite(state);
            self.update_image(state);
            self.initialized = true;
        }
        SidePanel::left("level_editor.left_panel").resizable(false).show_inside(ui, |ui| self.left_panel(ui, state));
        CentralPanel::default().show_inside(ui, |ui| self.central_panel(ui, state));
    }

    fn title(&self) -> WidgetText {
        "Level Editor".into()
    }

    fn on_closed(&mut self) {
        self.level_renderer.lock().unwrap().destroy(&self.gl);
    }
}

impl UiLevelEditor {
    fn left_panel(&mut self, ui: &mut Ui, state: &mut EditorState) {
        let mut need_update_level = false;
        let mut need_update = false;
        need_update_level |= {
            let switcher = ValueSwitcher::new(&mut self.level_num, "Level", ValueSwitcherButtons::MinusPlus)
                .range(0..=0x1FF)
                .hexadecimal(3, false, true);
            ui.add(switcher).changed()
        };
        need_update |= {
            let switcher = ValueSwitcher::new(&mut self.anim_frame, "Animation Frame", ValueSwitcherButtons::MinusPlus)
                .range(0..=4);
            ui.add(switcher).changed()
        };
        ui.horizontal(|ui| {
            need_update |= ui
                .add(DragValue::new(&mut self.palette_line).clamp_range(0x0..=0xF).hexadecimal(1, false, true))
                .changed();
            ui.label("Palette");
        });
        need_update |= ui.checkbox(&mut self.blue_pswitch, "Blue P-Switch").changed();
        need_update |= ui.checkbox(&mut self.silver_pswitch, "Silver P-Switch").changed();
        need_update |= ui.checkbox(&mut self.on_off_switch, "ON/OFF Switch").changed();
        if ui.button("frame advance").hovered() {
            self.update_cpu_sprite(state);
            // self.draw_sprites(state, ui.ctx());
        }

        if need_update_level {
            self.update_cpu(state);
            self.update_cpu_sprite(state);
        }
        if need_update || need_update_level {
            self.update_image(state);
        }
    }

    fn central_panel(&mut self, ui: &mut Ui, _state: &mut EditorState) {
        // if let Some(handle) = &self.image_handle {
        //     let (img_w, img_h) = self.curr_image_size;
        //     ScrollArea::both().min_scrolled_height(ui.available_height()).show(ui, |ui| {
        //         let image_rect = vec2(img_w as f32, img_h as f32);
        //         let (_id, rect) = ui.allocate_space(image_rect);
        //         ui.image(handle, [(img_w * 1) as f32, (img_h * 1) as f32]);
        //         ui.painter().image(
        //             handle.id(),
        //             rect,
        //             Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        //             Color32::WHITE,
        //         );
        //         ui.painter().image(
        //             self.spr_image_handle.as_ref().unwrap().id(),
        //             rect,
        //             Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        //             Color32::WHITE,
        //         );
        //     });
        // }
        let level_renderer = Arc::clone(&self.level_renderer);
        Frame::canvas(ui.style()).show(ui, |ui| {
            let (rect, _response) =
                ui.allocate_exact_size(vec2(ui.available_width(), (16 * 27) as f32), Sense::click_and_drag());
            ui.painter().add(PaintCallback {
                rect,
                callback: Arc::new(CallbackFn::new(move |_info, painter| {
                    level_renderer
                        .lock()
                        .expect("Cannot lock mutex on level_renderer")
                        .paint(painter.gl(), rect.size());
                })),
            });
        });
    }

    fn update_cpu(&mut self, state: &mut EditorState) {
        let project = state.project.as_ref().unwrap().borrow();
        let mut cpu = Cpu::new(smwe_emu::emu::CheckedMem::new(project.rom.clone()));
        drop(project);
        smwe_emu::emu::decompress_sublevel(&mut cpu, self.level_num);
        println!("Updated CPU");
        self.level_renderer.lock().unwrap().upload_level(&self.gl, &mut cpu);
        state.cpu = Some(cpu);
    }

    fn update_cpu_sprite(&mut self, state: &mut EditorState) {
        let cpu = state.cpu.as_mut().unwrap(); // should be set already
        let m = cpu.mem.load_u8(0x13);
        cpu.mem.store_u8(0x13, m.wrapping_add(1));
        let m = cpu.mem.load_u8(0x14);
        cpu.mem.store_u8(0x14, m.wrapping_add(1));
        cpu.mem.wram[0x300..0x400].fill(0xE0);
        smwe_emu::emu::exec_sprites(cpu);
    }

    fn update_anim_frame(&mut self, state: &mut EditorState) {
        let cpu = state.cpu.as_mut().unwrap(); // should be set already
        cpu.mem.store_u8(0x14AD, self.blue_pswitch as u8);
        cpu.mem.store_u8(0x14AE, self.silver_pswitch as u8);
        cpu.mem.store_u8(0x14AF, self.on_off_switch as u8);
        for i in 0..8 {
            cpu.mem.store_u8(0x14, self.anim_frame * 8 + i);
            smwe_emu::emu::fetch_anim_frame(cpu);
        }
    }

    fn update_image(&mut self, state: &mut EditorState) {
        self.update_anim_frame(state);

        // should be set already
        let cpu = state.cpu.as_mut().unwrap();
        // let mut new_image;
        // let new_spr_image;
        // match 0 {
        //     0 => {
        //         let palette = &cpu.mem.cgram;
        //         let mut data = &cpu.mem.vram[..];
        //         let img_w = 128;
        //         let img_h = 1024;
        //         self.curr_image_size = (img_w, img_h);
        //         let mut tile_idx = 0;
        //         new_image = ColorImage::new([img_w, img_h], Color32::TRANSPARENT);
        //         new_spr_image = ColorImage::new([img_w, img_h], Color32::TRANSPARENT);
        //         while let Ok((rest, tile)) = Tile::from_4bpp(data) {
        //             data = rest;
        //             let (row, column) = (tile_idx / N_TILES_IN_ROW, tile_idx % N_TILES_IN_ROW);
        //             let (tile_x, tile_y) = (column * 8, row * 8);
        //             for (pixel_idx, &id) in tile.color_indices.iter().enumerate() {
        //                 if id == 0 {
        //                     continue;
        //                 }
        //                 let (pixel_x, pixel_y) = (tile_x + (pixel_idx % 8), tile_y + (pixel_idx / 8));
        //                 let id = id as usize * 2 + self.palette_line as usize * 0x20;
        //                 let color = u16::from_le_bytes([palette[id], palette[id + 1]]);
        //                 new_image[(pixel_x, pixel_y)] = Color32::from(Abgr1555(color));
        //             }
        //             tile_idx += 1;
        //         }
        //     }
        //     1 => {
        //         let img_w = 256;
        //         let img_h = 512;
        //         self.curr_image_size = (img_w, img_h);
        //         new_image = ColorImage::new([img_w, img_h], Color32::TRANSPARENT);
        //         new_spr_image = ColorImage::new([img_w, img_h], Color32::TRANSPARENT);
        //         // TODO: symbols
        //         for (idx, i) in (0..512).enumerate() {
        //             let (row, column) = (idx / 16, idx % 16);
        //             let (block_x, block_y) = (column * 16, row * 16);
        //             Self::draw_block(cpu, &mut new_image, i, block_x, block_y);
        //         }
        //     }
        //     2 => {
        //         let img_w = 512 * 16;
        //         let img_h = 16 * 27;
        //         self.curr_image_size = (img_w, img_h);
        //         new_image = ColorImage::new([img_w, img_h], Color32::TRANSPARENT);
        //         new_spr_image = ColorImage::new([img_w, img_h], Color32::TRANSPARENT);
        //         // TODO: symbols
        //         for idx in 0..512 * 27 {
        //             let (screen, sidx) = (idx / (16 * 27), idx % (16 * 27));
        //             let (row, column) = (sidx / 16, sidx % 16);
        //             let (block_x, block_y) = (column * 16 + screen * 256, row * 16);
        //             let block_id = cpu.mem.load_u8(0x7EC800 + idx as u32) as u16
        //                 | ((cpu.mem.load_u8(0x7FC800 + idx as u32) as u16) << 8);
        //             Self::draw_block(cpu, &mut new_image, block_id, block_x, block_y);
        //         }
        //     }
        //     _ => return,
        // }
        // self.image_handle = Some(ctx.load_texture("vram-image", new_image, TextureOptions::NEAREST));
        // self.spr_image_handle = Some(ctx.load_texture("sprite-image", new_spr_image, TextureOptions::NEAREST));
        // log::info!("Successfully created a VRAM file image (w = {img_w}, h = {img_h}).");

        let level_renderer = self.level_renderer.lock().unwrap();
        level_renderer.upload_palette(&self.gl, &cpu.mem.cgram);
        level_renderer.upload_gfx(&self.gl, &cpu.mem.vram);
    }

    fn draw_sprites(&mut self, state: &mut EditorState, ctx: &Context) {
        let cpu = state.cpu.as_mut().unwrap(); // should be set already
        let img_w = 512 * 16;
        let img_h = 16 * 27;
        self.curr_image_size = (img_w, img_h);
        let mut new_image = ColorImage::new([img_w, img_h], Color32::TRANSPARENT);
        // sprites:
        for spr in (0..64).rev() {
            let mut x = cpu.mem.load_u8(0x300 + spr * 4) as usize;
            let mut y = cpu.mem.load_u8(0x301 + spr * 4) as usize;
            if y >= 0xE0 {
                continue;
            }
            x += cpu.mem.load_u16(0x1A) as usize;
            y += cpu.mem.load_u16(0x1C) as usize;
            let tile = cpu.mem.load_u16(0x302 + spr * 4);
            let size = cpu.mem.load_u8(0x460 + spr);
            if size & 0x01 != 0 {
                x = x.wrapping_sub(256);
            }
            if size & 0x02 != 0 {
                let (xn, xf) = if tile & 0x4000 == 0 { (0, 8) } else { (8, 0) };
                let (yn, yf) = if tile & 0x8000 == 0 { (0, 8) } else { (8, 0) };
                Self::draw_tile_sp(cpu, &mut new_image, tile, x + xn, y + yn);
                Self::draw_tile_sp(cpu, &mut new_image, tile + 1, x + xf, y + yn);
                Self::draw_tile_sp(cpu, &mut new_image, tile + 16, x + xn, y + yf);
                Self::draw_tile_sp(cpu, &mut new_image, tile + 17, x + xf, y + yf);
            } else {
                Self::draw_tile_sp(cpu, &mut new_image, tile, x, y);
            }
        }
        self.spr_image_handle = Some(ctx.load_texture("sprite-image", new_image, TextureOptions::NEAREST));
    }

    fn draw_block(cpu: &mut Cpu, img: &mut ColorImage, block_id: u16, x: usize, y: usize) {
        let block_ptr = cpu.mem.load_u16(0x0FBE + block_id as u32 * 2) as u32 + 0x0D0000;
        for (tile_id, (off_x, off_y)) in (0..4).zip([(0, 0), (0, 8), (8, 0), (8, 8)].into_iter()) {
            let tile_id = cpu.mem.load_u16(block_ptr + tile_id * 2);
            Self::draw_tile(cpu, img, tile_id, x + off_x, y + off_y);
        }
    }

    fn draw_tile(cpu: &Cpu, img: &mut ColorImage, tile_id: u16, x: usize, y: usize) {
        let palette = &cpu.mem.cgram;
        let tile = (tile_id & 0x3FF) as usize;
        let pal_line = ((tile_id >> 10) & 0x7) as usize;
        let h = tile_id & 0x4000 != 0;
        let v = tile_id & 0x8000 != 0;
        let data = &cpu.mem.vram[tile * 32..];
        let tile = Tile::from_4bpp(data).unwrap().1;
        for (pixel_idx, &id) in tile.color_indices.iter().enumerate() {
            if id == 0 {
                continue;
            }
            let mut pixel_x = pixel_idx % 8;
            let mut pixel_y = pixel_idx / 8;
            if h {
                pixel_x = 7 - pixel_x;
            }
            if v {
                pixel_y = 7 - pixel_y;
            }
            pixel_x += x;
            pixel_y += y;
            let id = id as usize * 2 + pal_line * 0x20;
            let color = u16::from_le_bytes([palette[id], palette[id + 1]]);
            if pixel_x < img.size[0] && pixel_y < img.size[1] {
                img[(pixel_x, pixel_y)] = Color32::from(Abgr1555(color));
            }
        }
    }

    fn draw_tile_sp(cpu: &Cpu, img: &mut ColorImage, tile_id: u16, x: usize, y: usize) {
        let palette = &cpu.mem.cgram;
        let tile = (tile_id & 0x1FF) as usize;
        let pal_line = ((tile_id >> 9) & 0x7) as usize + 8;
        let h = tile_id & 0x4000 != 0;
        let v = tile_id & 0x8000 != 0;
        let data = &cpu.mem.vram[0xC000 + tile * 32..];
        let tile = Tile::from_4bpp(data).unwrap().1;
        for (pixel_idx, &id) in tile.color_indices.iter().enumerate() {
            if id == 0 {
                continue;
            }
            let mut pixel_x = pixel_idx % 8;
            let mut pixel_y = pixel_idx / 8;
            if h {
                pixel_x = 7 - pixel_x;
            }
            if v {
                pixel_y = 7 - pixel_y;
            }
            pixel_x += x;
            pixel_y += y;
            let id = id as usize * 2 + pal_line * 0x20;
            let color = u16::from_le_bytes([palette[id], palette[id + 1]]);
            if pixel_x < img.size[0] && pixel_y < img.size[1] {
                img[(pixel_x, pixel_y)] = Color32::from(Abgr1555(color));
            }
        }
    }
}
