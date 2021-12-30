use imgui::MenuItem;

use crate::{
    frame_context::FrameContext,
    ui::{
        dev_utils::disassembler::UiDisassembler,
        UiAddressConverter,
        UiGfxViewer,
        UiPaletteViewer,
        UiProjectCreator,
        UiRomInfo,
        UiTool,
    },
};

pub struct UiMainWindow {
    last_open_tool_id: i32,
    tools:             Vec<Box<dyn UiTool>>,
    running:           bool,
}

impl UiMainWindow {
    pub fn new() -> Self {
        UiMainWindow { last_open_tool_id: 0, tools: Vec::new(), running: true }
    }

    pub fn tick(&mut self, ctx: &mut FrameContext) -> bool {
        self.main_menu_bar(ctx);
        self.handle_tools(ctx);

        self.running
    }

    pub fn open_tool<ToolType, Construct>(&mut self, construct_tool: Construct)
    where
        ToolType: 'static + UiTool,
        Construct: FnOnce(i32) -> ToolType,
    {
        if self.last_open_tool_id < i32::MAX {
            self.tools.push(Box::new(construct_tool(self.last_open_tool_id)));
            self.last_open_tool_id += 1;
        }
    }

    fn main_menu_bar(&mut self, ctx: &mut FrameContext) {
        ctx.ui.main_menu_bar(|| {
            self.menu_file(ctx);
            self.menu_tools(ctx);
        });
    }

    fn menu_file(&mut self, ctx: &mut FrameContext) {
        let FrameContext { ui, project_ref, .. } = ctx;
        let project = project_ref.as_ref().map(|p| p.borrow_mut());
        ui.menu("File", || {
            let menu_item = |label| MenuItem::new(label).build(ui);
            if menu_item("New project") {
                self.open_tool(UiProjectCreator::new);
            }
            if MenuItem::new("Save ROM dump").enabled(project.is_some()).build(ui) {
                use nfd2::Response;
                if let Response::Okay(path) =
                    nfd2::open_save_dialog(Some("txt"), None) //
                        .unwrap_or_else(|e| panic!("Cannot open file selector: {}", e))
                {
                    use std::fmt::Write;
                    let mut dump = String::with_capacity(4096);
                    write!(&mut dump, "{:?}", project.unwrap().rom_data.disassembly).unwrap();
                    std::fs::write(path, dump).unwrap();
                }
            }
            if menu_item("Exit") {
                self.running = false;
            }
        });
    }

    fn menu_tools(&mut self, ctx: &mut FrameContext) {
        let FrameContext { ui, project_ref, .. } = ctx;
        let project = project_ref.as_ref().map(|p| p.borrow_mut());

        ui.menu("Tools", || {
            if MenuItem::new("Address converter") //
                .build(ui)
            {
                self.open_tool(UiAddressConverter::new);
            }
            if MenuItem::new("ROM info") //
                .enabled(project.is_some())
                .build(ui)
            {
                let internal_header = &project.as_ref().unwrap().rom_data.internal_header;
                self.open_tool(|id| UiRomInfo::new(id, internal_header));
            }
            if MenuItem::new("Color palettes") //
                .enabled(project.is_some())
                .build(ui)
            {
                self.open_tool(UiPaletteViewer::new);
            }
            if MenuItem::new("GFX") //
                .enabled(project.is_some())
                .build(ui)
            {
                self.open_tool(UiGfxViewer::new);
            }
            if MenuItem::new("Disassembler") //
                .enabled(project.is_some())
                .build(ui)
            {
                self.open_tool(UiDisassembler::new);
            }
        });
    }

    fn handle_tools(&mut self, ctx: &mut FrameContext) {
        let mut tools_to_close = Vec::with_capacity(1);
        for (i, tool) in self.tools.iter_mut().enumerate() {
            if !tool.tick(ctx) {
                tools_to_close.push(i);
            }
        }
        tools_to_close.into_iter().rev().for_each(|i| {
            self.tools.swap_remove(i);
        });
    }
}
