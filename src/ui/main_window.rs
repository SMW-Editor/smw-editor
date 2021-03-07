use crate::{
    frame_context::FrameContext,
    ui::{
        UiAddressConverter,
        UiGfxViewer,
        UiPaletteViewer,
        UiProjectCreator,
        UiRomInfo,
        UiTool,
    },
};

use imgui::{
    MenuItem,
    im_str,
};

pub struct UiMainWindow {
    last_open_tool_id: i32,
    tools: Vec<Box<dyn UiTool>>,
    running: bool,
}

impl UiMainWindow {
    pub fn new() -> Self {
        UiMainWindow {
            last_open_tool_id: 0,
            tools: Vec::new(),
            running: true,
        }
    }

    pub fn tick(&mut self, ctx: &mut FrameContext) -> bool {
        self.main_menu_bar(ctx);
        self.handle_tools(ctx);

        self.running
    }

    pub fn open_tool<ToolType, Construct>(&mut self, construct_tool: Construct)
        where ToolType: 'static + UiTool,
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
        ctx.ui.menu(im_str!("File"), true, || {
            let menu_item = |label| MenuItem::new(label).build(ctx.ui);
            if menu_item(im_str!("New project")) {
                self.open_tool(UiProjectCreator::new);
            }
            if menu_item(im_str!("Exit")) {
                self.running = false;
            }
        });
    }

    fn menu_tools(&mut self, ctx: &mut FrameContext) {
        let FrameContext {
            ui,
            project_ref,
            ..
        } = ctx;
        let project = project_ref.as_ref().map(|p| p.borrow_mut());

        ui.menu(im_str!("Tools"), true, || {
            if MenuItem::new(im_str!("Address converter"))
                .build(ui)
            {
                self.open_tool(UiAddressConverter::new);
            }
            if MenuItem::new(im_str!("ROM info"))
                .enabled(project.is_some())
                .build(ui)
            {
                let internal_header = &project.as_ref().unwrap().rom_data.internal_header;
                self.open_tool(|id| UiRomInfo::new(id, internal_header));
            }
            if MenuItem::new(im_str!("Color palettes"))
                .enabled(project.is_some())
                .build(ui)
            {
                self.open_tool(UiPaletteViewer::new);
            }
            if MenuItem::new(im_str!("GFX"))
                .enabled(project.is_some())
                .build(ui)
            {
                self.open_tool(UiGfxViewer::new);
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
        tools_to_close
            .into_iter()
            .rev()
            .for_each(|i| { self.tools.swap_remove(i); });
    }
}
