use std::convert::TryFrom;

use imgui::{im_str, ComboBox, ImString, Ui, Window};
use num_enum::TryFromPrimitive;
use smwe_rom::graphics::{
    color::Rgba32,
    palette::{ColorPalette, OverworldState},
};

use crate::{
    frame_context::FrameContext,
    ui::{title_with_id, UiTool, WindowId},
};

#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, TryFromPrimitive)]
pub enum PaletteContext {
    Level     = 0,
    Overworld = 1,
}

pub struct UiPaletteViewer {
    title:             ImString,
    palette_context:   PaletteContext,
    // Level viewer
    level_num:         i32,
    // Overworld viewer
    submap_num:        i32,
    special_completed: bool,
}

impl UiTool for UiPaletteViewer {
    fn tick(&mut self, ctx: &mut FrameContext) -> bool {
        let mut running = true;

        let title = std::mem::take(&mut self.title);
        Window::new(&title)
            .always_auto_resize(true)
            .resizable(false)
            .collapsible(false)
            .scroll_bar(false)
            .opened(&mut running)
            .build(ctx.ui, || {
                let mut context_raw = self.palette_context as usize;
                ComboBox::new(im_str!("Context"))
                    .build_simple_string(ctx.ui, &mut context_raw, &[im_str!("Level"), im_str!("Overworld")]);
                self.palette_context = PaletteContext::try_from(context_raw).unwrap_or(PaletteContext::Level);
                match self.palette_context {
                    PaletteContext::Level => self.viewer_level(ctx),
                    PaletteContext::Overworld => self.viewer_overworld(ctx),
                }
            });
        self.title = title;

        if !running {
            log::info!("Closed Palette Viewer");
        }
        running
    }
}

impl UiPaletteViewer {
    pub fn new(id: WindowId) -> Self {
        log::info!("Opened Palette Viewer");
        UiPaletteViewer {
            title:             title_with_id("Color palettes", id),
            palette_context:   PaletteContext::Level,
            level_num:         0,
            submap_num:        0,
            special_completed: false,
        }
    }

    fn adjust_level_num(&mut self, ctx: &mut FrameContext) {
        let project = ctx.project_ref.as_ref().unwrap().borrow();
        let level_count = project.rom_data.levels.len() as i32;
        self.level_num = self.level_num.rem_euclid(level_count);
    }

    fn adjust_ow_submap_num(&mut self, ctx: &mut FrameContext) {
        let project = ctx.project_ref.as_ref().unwrap().borrow();
        let submap_count = project.rom_data.color_palettes.ow_specific_set.layer2_indices.len() as i32;
        self.submap_num = self.submap_num.rem_euclid(submap_count);
    }

    fn viewer_level(&mut self, ctx: &mut FrameContext) {
        if ctx.ui.input_int(im_str!("Level number"), &mut self.level_num).chars_hexadecimal(true).build() {
            self.adjust_level_num(ctx);
            log::info!("Showing color palette for level {:X}", self.level_num);
        }
        ctx.ui.new_line();
        self.display_level_palette(ctx);
    }

    fn viewer_overworld(&mut self, ctx: &mut FrameContext) {
        if ctx.ui.checkbox(im_str!("Special world completed"), &mut self.special_completed) {
            log::info!(
                "Showing color palette for {}",
                if self.special_completed { "post-special world" } else { "pre-special world" }
            );
        }
        if ctx.ui.input_int(im_str!("Submap number"), &mut self.submap_num).chars_hexadecimal(true).build() {
            self.adjust_ow_submap_num(ctx);
            log::info!("Showing color palette for submap {:X}", self.submap_num);
        }
        self.display_overworld_palette(ctx);
    }

    fn display_level_palette(&mut self, ctx: &mut FrameContext) {
        let FrameContext { ui, project_ref, .. } = ctx;
        let project = project_ref.as_ref().unwrap().borrow();
        let rom = &project.rom_data;

        let header = &rom.levels[self.level_num as usize].primary_header;
        let palette = &rom.color_palettes.get_level_palette(header).unwrap();
        self.display_palette(ui, palette);
    }

    fn display_overworld_palette(&mut self, ctx: &mut FrameContext) {
        let FrameContext { ui, project_ref, .. } = ctx;
        let project = project_ref.as_ref().unwrap().borrow();
        let rom = &project.rom_data;

        let ow_state = if self.special_completed { OverworldState::PostSpecial } else { OverworldState::PreSpecial };
        let palette = &rom.color_palettes.get_submap_palette(self.submap_num as usize, ow_state).unwrap();
        self.display_palette(ui, palette);
    }

    fn display_palette(&mut self, ui: &Ui, palette: &dyn ColorPalette) {
        const CELL_SIZE: f32 = 20.0;

        let draw_list = ui.get_window_draw_list();
        let [wx, wy] = ui.cursor_screen_pos();

        for row in 0..=0xF {
            for col in 0..=0xF {
                let color = palette.get_color_at(row, col).unwrap();

                let y = wy + (row as f32 * CELL_SIZE);
                let x = wx + (col as f32 * CELL_SIZE);

                let p1 = [x, y];
                let p2 = [x + CELL_SIZE, y + CELL_SIZE];
                let c: [f32; 4] = Rgba32::from(color).into();

                draw_list.add_rect(p1, p2, c).filled(true).build();
            }
        }
        ui.set_cursor_screen_pos([wx + 16.0 * CELL_SIZE, wy + 16.0 * CELL_SIZE]);
    }
}
