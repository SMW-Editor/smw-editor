mod central_panel;
mod left_panel;
mod level_renderer;
mod object_layer;
mod properties;

use std::sync::{Arc, Mutex};

use egui::{CentralPanel, SidePanel, Ui, WidgetText, *};
use smwe_emu::{emu::CheckedMem, rom::Rom, Cpu};

use self::{level_renderer::LevelRenderer, object_layer::EditableObjectLayer, properties::LevelProperties};
use crate::ui::tool::DockableEditorTool;

pub struct UiLevelEditor {
    gl:             Arc<glow::Context>,
    cpu:            Cpu,
    level_renderer: Arc<Mutex<LevelRenderer>>,

    level_num:      u16,
    blue_pswitch:   bool,
    silver_pswitch: bool,
    on_off_switch:  bool,
    run_sprites:    bool,
    palette_line:   u8,
    sprite_id:      u8,
    timestamp:      std::time::Instant,

    offset:           Vec2,
    zoom:             f32,
    tile_size_px:     f32,
    pixels_per_point: f32,
    always_show_grid: bool,

    level_properties: LevelProperties,
    layer1:           EditableObjectLayer,
}

impl UiLevelEditor {
    pub fn new(gl: Arc<glow::Context>, rom: Arc<Rom>) -> Self {
        let level_renderer = Arc::new(Mutex::new(LevelRenderer::new(&gl)));
        let mut editor = Self {
            gl,
            cpu: Cpu::new(CheckedMem::new(rom)),
            level_renderer,
            level_num: 0x105,
            blue_pswitch: false,
            silver_pswitch: false,
            on_off_switch: false,
            run_sprites: false,
            palette_line: 0,
            sprite_id: 0,
            timestamp: std::time::Instant::now(),
            offset: Vec2::ZERO,
            zoom: 2.,
            tile_size_px: 16.,
            pixels_per_point: 1.,
            always_show_grid: false,
            level_properties: LevelProperties::default(),
            layer1: EditableObjectLayer::default(),
        };
        editor.init_cpu();
        editor.update_cpu_sprite();
        editor.update_renderer();
        editor
    }
}

// UI
impl DockableEditorTool for UiLevelEditor {
    fn update(&mut self, ui: &mut Ui) {
        self.pixels_per_point = ui.ctx().pixels_per_point();

        SidePanel::left("level_editor.left_panel").resizable(false).show_inside(ui, |ui| self.left_panel(ui));

        CentralPanel::default()
            .frame(Frame::none().inner_margin(0.).fill(Color32::GRAY))
            .show_inside(ui, |ui| self.central_panel(ui));

        // Auto-play animations
        let ft = std::time::Duration::from_secs_f32(1. / 60.);
        let now = std::time::Instant::now();
        while now - self.timestamp > ft {
            self.timestamp += ft;
            self.update_timers();
            self.update_anim_frame();
            if self.run_sprites {
                self.update_cpu_sprite();
                //self.level_renderer.lock().unwrap().upload_level(&self.gl, &mut self.cpu);
            }
        }
        ui.ctx().request_repaint();
    }

    fn title(&self) -> WidgetText {
        "Level Editor".into()
    }

    fn on_closed(&mut self) {
        self.level_renderer.lock().unwrap().destroy(&self.gl);
    }
}

// Internals
impl UiLevelEditor {
    fn init_cpu(&mut self) {
        smwe_emu::emu::decompress_sublevel(&mut self.cpu, self.level_num);
        println!("Updated CPU");
        self.level_renderer.lock().unwrap().upload_level(&self.gl, &mut self.cpu);
        self.update_level_properties();
        self.update_layer1();
    }

    fn update_cpu(&mut self) {
        smwe_emu::emu::decompress_extram(&mut self.cpu, self.level_num);
        println!("Updated CPU");
        self.level_renderer.lock().unwrap().upload_level(&self.gl, &mut self.cpu);
    }

    fn update_level_properties(&mut self) {
        self.level_properties = LevelProperties::parse_from_ram(&mut self.cpu);
    }

    fn update_layer1(&mut self) {
        self.layer1 = EditableObjectLayer::parse_from_ram(&mut self.cpu, self.level_properties.is_vertical)
            .expect("Failed to parse objects from ExtRAM");
    }

    fn update_timers(&mut self) {
        let m = self.cpu.mem.load_u8(0x13);
        self.cpu.mem.store_u8(0x13, m.wrapping_add(1));
        let m = self.cpu.mem.load_u8(0x14);
        self.cpu.mem.store_u8(0x14, m.wrapping_add(1));
    }

    fn update_cpu_sprite(&mut self) {
        self.cpu.mem.wram[0x300..0x400].fill(0xE0);
        smwe_emu::emu::exec_sprites(&mut self.cpu);
        self.level_renderer.lock().unwrap().upload_sprites(&self.gl, &mut self.cpu);
    }

    fn update_cpu_sprite_id(&mut self) {
        let mut cpu = self.cpu.clone();
        cpu.mem.wram[0x300..0x400].fill(0xE0);
        smwe_emu::emu::exec_sprite_id(&mut cpu, self.sprite_id);
        self.level_renderer.lock().unwrap().upload_sprites(&self.gl, &mut cpu);
    }

    fn update_anim_frame(&mut self) {
        self.cpu.mem.store_u8(0x14AD, self.blue_pswitch as u8);
        self.cpu.mem.store_u8(0x14AE, self.silver_pswitch as u8);
        self.cpu.mem.store_u8(0x14AF, self.on_off_switch as u8);
        smwe_emu::emu::fetch_anim_frame(&mut self.cpu);
        self.level_renderer
            .lock()
            .expect("Cannot lock mutex on level_renderer")
            .upload_gfx(&self.gl, &self.cpu.mem.vram);
    }

    fn update_renderer(&mut self) {
        self.update_anim_frame();

        let level_renderer = self.level_renderer.lock().expect("Cannot lock mutex on level_renderer");
        level_renderer.upload_palette(&self.gl, &self.cpu.mem.cgram);
        level_renderer.upload_gfx(&self.gl, &self.cpu.mem.vram);
    }
}
