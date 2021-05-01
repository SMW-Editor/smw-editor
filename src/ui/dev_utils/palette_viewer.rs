use crate::{
    frame_context::FrameContext,
    ui::{
        title_with_id,
        UiTool,
        WindowId,
    },
};

use imgui::{
    im_str,
    ImString,
    Window,
};

use nsmwe_rom::graphics::{
    color::Rgba32,
    palette::ColorPalette,
};

pub struct UiPaletteViewer {
    title: ImString,
    level_num: i32,
}

impl UiTool for UiPaletteViewer {
    fn tick(&mut self, ctx: &mut FrameContext) -> bool {
        let mut running = true;

        let title = std::mem::take(&mut self.title);
        Window::new(&title)
            .always_auto_resize(true)
            .resizable(false)
            .collapsible(false)
            .content_size([325.0, 355.0])
            .scroll_bar(false)
            .opened(&mut running)
            .build(ctx.ui, || {
                if ctx.ui.input_int(im_str!("Level number"), &mut self.level_num)
                    .chars_hexadecimal(true)
                    .build()
                {
                    self.adjust_level_num(ctx);
                    log::info!("Showing color palette for level {:X}", self.level_num);
                }
                self.display_palette(ctx);
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
            title: title_with_id("Color palettes", id),
            level_num: 0,
        }
    }

    fn adjust_level_num(&mut self, ctx: &mut FrameContext) {
        let project = ctx.project_ref.as_ref().unwrap().borrow();
        let level_count = project.rom_data.levels.len() as i32;
        self.level_num = self.level_num.rem_euclid(level_count);
    }

    fn display_palette(&mut self, ctx: &mut FrameContext) {
        const CELL_SIZE: f32 = 20.0;
        const PADDING_TOP: f32 = 60.0;
        const PADDING_LEFT: f32 = 10.0;

        let FrameContext {
            ui,
            project_ref,
            ..
        } = ctx;
        let project = project_ref.as_ref().unwrap().borrow();
        let rom = &project.rom_data;

        let header = &rom.levels[self.level_num as usize].primary_header;
        let palette = &rom.level_color_palette_set
            .get_level_palette(header, &rom.global_level_color_palette)
            .unwrap();
        let draw_list = ui.get_window_draw_list();
        let [wx, wy] = ui.window_pos();

        for row in 0..=0xF {
            for col in 0..=0xF {
                let color = palette.get_color_at(row, col).unwrap();

                let y = wy + (row as f32 * CELL_SIZE) + PADDING_TOP;
                let x = wx + (col as f32 * CELL_SIZE) + PADDING_LEFT;

                let p1 = [x, y];
                let p2 = [x + CELL_SIZE, y + CELL_SIZE];
                let c: [f32; 4] = Rgba32::from(color).into();

                draw_list.add_rect(p1, p2, c).filled(true).build();
            }
        }
    }
}
