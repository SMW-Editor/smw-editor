use imgui::{
    Condition,
    ImString,
    Window,
    Ui,
    im_str,
};
use inline_tweak::*;
use nsmwe_rom::addr;
use std::cmp::PartialEq;

#[derive(Clone, Copy, PartialEq)]
enum ConversionMode {
    LoRom,
    HiRom,
}

enum ConvDir {
    PcToSnes,
    SnesToPc,
}

pub struct UiAddressConverter {
    conversion_mode: ConversionMode,
    include_header: bool,

    text_pc: ImString,
    text_snes: ImString,
    error: ImString,
}

impl UiAddressConverter {
    pub fn new() -> Self {
        UiAddressConverter {
            conversion_mode: ConversionMode::LoRom,
            include_header: false,
            text_pc: ImString::new("0"),
            text_snes: ImString::new("0"),
            error: ImString::new(""),
        }
    }

    pub fn run(&mut self, ui: &Ui) {
        Window::new(im_str!("Address converter"))
            .size([tweak!(300.0), tweak!(165.0)], Condition::Always)
            .resizable(false)
            .collapsible(false)
            .build(ui, || {
                self.mode_selection(ui);
                self.conversions(ui);
            });
    }

    fn mode_selection(&mut self, ui: &Ui) {
        let lorom_changed = ui.radio_button(
            im_str!("PC and LoROM"),
            &mut self.conversion_mode,
            ConversionMode::LoRom);
        let hirom_changed = ui.radio_button(
            im_str!("PC and HiROM"),
            &mut self.conversion_mode,
            ConversionMode::HiRom);
        if lorom_changed || hirom_changed {
            self.update_addresses(ConvDir::PcToSnes);
        }

        if ui.checkbox(im_str!("Include header"), &mut self.include_header) {
            let addr_pc = u32::from_str_radix(self.text_pc.to_str(), 16)
                .unwrap_or(0);
            let addr_pc = adjust_to_header(addr_pc, self.include_header);
            self.text_pc = ImString::new(format!("{:x}", addr_pc));
            self.update_addresses(ConvDir::PcToSnes);
        }
    }

    fn conversions(&mut self, ui: &Ui) {
        self.address_input(&ui, ConvDir::PcToSnes);
        self.address_input(&ui, ConvDir::SnesToPc);
        if !self.error.is_empty() {
            ui.text_colored([1.0, 0.0, 0.0, 1.0], self.error.to_str());
        }
    }

    fn address_input(&mut self, ui: &Ui, direction: ConvDir) {
        let (label, buf) = match direction {
            ConvDir::PcToSnes => (im_str!("PC"), &mut self.text_pc),
            ConvDir::SnesToPc => (im_str!("SNES"), &mut self.text_snes),
        };

        if ui.input_text(label, buf)
            .chars_hexadecimal(true)
            .chars_noblank(true)
            .auto_select_all(true)
            .build()
        {
            self.update_addresses(direction);
        }
    }

    fn update_addresses(&mut self, direction: ConvDir) {
        let (buf_src, buf_dst) = match direction {
            ConvDir::PcToSnes => (&mut self.text_pc, &mut self.text_snes),
            ConvDir::SnesToPc => (&mut self.text_snes, &mut self.text_pc),
        };

        let addr_src = u32::from_str_radix(buf_src.to_str(), 16).unwrap_or(0);
        let addr_src = if self.include_header {
            match direction {
                ConvDir::PcToSnes => adjust_to_header(addr_src, false),
                ConvDir::SnesToPc => addr_src + 0x200,
            }
        } else {
            addr_src
        };

        let addr_dst = match self.conversion_mode {
            ConversionMode::LoRom => match direction {
                ConvDir::PcToSnes => addr::pc_to_snes::lorom(addr_src),
                ConvDir::SnesToPc => addr::snes_to_pc::lorom(addr_src),
            }
            ConversionMode::HiRom => match direction {
                ConvDir::PcToSnes => addr::pc_to_snes::hirom(addr_src),
                ConvDir::SnesToPc => addr::snes_to_pc::hirom(addr_src),
            }
        };

        if let Err(msg) = addr_dst {
            self.error = ImString::new(msg);
        } else {
            let addr_dst = addr_dst.unwrap();
            *buf_dst = ImString::new(format!("{:x}", addr_dst));
            self.error.clear();
        }
    }
}

fn adjust_to_header(addr: u32, include_header: bool) -> u32 {
    if include_header {
        addr + 0x200
    } else if addr >= 0x200 {
        addr - 0x200
    } else {
        0
    }
}