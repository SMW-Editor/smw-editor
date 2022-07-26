use std::collections::BTreeMap;

use eframe::egui::{self, TextEdit, TopBottomPanel, Ui, Window};
use smwe_rom::snes_utils::addr::{Addr, AddrPc, AddrSnes};

use crate::{frame_context::EFrameContext, ui_new::tool::UiTool};

pub struct UiDisassembler {
    current_address_scroll: u32,
    address_y_map:          BTreeMap<AddrSnes, f32>,
    opt_draw_debug_info:    bool,
}

impl Default for UiDisassembler {
    fn default() -> Self {
        log::info!("Opened disassembler");
        Self {
            current_address_scroll: AddrSnes::MIN.0 as u32,
            address_y_map:          BTreeMap::new(),
            opt_draw_debug_info:    false,
        }
    }
}

impl UiTool for UiDisassembler {
    fn update(&mut self, ui: &mut Ui, ctx: &mut EFrameContext) -> bool {
        let mut running = true;

        Window::new("Disassembler") //
            .min_width(512.0)
            .min_height(128.0)
            .vscroll(true)
            .open(&mut running)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                TopBottomPanel::top("disasm_switches_panel").show_inside(ui, |ui| self.switches(ui, ctx));
                self.display_code(ui, ctx);
            });

        if !running {
            log::info!("Closed disassembler");
        }
        running
    }
}

impl UiDisassembler {
    fn switches(&mut self, ui: &mut Ui, ctx: &mut EFrameContext) {
        let project = ctx.project_ref.as_ref().unwrap().borrow();
        let disasm = &project.rom_data.disassembly;

        ui.checkbox(&mut self.opt_draw_debug_info, "Draw debug info");

        // TODO: enable the following with the next version of egui
        // ui.add(egui::DragValue::new(&mut self.current_address_scroll)
        //     .clamp_range({
        //         let min = AddrSnes::MIN;
        //         let max = AddrSnes::try_from_lorom(AddrPc(disasm.rom_bytes().len())).unwrap();
        //         min.0 ..= max.0 - 1
        //     })
        //     .prefix("$")
        //     .custom_formatter(|n, _| format!("{:06X}", n as i64)));
        // ui.label("Address");

        // TODO: delete the following after the above gets enabled
        ui.horizontal(|ui| {
            let mut addr_buf = format!("{:06X}", self.current_address_scroll);
            let mut addr_changed = false;
            if ui.add(TextEdit::singleline(&mut addr_buf).desired_width(50.0)).changed() {
                addr_buf.retain(|c| "0123456789abcdef".contains(c));
                self.current_address_scroll = u32::from_str_radix(&addr_buf, 16).unwrap();
                addr_changed = true;
            }
            if ui.button("+").clicked() {
                self.current_address_scroll += 4;
                addr_changed = true;
            }
            if ui.button("-").clicked() {
                self.current_address_scroll -= 4;
                addr_changed = true;
            }
            ui.label("Address");

            if addr_changed {
                self.current_address_scroll = self.current_address_scroll.clamp(
                    AddrSnes::MIN.0 as u32,
                    AddrSnes::try_from_lorom(AddrPc(disasm.rom_bytes().len())).unwrap().0 as u32,
                );
                log::info!("Changed address to: ${:06X}", self.current_address_scroll);
            }
        });
    }

    fn display_code(&mut self, ui: &mut Ui, ctx: &mut EFrameContext) {
        let mut buf = "".to_string();
        ui.text_edit_multiline(&mut buf);
    }
}
