use std::fmt::Write;

use egui::{TextEdit, Ui, WidgetText};
use smwe_rom::snes_utils::addr::{AddrPc, AddrSnes};

use crate::ui::{
    dev_utils::address_converter::{
        helpers::adjust_to_header,
        modes::{ConvDir, ConversionMode},
    },
    style::{EditorStyle, ErrorStyle},
    tool::DockableEditorTool,
};

#[derive(Debug)]
pub struct UiAddressConverter {
    conversion_mode: ConversionMode,
    include_header:  bool,

    text_pc:    String,
    text_snes:  String,
    text_error: String,
}

impl Default for UiAddressConverter {
    fn default() -> Self {
        UiAddressConverter {
            conversion_mode: ConversionMode::LoRom,
            include_header:  false,
            text_pc:         String::from("0"),
            text_snes:       String::from("8000"),
            text_error:      String::new(),
        }
    }
}

impl DockableEditorTool for UiAddressConverter {
    fn update(&mut self, ui: &mut Ui) {
        self.mode_selection(ui);
        self.conversions(ui);
    }

    fn title(&self) -> WidgetText {
        "Address converter".into()
    }
}

impl UiAddressConverter {
    fn mode_selection(&mut self, ui: &mut Ui) {
        let lorom_changed = ui.radio_value(&mut self.conversion_mode, ConversionMode::LoRom, "PC and LoROM").clicked();
        let hirom_changed = ui.radio_value(&mut self.conversion_mode, ConversionMode::HiRom, "PC and HiROM").clicked();
        if lorom_changed || hirom_changed {
            log::info!("Conversion mode changed to {}", self.conversion_mode);
            self.update_addresses(ConvDir::PcToSnes);
        }

        if ui.checkbox(&mut self.include_header, "Include header").clicked() {
            log::info!("Inclusion of SMC header: {}", if self.include_header { "ON" } else { "OFF" });
            let addr_pc = u32::from_str_radix(&self.text_pc, 16).unwrap_or(0);
            let addr_pc = adjust_to_header(addr_pc, self.include_header);
            self.text_pc.clear();
            write!(&mut self.text_pc, "{addr_pc:x}").unwrap();
            self.update_addresses(ConvDir::PcToSnes);
        }
    }

    fn conversions(&mut self, ui: &mut Ui) {
        self.address_input(ui, ConvDir::PcToSnes);
        self.address_input(ui, ConvDir::SnesToPc);
        if !self.text_error.is_empty() {
            ui.colored_label(ErrorStyle::get_from_egui(ui.ctx(), |style| style.text_color), &self.text_error);
        }
    }

    fn address_input(&mut self, ui: &mut Ui, direction: ConvDir) {
        ui.horizontal(|ui| {
            let (label, buf) = match direction {
                ConvDir::PcToSnes => ("PC", &mut self.text_pc),
                ConvDir::SnesToPc => ("SNES", &mut self.text_snes),
            };

            if ui.add(TextEdit::singleline(buf).desired_width(50.0)).changed() {
                buf.retain(|c| c.is_ascii_hexdigit());
                log::info!("Changed input '{}' to: {}", direction, buf);
                self.update_addresses(direction);
            }
            ui.label(label);
        });
    }

    fn update_addresses(&mut self, direction: ConvDir) {
        let (buf_src, buf_dst) = match direction {
            ConvDir::PcToSnes => (&mut self.text_pc, &mut self.text_snes),
            ConvDir::SnesToPc => (&mut self.text_snes, &mut self.text_pc),
        };

        let addr_src = {
            let addr = u32::from_str_radix(buf_src, 16).unwrap_or(0);
            if self.include_header {
                match direction {
                    ConvDir::PcToSnes => adjust_to_header(addr, false),
                    ConvDir::SnesToPc => adjust_to_header(addr, true),
                }
            } else {
                addr
            }
        };

        let addr_dst = match direction {
            ConvDir::PcToSnes => match self.conversion_mode {
                ConversionMode::LoRom => AddrSnes::try_from_lorom(AddrPc(addr_src)),
                ConversionMode::HiRom => AddrSnes::try_from_hirom(AddrPc(addr_src)),
            }
            .map(|addr| addr.0),
            ConvDir::SnesToPc => match self.conversion_mode {
                ConversionMode::LoRom => AddrPc::try_from_lorom(AddrSnes(addr_src)),
                ConversionMode::HiRom => AddrPc::try_from_hirom(AddrSnes(addr_src)),
            }
            .map(|addr| addr.0),
        };

        match addr_dst {
            Ok(addr_dst) => {
                buf_dst.clear();
                write!(buf_dst, "{addr_dst:x}").unwrap();
                self.text_error.clear();
            }
            Err(msg) => self.text_error = msg.to_string(),
        }
    }
}

mod modes {
    use std::{cmp::PartialEq, fmt};

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum ConversionMode {
        LoRom,
        HiRom,
    }

    pub enum ConvDir {
        PcToSnes,
        SnesToPc,
    }

    impl fmt::Display for ConversionMode {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(match self {
                ConversionMode::LoRom => "PC ↔ LoRom",
                ConversionMode::HiRom => "PC ↔ HiRom",
            })
        }
    }

    impl fmt::Display for ConvDir {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(match self {
                ConvDir::PcToSnes => "PC → SNES",
                ConvDir::SnesToPc => "SNES → PC",
            })
        }
    }
}

mod helpers {
    use smwe_rom::snes_utils::rom::SMC_HEADER_SIZE;

    pub fn adjust_to_header(addr: u32, include_header: bool) -> u32 {
        if include_header {
            addr + SMC_HEADER_SIZE as u32
        } else if addr >= SMC_HEADER_SIZE as u32 {
            addr - SMC_HEADER_SIZE as u32
        } else {
            0
        }
    }
}
