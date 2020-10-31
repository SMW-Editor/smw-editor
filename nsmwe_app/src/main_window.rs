use crate::address_converter::UiAddressConverter;

use imgui::Ui;

pub struct UiMainWindow {
    address_converter: UiAddressConverter,
}

impl UiMainWindow {
    pub fn new() -> Self {
        UiMainWindow {
            address_converter: UiAddressConverter::new(),
        }
    }

    pub fn run(&mut self, ui: &Ui) {
        self.address_converter.run(ui);
    }
}
