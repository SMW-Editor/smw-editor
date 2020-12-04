use crate::ui::{
    color,
    tool::UiTool,
};

use self::helpers::*;
use self::modes::*;

use imgui::{
    ImString,
    Window,
    Ui,
    im_str,
};

use nsmwe_rom::addr;

pub struct UiAddressConverter {
    conversion_mode: ConversionMode,
    include_header: bool,

    text_pc: ImString,
    text_snes: ImString,
    text_error: ImString,
}

impl UiTool for UiAddressConverter {
    fn run(&mut self, ui: &Ui) -> bool {
        let mut running = true;

        Window::new(im_str!("Address converter"))
            .always_auto_resize(true)
            .resizable(false)
            .collapsible(false)
            .scroll_bar(false)
            .opened(&mut running)
            .build(ui, || {
                self.mode_selection(ui);
                self.conversions(ui);
            });

        running
    }
}

impl Default for UiAddressConverter {
    fn default() -> Self {
        UiAddressConverter::new()
    }
}

impl UiAddressConverter {
    pub fn new() -> Self {
        UiAddressConverter {
            conversion_mode: ConversionMode::LoRom,
            include_header: false,
            text_pc: ImString::new("0"),
            text_snes: ImString::new("8000"),
            text_error: ImString::new(""),
        }
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
            let addr_pc = usize::from_str_radix(self.text_pc.to_str(), 16)
                .unwrap_or(0);
            let addr_pc = adjust_to_header(addr_pc, self.include_header);
            self.text_pc = ImString::new(format!("{:x}", addr_pc));
            self.update_addresses(ConvDir::PcToSnes);
        }
    }

    fn conversions(&mut self, ui: &Ui) {
        self.address_input(&ui, ConvDir::PcToSnes);
        self.address_input(&ui, ConvDir::SnesToPc);
        if !self.text_error.is_empty() {
            ui.text_colored(color::TEXT_ERROR, self.text_error.to_str());
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

        let addr_src = {
            let addr = usize::from_str_radix(buf_src.to_str(), 16)
                .unwrap_or(0);
            if self.include_header {
                match direction {
                    ConvDir::PcToSnes => adjust_to_header(addr, false),
                    ConvDir::SnesToPc => adjust_to_header(addr, true),
                }
            } else {
                addr
            }
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
            self.text_error = ImString::new(msg);
        } else {
            let addr_dst = addr_dst.unwrap();
            *buf_dst = ImString::new(format!("{:x}", addr_dst));
            self.text_error.clear();
        }
    }
}

mod modes {
    use std::cmp::PartialEq;

    #[derive(Clone, Copy, PartialEq)]
    pub enum ConversionMode {
        LoRom,
        HiRom,
    }

    pub enum ConvDir {
        PcToSnes,
        SnesToPc,
    }
}

mod helpers {
    use nsmwe_rom::SMC_HEADER_SIZE;

    pub fn adjust_to_header(addr: usize, include_header: bool) -> usize {
        if include_header {
            addr + SMC_HEADER_SIZE
        } else if addr >= SMC_HEADER_SIZE {
            addr - SMC_HEADER_SIZE
        } else {
            0
        }
    }
}
