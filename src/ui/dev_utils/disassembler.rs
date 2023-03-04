use std::{cell::RefCell, collections::BTreeMap, fmt::Write, ops::Deref};

use eframe::egui::{Align, Color32, DragValue, Layout, Pos2, Rect, RichText, SidePanel, Stroke, Ui, Window};
use egui_extras::{Column, TableBuilder};
use inline_tweak::tweak;
use itertools::Itertools;
use smwe_rom::{
    disassembler::{binary_block::BinaryBlock, instruction::Instruction},
    snes_utils::addr::{Addr, AddrInner, AddrPc, AddrSnes},
};

use crate::{frame_context::FrameContext, ui::tool::UiTool};

pub struct UiDisassembler {
    current_address_scroll: u32,
    address_y_map:          BTreeMap<AddrSnes, f32>,
    // opt_draw_debug_info:    bool,
    branch_arrows:          Vec<BranchArrow>,
}

#[derive(Clone, Default)]
struct BranchArrow {
    source: AddrSnes,
    target: AddrSnes,
}

impl Default for UiDisassembler {
    fn default() -> Self {
        log::info!("Opened disassembler");
        Self {
            current_address_scroll: AddrSnes::MIN.0,
            address_y_map:          BTreeMap::new(),
            // opt_draw_debug_info:    false,
            branch_arrows:          Vec::with_capacity(30),
        }
    }
}

impl UiTool for UiDisassembler {
    fn update(&mut self, ui: &mut Ui, ctx: &mut FrameContext) -> bool {
        let mut running = true;

        Window::new("Disassembler") //
            .min_width(512.0)
            .min_height(128.0)
            .vscroll(true)
            .open(&mut running)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                SidePanel::left("disasm_switches_panel").show_inside(ui, |ui| self.switches(ui, ctx));
                let avail_area = ui.available_rect_before_wrap();
                self.code(ui, ctx);
                self.branch_arrows(ui, ctx, avail_area);
            });

        if !running {
            log::info!("Closed disassembler");
        }
        running
    }
}

impl UiDisassembler {
    fn switches(&mut self, ui: &mut Ui, ctx: &mut FrameContext) {
        let project = ctx.project_ref.as_ref().unwrap().borrow();
        let disasm = &project.rom_data.disassembly;

        ui.horizontal(|ui| {
            ui.add(
                DragValue::new(&mut self.current_address_scroll)
                    .clamp_range({
                        let min = AddrSnes::MIN;
                        let max = AddrSnes::try_from_lorom(AddrPc(disasm.rom_bytes().len() as AddrInner)).unwrap();
                        min.0..=max.0 - 1
                    })
                    .prefix("$")
                    .hexadecimal(6, false, true),
            );
            ui.label("Address");
        });

        // ui.checkbox(&mut self.opt_draw_debug_info, "Draw debug info");
    }

    fn code(&mut self, ui: &mut Ui, ctx: &mut FrameContext) {
        const COLOR_ADDRESS: Color32 = Color32::from_rgba_premultiplied(0xaa, 0xaa, 0xaa, 0xff);
        const COLOR_DATA: Color32 = Color32::from_rgba_premultiplied(0xdd, 0xdd, 0xee, 0xff);
        const COLOR_CODE: Color32 = Color32::from_rgba_premultiplied(0xee, 0xdd, 0xdd, 0xff);
        // const COLOR_BRANCH_TARGET: Color32 = Color32::from_rgba_premultiplied(0xbb, 0xaa, 0xaa, 0xff);
        const COLOR_CODE_HEX: Color32 = Color32::from_rgba_premultiplied(0xdd, 0xcc, 0xcc, 0xff);
        // const COLOR_DEBUG_NOTE: Color32 = Color32::from_rgba_premultiplied(0xee, 0xee, 0x55, 0xff);

        self.address_y_map.clear();

        let project = ctx.project_ref.as_ref().unwrap().borrow();
        let disasm = &project.rom_data.disassembly;

        let str_buf = RefCell::new(String::with_capacity(256));

        let write_hex = |bytes: &mut dyn Iterator<Item = u8>| {
            let mut str_buf = str_buf.borrow_mut();
            str_buf.clear();
            let mut num_bytes = 0;
            for byte in bytes {
                write!(str_buf, "{byte:02X} ").unwrap();
                num_bytes += 1;
            }
            (str_buf, num_bytes)
        };

        let curr_pc_addr_scroll = AddrPc::try_from_lorom(AddrSnes(self.current_address_scroll)).unwrap().0;
        let first_block_idx = disasm.chunks.partition_point(|(a, _)| a.0 < curr_pc_addr_scroll).max(1) - 1;
        let mut current_address = curr_pc_addr_scroll;

        let row_height = tweak!(17.0);
        let header_height = tweak!(30.0);
        let spacing = ui.spacing().item_spacing;
        let total_rows = ((ui.available_height() - header_height) / (row_height + spacing.y)) as _;

        let mut curr_y = ui.cursor().top() + header_height + (0.5 * row_height + spacing.y);

        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(Layout::left_to_right(Align::Min))
            .column(Column::exact(tweak!(90.0)))
            .column(Column::exact(tweak!(170.0)))
            .column(Column::exact(tweak!(250.0)))
            .column(Column::exact(tweak!(50.0)))
            .column(Column::exact(tweak!(70.0)))
            .header(header_height, |mut th| {
                th.col(|ui| {
                    ui.heading("Label");
                });
                th.col(|ui| {
                    ui.heading("Bytes");
                });
                th.col(|ui| {
                    ui.heading("Code");
                });
                th.col(|ui| {
                    ui.heading("A Size");
                });
                th.col(|ui| {
                    ui.heading("X&Y Size");
                });
            })
            .body(|mut tb| {
                let mut lines_drawn_so_far = 0;
                'draw_lines: for (chunk_idx, (chunk_pc, chunk)) in
                    disasm.chunks.iter().enumerate().skip(first_block_idx)
                {
                    if lines_drawn_so_far >= total_rows {
                        break 'draw_lines;
                    }

                    let chunk_pc = *chunk_pc;
                    let next_chunk_pc = disasm
                        .chunks
                        .get(chunk_idx + 1)
                        .map(|c| c.0)
                        .unwrap_or_else(|| AddrPc(disasm.rom_bytes().len() as AddrInner));
                    let chunk_bytes = &disasm.rom_bytes()[chunk_pc.as_index()..next_chunk_pc.as_index()];

                    match chunk {
                        BinaryBlock::EndOfRom => break 'draw_lines,
                        BinaryBlock::Unknown | BinaryBlock::Data(_) => {
                            let stride = 8;
                            let skip_lines = (current_address - chunk_pc.0) / stride;
                            let chunks = chunk_bytes.iter().copied().chunks(stride as usize);
                            for (line_number, mut byte_line) in chunks.into_iter().enumerate().skip(skip_lines as usize)
                            {
                                let line_addr_str = {
                                    let pc = AddrPc(chunk_pc.0 + line_number as AddrInner * stride);
                                    let snes = AddrSnes::try_from_lorom(pc).unwrap();
                                    self.address_y_map.insert(snes, curr_y);
                                    format!("DATA_{:06X}:", snes.0)
                                };

                                let (bytes_str, num_bytes) = write_hex(&mut byte_line);
                                current_address += num_bytes;

                                let data_str = {
                                    let mut s = String::with_capacity(40);
                                    write!(s, ".db ").unwrap();
                                    for byte in bytes_str.split(' ').filter(|s| !s.is_empty()) {
                                        write!(s, "${byte},").unwrap();
                                    }
                                    s.pop().unwrap();
                                    s
                                };

                                tb.row(row_height, |mut tr| {
                                    tr.col(|ui| {
                                        ui.monospace(RichText::new(line_addr_str).color(COLOR_ADDRESS));
                                    });
                                    tr.col(|ui| {
                                        ui.monospace(RichText::new(bytes_str.deref()).color(COLOR_CODE_HEX));
                                    });
                                    tr.col(|ui| {
                                        ui.monospace(RichText::new(data_str).color(COLOR_DATA));
                                    });
                                    tr.col(|_ui| {});
                                    tr.col(|_ui| {});
                                });

                                curr_y += row_height + spacing.y;
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

                                let line_addr_str = {
                                    let snes = AddrSnes::try_from_lorom(addr).unwrap();
                                    self.address_y_map.insert(snes, curr_y);
                                    format!("CODE_{:06X}:", snes.0)
                                };

                                let (code_bytes_str, num_bytes) = {
                                    let mut b_it = disasm
                                        .rom_bytes()
                                        .iter()
                                        .copied()
                                        .skip(addr.as_index())
                                        .take(ins.opcode.instruction_size());
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
                                        ui.monospace(RichText::new(code_str).color(COLOR_CODE));
                                    });
                                    tr.col(|ui| {
                                        ui.monospace(
                                            RichText::new(format!("{}", 8 * (m_flag as u32 + 1))).color(COLOR_CODE),
                                        );
                                    });
                                    tr.col(|ui| {
                                        ui.monospace(
                                            RichText::new(format!("{}", 8 * (x_flag as u32 + 1))).color(COLOR_CODE),
                                        );
                                    });
                                });

                                if ins.is_branch_or_jump() {
                                    let source = addr.try_into().unwrap();
                                    for &target in code.exits.iter() {
                                        self.branch_arrows.push(BranchArrow { source, target });
                                    }
                                }

                                curr_y += row_height + spacing.y;
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

    fn branch_arrows(&mut self, ui: &mut Ui, _ctx: &mut FrameContext, avail_area: Rect) {
        const ARROW_SZ: f32 = 4.0f32;
        const ARROW_SEP: f32 = 3.0f32;

        let first_visible_addr = self.address_y_map.iter().next().map(|(&e, _)| e).unwrap_or_default();
        let branch_colors: [Color32; 6] = [
            Color32::from_rgba_premultiplied(0xaa, 0xaa, 0xee, 0xff),
            Color32::from_rgba_premultiplied(0xaa, 0xee, 0xaa, 0xff),
            Color32::from_rgba_premultiplied(0xee, 0xaa, 0xaa, 0xff),
            Color32::from_rgba_premultiplied(0xee, 0xee, 0xaa, 0xff),
            Color32::from_rgba_premultiplied(0xee, 0xaa, 0xee, 0xff),
            Color32::from_rgba_premultiplied(0xaa, 0xee, 0xee, 0xff),
        ];
        let mut branch_color_it = branch_colors.iter().copied().cycle();
        let mut arrx = avail_area.left() - ARROW_SZ;
        let mut arrows_at_addr: BTreeMap<AddrSnes, i32> = BTreeMap::new();

        for arrow in self.branch_arrows.iter() {
            arrx = (arrx - ARROW_SEP).max(ARROW_SZ);
            let start_arrows = arrows_at_addr.entry(arrow.source).or_insert(0);
            let arrow_ystart = self.address_y_map.get(&arrow.source).copied().unwrap() + (*start_arrows as f32);
            *start_arrows += 1;
            let end_arrows = arrows_at_addr.entry(arrow.target).or_insert(0);
            let target_y = self.address_y_map.get(&arrow.target).copied().map(|v| v + (*end_arrows as f32));
            *end_arrows += 1;
            let arrow_yend =
                target_y.unwrap_or(if arrow.target < first_visible_addr { 0.0f32 } else { avail_area.bottom() });
            let color = branch_color_it.next().unwrap();
            let stroke = Stroke::new(1.0, color);
            ui.painter()
                .line_segment([Pos2::new(arrx, arrow_ystart), Pos2::new(avail_area.left(), arrow_ystart)], stroke);
            ui.painter().line_segment([Pos2::new(arrx, arrow_ystart), Pos2::new(arrx, arrow_yend)], stroke);

            if target_y.is_some() {
                let xoff = avail_area.left() - (*end_arrows - 1) as f32 * ARROW_SEP;

                // - insn
                ui.painter()
                    .line_segment([Pos2::new(arrx, arrow_yend), Pos2::new(avail_area.left(), arrow_yend)], stroke);

                // \ insn
                ui.painter().line_segment(
                    [Pos2::new(xoff - ARROW_SZ, arrow_yend - ARROW_SZ), Pos2::new(xoff, arrow_yend)],
                    stroke,
                );

                // / insn
                ui.painter().line_segment(
                    [Pos2::new(xoff - ARROW_SZ, arrow_yend + ARROW_SZ), Pos2::new(xoff, arrow_yend)],
                    stroke,
                );
            }
        }

        self.branch_arrows.clear();
    }
}
