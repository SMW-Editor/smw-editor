use crate::{
    frame_context::FrameContext,
    ui::{title_with_id, UiTool, WindowId},
};

use glium::{
    backend::Facade,
    Rect,
    texture::{ClientFormat, RawImage2d},
    uniforms::{
        MagnifySamplerFilter,
        MinifySamplerFilter,
        SamplerBehavior,
    },
    Texture2d,
};

use imgui::{
    Image,
    im_str,
    ImString,
    TextureId,
    Window,
};
use imgui_glium_renderer::Texture;

use nsmwe_rom::graphics::{
    color::Rgba32,
    gfx_file::N_PIXELS_IN_TILE,
    palette::ColorPalette,
};

use std::{
    array::IntoIter,
    borrow::Cow,
    rc::Rc,
};

// -------------------------------------------------------------------------------------------------

#[allow(dead_code)]
mod constants {
    use imgui::{ImStr, im_str};

    pub const N_TILES_IN_ROW: usize = 16;

    pub const I_PAL_LEVEL_WTF:        usize = 0;
    pub const I_PAL_LEVEL_PLAYERS:    usize = 1;
    pub const I_PAL_LEVEL_LAYER3:     usize = 2;
    pub const I_PAL_LEVEL_BERRY:      usize = 3;
    pub const I_PAL_LEVEL_BACKGROUND: usize = 4;
    pub const I_PAL_LEVEL_FOREGROUND: usize = 5;
    pub const I_PAL_LEVEL_SPRITE:     usize = 6;
    pub const PALETTE_CATEGORIES: [&ImStr; 7] = [
        im_str!("Level: Wtf"),
        im_str!("Level: Players"),
        im_str!("Level: Layer3"),
        im_str!("Level: Berry"),
        im_str!("Level: Background"),
        im_str!("Level: Foreground"),
        im_str!("Level: Sprite"),
    ];
}
use constants::*;

// -------------------------------------------------------------------------------------------------

struct BufferInfo {
    pub texture_id: TextureId,
    pub size: (u32, u32),
    pub file_num: i32,
}

pub struct UiGfxViewer {
    title: ImString,
    buffer_info: Option<BufferInfo>,
    curr_image_size: (usize, usize),
    curr_gfx_file_num: i32,
    curr_palette_row_idx: i32,
    curr_bg_palette_num: i32,
    curr_fg_palette_num: i32,
    curr_sp_palette_num: i32,
    curr_pl_palette_num: i32,
}

// -------------------------------------------------------------------------------------------------

impl UiTool for UiGfxViewer {
    fn tick(&mut self, ctx: &mut FrameContext) -> bool {
        if self.buffer_info.is_none() {
            self.create_texture(ctx);
            self.update_texture(ctx);
        }

        let mut running = true;

        let title = std::mem::take(&mut self.title);
        Window::new(&title)
            .always_auto_resize(true)
            .resizable(false)
            .collapsible(false)
            .scroll_bar(false)
            .opened(&mut running)
            .build(ctx.ui, || {
                ctx.ui.group(|| self.switches(ctx));
                ctx.ui.same_line(0.0);
                self.gfx_image(ctx);
            });
        self.title = title;

        if !running {
            log::info!("Closed GFX Viewer");
        }
        running
    }
}

impl UiGfxViewer {
    pub fn new(id: WindowId) -> Self {
        log::info!("Opened GFX Viewer");
        UiGfxViewer {
            title: title_with_id("GFX Viewer", id),
            buffer_info: None,
            curr_image_size: (0, 0),
            curr_gfx_file_num: 0,
            curr_palette_row_idx: 0,
            curr_bg_palette_num: 0,
            curr_fg_palette_num: 0,
            curr_sp_palette_num: 0,
            curr_pl_palette_num: 0,
        }
    }

    fn switches(&mut self, ctx: &mut FrameContext) {
        let mut changed_any = false;
        let mut input_int = |label, var| {
            changed_any |= ctx.ui.input_int(label, var)
                .chars_hexadecimal(true)
                .build()
        };

        input_int(im_str!("GFX file number"), &mut self.curr_gfx_file_num);
        input_int(im_str!("Palette row index"), &mut self.curr_palette_row_idx);
        input_int(im_str!("Background palette index"), &mut self.curr_bg_palette_num);
        input_int(im_str!("Foreground palette index"), &mut self.curr_fg_palette_num);
        input_int(im_str!("Sprite palette index"), &mut self.curr_sp_palette_num);
        input_int(im_str!("Player palette index"), &mut self.curr_pl_palette_num);

        if changed_any {
            self.adjust_nums(ctx);
            self.update_texture(ctx);
        }

        ctx.ui.text(format!("{} x {} px", self.curr_image_size.0, self.curr_image_size.1));
    }

    fn gfx_image(&mut self, ctx: &mut FrameContext) {
        if let Some(buf) = &self.buffer_info {
            let (img_w, img_h) = self.curr_image_size;
            let (_, buf_h) = buf.size;
            Image::new(buf.texture_id, [(img_w * 3) as f32, (img_h * 3) as f32])
                .uv1([1.0, img_h as f32 / buf_h as f32])
                .build(ctx.ui);
        }
    }

    fn create_texture(&mut self, ctx: &mut FrameContext) {
        let project = ctx.project_ref.as_ref().unwrap().borrow();

        let max_tile_count = project.rom_data.gfx_files.iter()
            .max_by(|a, b| a.tiles.len().cmp(&b.tiles.len()))
            .expect("Cannot create texture: No GFX files are loaded")
            .tiles.len();
        let row_count = 1 + (max_tile_count / N_TILES_IN_ROW);
        let texture_size = row_count * N_TILES_IN_ROW * N_PIXELS_IN_TILE;
        let width = (8 * N_TILES_IN_ROW) as u32;
        let height = (8 * row_count) as u32;

        let image = RawImage2d {
            data: Cow::Owned(vec![Rgba32::default().as_tuple(); texture_size]),
            format: ClientFormat::F32F32F32F32,
            width, height,
        };
        let gl_texture = Texture2d::new(ctx.display.get_context(), image)
            .expect("Failed to create GL texture.");
        let texture = Texture {
            texture: Rc::new(gl_texture),
            sampler: SamplerBehavior {
                magnify_filter: MagnifySamplerFilter::Nearest,
                minify_filter: MinifySamplerFilter::Nearest,
                ..Default::default()
            },
        };

        self.buffer_info = Some(BufferInfo {
            texture_id: ctx.renderer.textures().insert(texture),
            size: (width, height),
            file_num: -1,
        });

        log::info!("Successfully created a texture buffer (w = {}, h = {}).", width, height);
    }

    fn update_texture(&mut self, ctx: &mut FrameContext) {
        if let Some(buf) = &mut self.buffer_info {
            buf.file_num = self.curr_gfx_file_num;

            let tex_id = buf.texture_id;
            let texture = ctx.renderer.textures().get_mut(tex_id)
                .unwrap_or_else(|| panic!("Texture with id {} does not exist", tex_id.id()))
                .texture.as_ref();

            let project = ctx.project_ref.as_ref().unwrap().borrow();
            let rom = &project.rom_data;
            let gfx_file = &rom.gfx_files[self.curr_gfx_file_num as usize];
            let palette = &rom.level_color_palette_set.palette_from_indices(
                    0,
                    self.curr_bg_palette_num as usize,
                    self.curr_fg_palette_num as usize,
                    self.curr_sp_palette_num as usize,
                    &rom.global_level_color_palette,
                )
                .unwrap()
                .get_row(self.curr_palette_row_idx as usize);

            let img_w = (gfx_file.tiles.len() * 8).clamp(8, N_TILES_IN_ROW * 8);
            let img_h = gfx_file.n_pixels() / (N_TILES_IN_ROW * 8);
            self.curr_image_size = (img_w, img_h);

            for (idx, tile) in gfx_file.tiles.iter().enumerate() {
                let (row, col) = (idx / N_TILES_IN_ROW, idx % N_TILES_IN_ROW);
                let (x, y) = ((col * 8) as u32, (row * 8) as u32);
                let rgba_tile: Vec<f32> = tile.to_rgba(palette)
                    .iter()
                    .flat_map(|c| IntoIter::new(c.as_array()))
                    .collect();
                let image = RawImage2d::from_raw_rgba(rgba_tile, (8, 8));
                let rect = Rect {
                    left: x,
                    bottom: y,
                    width: 8,
                    height: 8,
                };
                texture.write(rect, image);
            }

            log::info!("Showing GFX file {:X}", self.curr_gfx_file_num);
        } else {
            log::error!("Tried to update texture that was never created");
        }
    }

    fn adjust_nums(&mut self, ctx: &mut FrameContext) {
        if let Some(project) = ctx.project_ref.as_ref() {
            let project = project.borrow();
            let rom = &project.rom_data;

            let level_count = rom.gfx_files.len() as i32;
            self.curr_gfx_file_num = self.curr_gfx_file_num.rem_euclid(level_count);

            let palette_row_count = 16i32;
            self.curr_palette_row_idx = self.curr_palette_row_idx.rem_euclid(palette_row_count);

            let bg_pals_count = rom.level_color_palette_set.bg_palettes.len() as i32;
            self.curr_bg_palette_num = self.curr_bg_palette_num.rem_euclid(bg_pals_count);

            let fg_pals_count = rom.level_color_palette_set.fg_palettes.len() as i32;
            self.curr_fg_palette_num = self.curr_fg_palette_num.rem_euclid(fg_pals_count);

            let sp_pals_count = rom.level_color_palette_set.sprite_palettes.len() as i32;
            self.curr_sp_palette_num = self.curr_sp_palette_num.rem_euclid(sp_pals_count);

            let pl_pals_count = rom.global_level_color_palette.players.len() as i32 / 10;
            self.curr_pl_palette_num = self.curr_pl_palette_num.rem_euclid(pl_pals_count);
        }
    }
}
