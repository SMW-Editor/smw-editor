use crate::{
    OptProjectRef,
    ui::{
        UiAddressConverter,
        UiPaletteViewer,
        UiProjectCreator,
        UiRomInfo,
        UiTool,
    },
};

use imgui::{
    MenuItem,
    Ui,
    im_str,
};

use std::{
    any::TypeId,
    rc::Rc,
};

pub struct UiMainWindow {
    tools: Vec<(Box<dyn UiTool>, TypeId)>,
    running: bool,

    project_ref: Rc<OptProjectRef>,
}

impl UiMainWindow {
    pub fn new(project_ref: Rc<OptProjectRef>) -> Self {
        UiMainWindow {
            tools: Vec::new(),
            running: true,

            project_ref,
        }
    }

    pub fn run(&mut self, ui: &Ui) -> bool {
        self.main_menu_bar(ui);
        self.handle_tools(ui);

        self.running
    }

    pub fn open_tool<ToolType, Construct>(&mut self, construct_tool: Construct)
        where ToolType: 'static + UiTool,
              Construct: FnOnce() -> ToolType,
    {
        let tool_type = TypeId::of::<ToolType>();
        if !self.tools.iter().any(|(_, type_id)| type_id == &tool_type) {
            self.tools.push((Box::new(construct_tool()), tool_type));
        }
    }

    pub fn close_tool(&mut self, tool_type_id: TypeId) {
        self.tools.retain(|(_, type_id)| type_id != &tool_type_id);
    }

    fn main_menu_bar(&mut self, ui: &Ui) {
        ui.main_menu_bar(|| {
            self.menu_file(ui);
            self.menu_tools(ui);
        });
    }

    fn menu_file(&mut self, ui: &Ui) {
        ui.menu(im_str!("File"), true, || {
            let menu_item = |label| MenuItem::new(label).build(ui);
            if menu_item(im_str!("New project")) {
                let project_ref = Rc::clone(&self.project_ref);
                self.open_tool(|| UiProjectCreator::new(project_ref));
            }
            if menu_item(im_str!("Exit")) {
                self.running = false;
            }
        });
    }

    fn menu_tools(&mut self, ui: &Ui) {
        let project = Rc::clone(&self.project_ref);
        let project = project.borrow();
        let project = project.as_ref().unwrap();

        ui.menu(im_str!("Tools"), true, || {
            if MenuItem::new(im_str!("Address converter"))
                .build(ui)
            {
                self.open_tool(UiAddressConverter::new);
            }
            if MenuItem::new(im_str!("ROM info"))
                .enabled(self.project_ref.borrow().is_some())
                .build(ui)
            {
                self.open_tool(|| UiRomInfo::new(&project.rom_data.internal_header));
            }
            if MenuItem::new(im_str!("Color palettes"))
                .enabled(self.project_ref.borrow().is_some())
                .build(ui)
            {
                self.open_tool(|| UiPaletteViewer::new(&project.rom_data.level_color_palettes));
            }
        });
    }

    fn handle_tools(&mut self, ui: &Ui) {
        let mut tools_to_close = Vec::new();
        for (tool, id) in self.tools.iter_mut() {
            if !tool.run(ui) {
                tools_to_close.push(*id);
            }
        }
        tools_to_close.into_iter().for_each(|id| self.close_tool(id));
    }
}
