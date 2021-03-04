use crate::UiTool;

use imgui::{
    im_str,
    Ui,
    Window,
};

use inline_tweak::tweak;

use nsmwe_rom::{
    graphics::{
        color::Rgba,
        palette::{
            ColorPalette,
            GlobalLevelColorPalette,
            LevelColorPaletteSet,
        },
    },
    level::Level,
};

use std::rc::Rc;

pub struct UiPaletteViewer {
    palettes: LevelColorPaletteSet,
    global_palette: Rc<GlobalLevelColorPalette>,
    levels: Vec<Level>,
    level_num: i32,
}

impl UiTool for UiPaletteViewer {
    fn run(&mut self, ui: &Ui) -> bool {
        let mut running = true;

        Window::new(im_str!("Color palettes"))
            .always_auto_resize(true)
            .resizable(false)
            .collapsible(false)
            .content_size([tweak!(325.0), tweak!(355.0)])
            .scroll_bar(false)
            .opened(&mut running)
            .build(ui, || {
                if ui.input_int(im_str!("Level number"), &mut self.level_num)
                    .chars_hexadecimal(true)
                    .build()
                {
                    log::info!("Showing color palette for level {:X}", self.level_num);
                    self.adjust_level_num();
                }
                self.display_palette(ui);
            });

        if !running {
            log::info!("Closed Palette Viewer");
        }
        running
    }
}

impl UiPaletteViewer {
    pub fn new(palettes: &LevelColorPaletteSet, levels: &[Level], gp: &Rc<GlobalLevelColorPalette>) -> Self {
        log::info!("Opened Palette Viewer");
        UiPaletteViewer {
            palettes: palettes.clone(),
            global_palette: Rc::clone(gp),
            levels: levels.to_vec(),
            level_num: 0,
        }
    }

    fn adjust_level_num(&mut self) {
        if self.level_num < 0 {
            self.level_num = self.levels.len() as i32 - 1;
        } else if self.level_num >= self.levels.len() as i32 {
            self.level_num = 0;
        }
    }

    fn display_palette(&mut self, ui: &Ui) {
        const CELL_SIZE: f32 = 20.0;
        const PADDING_TOP: f32 = 60.0;
        const PADDING_LEFT: f32 = 10.0;

        let header = &self.levels[self.level_num as usize].primary_header;
        let palette = &self.palettes.get_level_palette(header, &self.global_palette).unwrap();
        let draw_list = ui.get_window_draw_list();
        let [wx, wy] = ui.window_pos();

        for row in 0..=0xF {
            for col in 0..=0xF {
                let color = palette.get_color_at(row, col).unwrap();

                let y = wy + (row as f32 * CELL_SIZE) + PADDING_TOP;
                let x = wx + (col as f32 * CELL_SIZE) + PADDING_LEFT;

                let p1 = [x, y];
                let p2 = [x + CELL_SIZE, y + CELL_SIZE];
                let c: [f32; 4] = Rgba::from(*color).into();

                draw_list.add_rect(p1, p2, c).filled(true).build();
            }
        }
    }
}
