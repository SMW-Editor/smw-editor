use std::fmt::Write;

use imgui::{ImColor32, Ui, Window};
use smwe_rom::disassembler::rom_disassembly::BinaryBlock;

use crate::{
    frame_context::FrameContext,
    ui::{title_with_id, UiTool, WindowId},
};

pub struct UiDisassembler {
    title:                  String,
    current_address_scroll: u32,
}

impl UiTool for UiDisassembler {
    fn tick(&mut self, ctx: &mut FrameContext) -> bool {
        let mut running = true;

        let ui = ctx.ui;
        let title = std::mem::take(&mut self.title);
        Window::new(&title)
            .always_auto_resize(false)
            .resizable(true)
            .collapsible(true)
            .scroll_bar(false)
            .size_constraints([512.0, 128.0], [1.0e9, 1.0e9])
            .opened(&mut running)
            .build(ui, || {
                self.display_code(ctx);
            });
        self.title = title;

        if !running {
            log::info!("Closed disassembler");
        }
        running
    }
}

impl UiDisassembler {
    pub fn new(id: WindowId) -> Self {
        log::info!("Opened disassembler");
        Self { title: title_with_id("Disassembler", id), current_address_scroll: 0 }
    }

    pub fn display_code(&mut self, ctx: &mut FrameContext) {
        let project = ctx.project_ref.as_ref().unwrap().borrow();
        let disas = &project.rom_data.disassembly;
        let ui = ctx.ui;
        let [available_w, available_h] = ui.content_region_avail();
        let [xoff, yoff] = ui.cursor_screen_pos();
        {
            let mut str_buf = String::with_capacity(256);
            let mut addr = self.current_address_scroll as usize;
            let x = 24.0;
            let mut y = 8.0;
            let yadv = ui.text_line_height_with_spacing();
            // VSliders are upside down in imgui
            let mut virtual_address_scroll = disas.rom_bytes().len() as u32 - self.current_address_scroll;
            imgui::VerticalSlider::new("", [16.0, available_h - 16.0], 16u32, disas.rom_bytes().len() as u32)
                .flags(imgui::SliderFlags::ALWAYS_CLAMP)
                .display_format("")
                .build(ui, &mut virtual_address_scroll);
            self.current_address_scroll = (disas.rom_bytes().len() as u32 - virtual_address_scroll) & !3;
            let draw_list = ui.get_window_draw_list();
            while y < available_h - yadv {
                let bytes = if let Some(bytes) = disas.rom_bytes().get(addr..addr + 4) { bytes } else { break };
                str_buf.clear();
                write!(str_buf, "{:06x}: ", addr).unwrap();
                draw_list.add_text([xoff + x, yoff + y], ImColor32::from(0xff_aa_aa_aa), &str_buf);
                let [xadv, _] = ui.calc_text_size(&str_buf);
                str_buf.clear();
                write!(str_buf, "{:02x} {:02x} {:02x} {:02x}", bytes[0], bytes[1], bytes[2], bytes[3]).unwrap();
                draw_list.add_text([xoff + x + xadv, yoff + y], ImColor32::from(0xff_dd_dd_dd), &str_buf);
                y += yadv;
                addr += 4;
            }
        }
    }
}
