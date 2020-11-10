use crate::{
    tool::UiTool,
    utils::{
        address_converter::UiAddressConverter,
        rom_info::UiRomInfo,
    },
};

use imgui::{
    MenuItem,
    Ui,
    im_str,
};
use std::any::TypeId;

pub struct UiMainWindow {
    tools: Vec<(Box<dyn UiTool>, TypeId)>,
}

impl UiTool for UiMainWindow {
    fn run(&mut self, ui: &Ui) -> bool {
        self.main_menu_bar(ui);

        let mut tools_to_close = Vec::new();
        for tool in self.tools.iter_mut() {
            if !tool.0.run(ui) {
                tools_to_close.push(tool.1);
            }
        }
        for id in tools_to_close {
            self.close_tool(id);
        }

        true
    }
}

impl UiMainWindow {
    pub fn new() -> Self {
        UiMainWindow {
            tools: Vec::new(),
        }
    }

    pub fn open_tool<ToolType, Construct>(&mut self, construct: Construct)
        where ToolType: 'static + UiTool,
              Construct: FnOnce() -> ToolType,
    {
        let tool_type = TypeId::of::<ToolType>();
        if self.tools.iter().find(|(_, type_id)| type_id == &tool_type).is_none() {
            self.tools.push((Box::new(construct()), tool_type));
        }
    }

    pub fn close_tool(&mut self, tool_type_id: TypeId) {
        self.tools.retain(|(_, type_id)| type_id != &tool_type_id);
    }

    fn main_menu_bar(&mut self, ui: &Ui) {
        ui.main_menu_bar(|| {
            self.menu_tools(ui);
        });
    }

    fn menu_tools(&mut self, ui: &Ui) {
        ui.menu(im_str!("Tools"), true, || {
            if MenuItem::new(im_str!("Address converter")).build(ui) {
                self.open_tool(UiAddressConverter::new);
            }
            if MenuItem::new(im_str!("ROM info")).build(ui) {
                self.open_tool(UiRomInfo::new);
            }
        });
    }
}
