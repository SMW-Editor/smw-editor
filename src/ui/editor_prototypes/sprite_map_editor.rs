use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use egui::{SidePanel, TopBottomPanel, Ui, WidgetText};
use glow::Context;
use smwe_emu::{emu::CheckedMem, Cpu};
use smwe_render::{
    gfx_buffers::GfxBuffers,
    tile_renderer::{Tile, TileRenderer},
};
use smwe_widgets::{
    value_switcher::{ValueSwitcher, ValueSwitcherButtons},
    vram_view::VramView,
};

use crate::ui::{tool::DockableEditorTool, EditorState};

pub struct UiSpriteMapEditor {
    gl:              Arc<Context>,
    tiles:           Vec<Tile>,
    vram_renderer:   Arc<Mutex<TileRenderer>>,
    sprite_renderer: Arc<Mutex<TileRenderer>>,
    gfx_bufs:        GfxBuffers,

    level_num:      u16,
    blue_pswitch:   bool,
    silver_pswitch: bool,
    on_off_switch:  bool,

    initialized: bool,
    timestamp:   Instant,
}

impl UiSpriteMapEditor {
    pub fn new(gl: Arc<Context>) -> Self {
        let vram_renderer = VramView::new_renderer(&gl);
        let sprite_renderer = TileRenderer::new(&gl);
        let gfx_bufs = GfxBuffers::new(&gl);
        Self {
            gl,
            tiles: Vec::new(),
            vram_renderer: Arc::new(Mutex::new(vram_renderer)),
            sprite_renderer: Arc::new(Mutex::new(sprite_renderer)),
            gfx_bufs,
            level_num: 0,
            blue_pswitch: false,
            silver_pswitch: false,
            on_off_switch: false,
            initialized: false,
            timestamp: Instant::now(),
        }
    }

    fn destroy(&self) {
        self.vram_renderer.lock().expect("Cannot lock mutex on VRAM renderer").destroy(&self.gl);
    }
}

impl DockableEditorTool for UiSpriteMapEditor {
    fn update(&mut self, ui: &mut Ui, state: &mut EditorState) {
        if !self.initialized {
            self.init(state);
        }

        // Auto-play animations
        let ft = Duration::from_secs_f32(1. / 60.);
        let now = Instant::now();
        while now - self.timestamp > ft {
            self.timestamp += ft;
            self.update_timers(state);
            self.update_anim_frame(state);
        }
        ui.ctx().request_repaint();

        TopBottomPanel::top("sprite_map_editor.top_panel")
            .resizable(false)
            .show_inside(ui, |ui| self.top_panel(ui, state));
        SidePanel::left("sprite_map_editor.left_panel")
            .resizable(false)
            .show_inside(ui, |ui| self.left_panel(ui, state));
    }

    fn title(&self) -> WidgetText {
        "Sprite Tile Editor".into()
    }

    fn on_closed(&mut self) {
        self.destroy();
    }
}

impl UiSpriteMapEditor {
    fn top_panel(&mut self, ui: &mut Ui, state: &mut EditorState) {
        ui.horizontal(|ui| {
            let mut need_update_level = false;
            let mut need_update = false;
            let level_switcher = ValueSwitcher::new(&mut self.level_num, "Level", ValueSwitcherButtons::MinusPlus)
                .range(0..=0x1FF)
                .hexadecimal(3, false, true);

            need_update_level |= ui.add(level_switcher).changed();
            need_update |= ui.checkbox(&mut self.blue_pswitch, "Blue P-Switch").changed();
            need_update |= ui.checkbox(&mut self.silver_pswitch, "Silver P-Switch").changed();
            need_update |= ui.checkbox(&mut self.on_off_switch, "ON/OFF Switch").changed();

            if need_update_level {
                self.update_cpu(state);
                self.update_cpu_sprite_id(state);
            }
            if need_update || need_update_level {
                self.update_renderers(state);
            }
        });
    }

    fn left_panel(&mut self, ui: &mut Ui, _state: &mut EditorState) {
        ui.add(VramView::new(Arc::clone(&self.vram_renderer), self.gfx_bufs));
    }
}

// Internals
impl UiSpriteMapEditor {
    fn init(&mut self, state: &mut EditorState) {
        self.update_cpu(state);
        self.update_cpu_sprite(state);
        self.update_renderers(state);
        self.initialized = true;
    }

    fn update_cpu(&mut self, state: &mut EditorState) {
        let project = state.project.as_ref().unwrap().borrow();
        let mut cpu = Cpu::new(CheckedMem::new(project.rom.clone()));
        drop(project);
        smwe_emu::emu::decompress_sublevel(&mut cpu, self.level_num);
        println!("Updated CPU");
        state.cpu = Some(cpu);
    }

    fn update_timers(&mut self, state: &mut EditorState) {
        let cpu = state.cpu.as_mut().unwrap(); // should be set already
        let m = cpu.mem.load_u8(0x13);
        cpu.mem.store_u8(0x13, m.wrapping_add(1));
        let m = cpu.mem.load_u8(0x14);
        cpu.mem.store_u8(0x14, m.wrapping_add(1));
    }

    fn update_cpu_sprite(&mut self, state: &mut EditorState) {
        let cpu = state.cpu.as_mut().unwrap(); // should be set already
        cpu.mem.wram[0x300..0x400].fill(0xE0);
        smwe_emu::emu::exec_sprites(cpu);
    }

    fn update_cpu_sprite_id(&mut self, state: &mut EditorState) {
        let cpu = state.cpu.as_mut().unwrap(); // should be set already
        let mut cpu = cpu.clone();
        cpu.mem.wram[0x300..0x400].fill(0xE0);
        smwe_emu::emu::exec_sprite_id(&mut cpu, 0);
    }

    fn update_anim_frame(&mut self, state: &mut EditorState) {
        let cpu = state.cpu.as_mut().unwrap(); // should be set already
        cpu.mem.store_u8(0x14AD, self.blue_pswitch as u8);
        cpu.mem.store_u8(0x14AE, self.silver_pswitch as u8);
        cpu.mem.store_u8(0x14AF, self.on_off_switch as u8);
        smwe_emu::emu::fetch_anim_frame(cpu);
        self.gfx_bufs.upload_vram(&self.gl, &cpu.mem.vram);
    }

    fn update_renderers(&mut self, state: &mut EditorState) {
        self.update_anim_frame(state);

        let cpu = state.cpu.as_mut().unwrap();
        self.gfx_bufs.upload_palette(&self.gl, &cpu.mem.cgram);
        self.gfx_bufs.upload_vram(&self.gl, &cpu.mem.vram);
    }
}
