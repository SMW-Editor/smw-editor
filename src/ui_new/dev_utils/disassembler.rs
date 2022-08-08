use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::ops::Deref;

use eframe::egui::{Color32, Layout, RichText, SidePanel, TextEdit, Ui, Window};
use egui_extras::{Size, TableBuilder};
use inline_tweak::tweak;
use itertools::Itertools;
use smwe_rom::disassembler::binary_block::BinaryBlock;
use smwe_rom::disassembler::instruction::Instruction;
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
                SidePanel::left("disasm_switches_panel").show_inside(ui, |ui| self.switches(ui, ctx));
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
        ui.label("Address");
        ui.horizontal(|ui| {
            let mut addr_buf = format!("{:06X}", self.current_address_scroll);
            let mut addr_changed = false;
            if ui.add(TextEdit::singleline(&mut addr_buf).desired_width(50.0)).changed() {
                addr_buf.retain(|c| "0123456789abcdef".contains(c.to_ascii_lowercase()));
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

            if addr_changed {
                self.current_address_scroll = self.current_address_scroll.clamp(
                    AddrSnes::MIN.0 as u32,
                    AddrSnes::try_from_lorom(AddrPc(disasm.rom_bytes().len())).unwrap().0 as u32,
                );
                log::info!("Changed address to: ${:06X}", self.current_address_scroll);
            }
        });

        ui.checkbox(&mut self.opt_draw_debug_info, "Draw debug info");
    }

    fn display_code(&mut self, ui: &mut Ui, ctx: &mut EFrameContext) {
        const COLOR_ADDRESS: Color32 = Color32::from_rgba_premultiplied(0xaa, 0xaa, 0xaa, 0xff);
        const COLOR_DATA: Color32 = Color32::from_rgba_premultiplied(0xdd, 0xdd, 0xee, 0xff);
        const COLOR_CODE: Color32 = Color32::from_rgba_premultiplied(0xee, 0xdd, 0xdd, 0xff);
        const COLOR_BRANCH_TARGET: Color32 = Color32::from_rgba_premultiplied(0xbb, 0xaa, 0xaa, 0xff);
        const COLOR_CODE_HEX: Color32 = Color32::from_rgba_premultiplied(0xdd, 0xcc, 0xcc, 0xff);
        const COLOR_DEBUG_NOTE: Color32 = Color32::from_rgba_premultiplied(0xee, 0xee, 0x55, 0xff);

        let project = ctx.project_ref.as_ref().unwrap().borrow();
        let disasm = &project.rom_data.disassembly;

        let str_buf = RefCell::new(String::with_capacity(256));

        let write_hex = |bytes: &mut dyn Iterator<Item = u8>| {
            let mut str_buf = str_buf.borrow_mut();
            str_buf.clear();
            let mut num_bytes = 0usize;
            for byte in bytes {
                write!(str_buf, "{:02X} ", byte).unwrap();
                num_bytes += 1;
            }
            (str_buf, num_bytes)
        };

        let curr_pc_addr_scroll = AddrPc::try_from_lorom(AddrSnes(self.current_address_scroll as usize)).unwrap().0;
        let first_block_idx = disasm.chunks.partition_point(|(a, _)| a.0 < curr_pc_addr_scroll).max(1) - 1;
        let mut current_address = curr_pc_addr_scroll;

        let row_height = tweak!(15.0);
        let col_width = tweak!(170.0);
        let total_rows = {
            let spacing = ui.spacing().item_spacing;
            (ui.available_height() / (row_height + spacing.y)) as _
        };

        TableBuilder::new(ui)
            .striped(true)
            .columns(Size::initial(col_width).at_least(col_width), 3)
            .cell_layout(Layout::left_to_right())
            .body(|mut tb| {
                let mut lines_drawn_so_far = 0;
                'draw_lines: for (chunk_idx, (chunk_pc, chunk)) in disasm.chunks.iter().enumerate().skip(first_block_idx) {
                    if lines_drawn_so_far >= total_rows {
                        break 'draw_lines;
                    }

                    let chunk_pc = *chunk_pc;
                    let next_chunk_pc =
                        disasm.chunks.get(chunk_idx + 1).map(|c| c.0).unwrap_or_else(|| AddrPc::from(disasm.rom_bytes().len()));
                    let chunk_bytes = &disasm.rom_bytes()[chunk_pc.0..next_chunk_pc.0];

                    match chunk {
                        BinaryBlock::EndOfRom => break 'draw_lines,
                        BinaryBlock::Unknown | BinaryBlock::Unused | BinaryBlock::Data(_) => {
                            let stride = 8;
                            let skip_lines = (current_address - chunk_pc.0) / stride;
                            let chunks = chunk_bytes.iter().copied().chunks(stride);
                            for (line_number, mut byte_line) in chunks.into_iter().enumerate().skip(skip_lines) {
                                let line_addr_str = {
                                    let pc = AddrPc(chunk_pc.0 + line_number * stride);
                                    let snes = AddrSnes::try_from_lorom(pc).unwrap();
                                    format!("${:06X}", snes.0)
                                };

                                let (bytes_str, num_bytes) = write_hex(&mut byte_line);
                                current_address += num_bytes;

                                tb.row(row_height, |mut tr| {
                                    tr.col(|ui| {
                                        ui.monospace(RichText::new(line_addr_str).color(COLOR_ADDRESS));
                                    });
                                    tr.col(|ui| {
                                        ui.monospace(RichText::new(bytes_str.deref()).color(COLOR_DATA));
                                    });
                                    tr.col(|ui| {});
                                });

                                lines_drawn_so_far += 1;
                                if lines_drawn_so_far >= total_rows {
                                    break 'draw_lines;
                                }
                            }
                        }
                        BinaryBlock::Code(code) => {
                            let first_instruction = code.instructions.partition_point(|i| i.offset.0 < current_address);

                            for ins in code.instructions.iter().copied().skip(first_instruction) {
                                let Instruction { offset: addr, x_flag, m_flag, .. } = ins;

                                let line_addr_str = format!("${:06X}", AddrSnes::try_from_lorom(addr).unwrap().0);

                                let (code_bytes_str, num_bytes) = {
                                    let mut b_it = disasm.rom_bytes().iter().copied().skip(addr.0).take(ins.opcode.instruction_size());
                                    write_hex(&mut b_it)
                                };
                                current_address += num_bytes;

                                let code_str = format!("{}", ins.display());

                                tb.row(row_height, |mut tr| {
                                    tr.col(|ui| {
                                        ui.monospace(RichText::new(line_addr_str).color(COLOR_ADDRESS));
                                    });
                                    tr.col(|ui| {
                                        ui.monospace(RichText::new(code_bytes_str.deref()).color(COLOR_CODE_HEX));
                                    });
                                    tr.col(|ui| {
                                        ui.monospace(RichText::new(code_str.deref()).color(COLOR_CODE));
                                    });
                                });

                                lines_drawn_so_far += 1;
                                if lines_drawn_so_far >= total_rows {
                                    break 'draw_lines;
                                }
                            }
                        }
                    }
                    current_address = next_chunk_pc.0;
                }
            });
    }
}
