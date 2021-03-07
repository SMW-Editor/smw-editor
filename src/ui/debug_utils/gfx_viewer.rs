use crate::ui::UiTool;

use imgui::{
    im_str,
    Ui,
    Window,
};
use imgui_glium_renderer::{
    Renderer,
    Texture,
};

use nsmwe_rom::graphics::gfx_file::{
    GfxFile,
    Tile,
    TileFormat,
};

pub struct UiGfxViewer {
    pub gfx_files: Vec<GfxFile>,
    pub curr_gfx_file_num: i32,
    pub texture_buffer: Option<Texture>,
    pub file_in_tex: Option<i32>,
}

impl UiTool for UiGfxViewer {
    fn tick(&mut self, ui: &Ui, renderer: &mut Renderer) -> bool {
        let mut running = true;

        Window::new(im_str!("GFX Viewer"))
            .always_auto_resize(true)
            .resizable(false)
            .collapsible(false)
            .scroll_bar(false)
            .opened(&mut running)
            .build(ui, || {
                if ui.input_int(im_str!("GFX file number"), &mut self.curr_gfx_file_num)
                    .chars_hexadecimal(true)
                    .build()
                {
                    log::info!("Showing GFX file {:X}", self.curr_gfx_file_num);
                    self.adjust_gfx_file_num();
                }
                self.display_curr_gfx_file(ui);
            });

        if !running {
            log::info!("Closed GFX Viewer");
        }
        running
    }
}

impl UiGfxViewer {
    pub fn new(gfx_files: &[GfxFile]) -> Self {
        log::info!("Opened GFX Viewer");
        UiGfxViewer {
            gfx_files: gfx_files.to_vec(),
            curr_gfx_file_num: 0,
            texture_buffer: None,
            file_in_tex: None,
        }
    }

    fn adjust_gfx_file_num(&mut self) {
        if self.curr_gfx_file_num < 0 {
            self.curr_gfx_file_num = self.gfx_files.len() as i32 - 1;
        } else if self.curr_gfx_file_num >= self.gfx_files.len() as i32 {
            self.curr_gfx_file_num = 0;
        }
    }

    fn display_curr_gfx_file(&mut self, ui: &Ui) {

    }
}
