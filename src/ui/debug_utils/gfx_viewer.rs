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
use imgui_glium_renderer::Texture;

use nsmwe_rom::graphics::gfx_file::{
    GfxFile,
    Tile,
    TileFormat,
};

pub struct UiGfxViewer {
    title: ImString,

    curr_gfx_file_num: i32,
    texture_buffer: Option<Texture>,
    file_in_tex: Option<i32>,
}

impl UiTool for UiGfxViewer {
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
                if ctx.ui.input_int(im_str!("GFX file number"), &mut self.curr_gfx_file_num)
                    .chars_hexadecimal(true)
                    .build()
                {
                    log::info!("Showing GFX file {:X}", self.curr_gfx_file_num);
                    self.adjust_gfx_file_num(ctx);
                }
                self.display_curr_gfx_file(ctx);
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
            curr_gfx_file_num: 0,
            texture_buffer: None,
            file_in_tex: None,
        }
    }

    fn adjust_gfx_file_num(&mut self, ctx: &mut FrameContext) {
        let project = ctx.project_ref.as_ref().unwrap().borrow();
        let level_count = project.rom_data.gfx_files.len() as i32;
        self.curr_gfx_file_num = self.curr_gfx_file_num.rem_euclid(level_count);
    }

    fn display_curr_gfx_file(&mut self, ctx: &mut FrameContext) {

    }
}
