use std::{
    cell::{Cell, RefCell},
    fmt::Write,
};

use imgui::{ImColor32, Window};
use itertools::Itertools;
use smwe_rom::{
    disassembler::rom_disassembly::{BinaryBlock, InstructionMeta},
    snes_utils::addr::{Addr, AddrPc, AddrSnes},
};

use crate::{
    frame_context::FrameContext,
    ui::{title_with_id, UiTool, WindowId},
};

pub struct UiDisassembler {
    title:                  String,
    current_address_scroll: i32,
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
        Self {
            title:                  title_with_id("Disassembler", id),
            current_address_scroll: AddrSnes::MIN.0 as i32,
        }
    }

    pub fn display_code(&mut self, ctx: &mut FrameContext) {
        let project = ctx.project_ref.as_ref().unwrap().borrow();
        let disas = &project.rom_data.disassembly;
        let ui = ctx.ui;

        if ui.input_int("Address", &mut self.current_address_scroll).chars_hexadecimal(true).build() {
            let min = AddrSnes::MIN;
            let max = AddrSnes::try_from_lorom(AddrPc(disas.rom_bytes().len())).unwrap();
            self.current_address_scroll = self.current_address_scroll.clamp(min.0 as i32, max.0 as i32);
        }

        let str_buf = RefCell::new(String::with_capacity(256));
        let draw_list = ui.get_window_draw_list();

        let xstart = 0.0f32;
        let x = Cell::new(xstart);
        let y = Cell::new(8.0f32);
        let yadv = ui.text_line_height_with_spacing();
        let [xoff, yoff] = ui.cursor_screen_pos();
        let [available_w, available_h] = ui.content_region_avail();

        const COLOR_ADDR: u32 = 0xff_aa_aa_aa;
        const COLOR_DATA: u32 = 0xff_ee_dd_dd;
        const COLOR_CODE: u32 = 0xff_dd_dd_ee;
        const COLOR_BRANCH_TARGET: u32 = 0xff_aa_aa_bb;
        const COLOR_CODE_HEX: u32 = 0xff_cc_cc_dd;
        const COLOR_DEBUG_NOTE: u32 = 0xff_55_ee_ee;
        let space_width: f32 = ui.calc_text_size("0")[0];

        let draw_end_line = || {
            x.set(xstart);
            y.set(y.get() + yadv);
            ui.set_cursor_pos([x.get(), y.get()]);
            y.get() >= available_h - yadv
        };
        let draw_chunk_line = || {
            x.set(xstart);
            y.set(y.get() + 4.0);
            ui.set_cursor_pos([x.get(), y.get()]);
            draw_list
                .add_line(
                    [xoff + x.get(), yoff + y.get() - 2.0],
                    [xoff + x.get() + available_w - 8.0, yoff + y.get() - 2.0],
                    COLOR_ADDR,
                )
                .build();
            y.get() >= available_h - yadv
        };
        let draw_text = |text: &str, color: u32| {
            draw_list.add_text([xoff + x.get(), yoff + y.get()], ImColor32::from(color), text);
            let [xadv, _] = ui.calc_text_size(text);
            x.set(x.get() + xadv);
            ui.set_cursor_pos([x.get(), y.get()]);
        };
        let draw_fmt = |fmt: std::fmt::Arguments, color: u32| {
            let mut str_buf = str_buf.borrow_mut();
            str_buf.clear();
            str_buf.write_fmt(fmt).unwrap();
            draw_text(&*str_buf, color);
        };
        let draw_addr = |addr: AddrPc, color: u32| {
            draw_fmt(format_args!("{:06x}: ", AddrSnes::try_from_lorom(addr).unwrap().0), color);
        };
        let draw_hex = |bytes: &mut dyn Iterator<Item = u8>, color: u32| {
            let mut str_buf = str_buf.borrow_mut();
            str_buf.clear();
            let mut num_bytes = 0usize;
            for byte in bytes {
                write!(str_buf, "{:02X} ", byte).unwrap();
                num_bytes += 1;
            }
            draw_text(&*str_buf, color);
            num_bytes
        };

        let curr_pc_addr_scroll = AddrPc::try_from_lorom(AddrSnes(self.current_address_scroll as usize)).unwrap().0;
        let first_block_idx = disas.chunks.partition_point(|(a, _)| a.0 < curr_pc_addr_scroll).max(1) - 1;
        let mut current_address = curr_pc_addr_scroll;
        'draw_lines: for (chunk_idx, (chunk_pc, chunk)) in disas.chunks.iter().enumerate().skip(first_block_idx) {
            let chunk_pc = *chunk_pc;
            let next_chunk_pc =
                disas.chunks.get(chunk_idx + 1).map(|c| c.0).unwrap_or_else(|| AddrPc::from(disas.rom_bytes().len()));
            let chunk_bytes = &disas.rom_bytes()[chunk_pc.0..next_chunk_pc.0];

            draw_fmt(
                format_args!("Chunk of {ty}: {ab}..{ae}", ty = chunk.type_name(), ab = chunk_pc, ae = next_chunk_pc),
                COLOR_DEBUG_NOTE,
            );
            if draw_end_line() {
                break 'draw_lines;
            }
            match chunk {
                BinaryBlock::EndOfRom => break 'draw_lines,
                BinaryBlock::Unknown | BinaryBlock::Unused | BinaryBlock::Data(_) => {
                    let stride = 8;
                    let skip_lines = (current_address - chunk_pc.0) / stride;
                    let chunks = chunk_bytes.iter().copied().chunks(stride);
                    for (line_number, mut byte_line) in chunks.into_iter().enumerate().skip(skip_lines) {
                        draw_addr(AddrPc::from(chunk_pc.0 + line_number * stride), COLOR_ADDR);
                        let num_bytes = draw_hex(&mut byte_line, COLOR_DATA);
                        current_address += num_bytes;
                        if draw_end_line() {
                            break 'draw_lines;
                        }
                    }
                }
                BinaryBlock::Code(code) => {
                    let first_instruction = code.instruction_metas.partition_point(|i| i.offset.0 < current_address);
                    draw_fmt(format_args!("First insn: {}", first_instruction), COLOR_DEBUG_NOTE);
                    if draw_end_line() {
                        break 'draw_lines;
                    }
                    for imeta in code.instruction_metas.iter().copied().skip(first_instruction) {
                        let InstructionMeta { offset: addr, instruction: ins, x_flag, m_flag, direct_page } = imeta;
                        draw_addr(addr, COLOR_ADDR);
                        let num_bytes = draw_hex(
                            &mut disas.rom_bytes().iter().copied().skip(addr.0).take(ins.opcode.instruction_size()),
                            COLOR_CODE_HEX,
                        );
                        x.set(x.get() + space_width * 3.0 * (4 - num_bytes) as f32);
                        draw_fmt(format_args!("{}", ins.display(addr, x_flag, m_flag, direct_page)), COLOR_CODE);
                        if ins.opcode.mnemonic.can_branch() {
                            draw_text(" ->", COLOR_BRANCH_TARGET);
                            debug_assert_eq!(addr, code.instruction_metas.last().unwrap().offset);
                            for target in code.exits.iter() {
                                draw_fmt(format_args!(" {}", target), COLOR_BRANCH_TARGET);
                            }
                        }
                        if draw_end_line() {
                            break 'draw_lines;
                        }
                    }
                }
            }
            current_address = next_chunk_pc.0;
            if draw_chunk_line() {
                break 'draw_lines;
            }
        }
    }
}
