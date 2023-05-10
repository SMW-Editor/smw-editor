use std::sync::{Arc, Mutex};

use egui::{
    vec2,
    CentralPanel,
    Color32,
    Frame,
    PaintCallback,
    Pos2,
    Rect,
    Rounding,
    Sense,
    SidePanel,
    Stroke,
    TopBottomPanel,
    Ui,
    Vec2,
    WidgetText,
};
use egui_glow::CallbackFn;
use glow::Context;
use inline_tweak::tweak;
use smwe_emu::{emu::CheckedMem, Cpu};
use smwe_render::{
    gfx_buffers::GfxBuffers,
    tile_renderer::{Tile, TileRenderer},
};
use smwe_widgets::{
    value_switcher::{ValueSwitcher, ValueSwitcherButtons},
    vram_view::{ViewedVramTiles, VramView},
};

use crate::ui::{tool::DockableEditorTool, EditorState};

pub struct UiSpriteMapEditor {
    gl:              Arc<Context>,
    sprite_tiles:    Vec<Tile>,
    tile_palette:    Vec<Tile>,
    vram_renderer:   Arc<Mutex<TileRenderer>>,
    sprite_renderer: Arc<Mutex<TileRenderer>>,
    gfx_bufs:        GfxBuffers,

    level_num:      u16,
    blue_pswitch:   bool,
    silver_pswitch: bool,
    on_off_switch:  bool,

    initialized: bool,

    selected_vram_tile: (u32, u32),
}

impl UiSpriteMapEditor {
    pub fn new(gl: Arc<Context>) -> Self {
        let (vram_renderer, tile_palette) = VramView::new_renderer(&gl);
        let sprite_renderer = TileRenderer::new(&gl);
        let gfx_bufs = GfxBuffers::new(&gl);
        Self {
            gl,
            sprite_tiles: Vec::new(),
            tile_palette,
            vram_renderer: Arc::new(Mutex::new(vram_renderer)),
            sprite_renderer: Arc::new(Mutex::new(sprite_renderer)),
            gfx_bufs,
            level_num: 0,
            blue_pswitch: false,
            silver_pswitch: false,
            on_off_switch: false,
            initialized: false,
            selected_vram_tile: (0, 0),
        }
    }

    fn destroy(&self) {
        self.vram_renderer.lock().expect("Cannot lock mutex on VRAM renderer").destroy(&self.gl);
    }
}

impl DockableEditorTool for UiSpriteMapEditor {
    fn update(&mut self, ui: &mut Ui, state: &mut EditorState) {
        if !self.initialized {
            self.update_cpu(state);
            self.update_renderers(state);
            self.initialized = true;
        }

        TopBottomPanel::top("sprite_map_editor.top_panel")
            .resizable(false)
            .show_inside(ui, |ui| self.top_panel(ui, state));
        SidePanel::left("sprite_map_editor.left_panel")
            .resizable(false)
            .show_inside(ui, |ui| self.left_panel(ui, state));
        CentralPanel::default().show_inside(ui, |ui| self.central_panel(ui, state));
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
            }
            if need_update || need_update_level {
                self.update_renderers(state);
            }
        });
    }

    fn left_panel(&mut self, ui: &mut Ui, _state: &mut EditorState) {
        let vram_renderer = Arc::clone(&self.vram_renderer);
        let gfx_bufs = self.gfx_bufs;

        // Tile selector
        ui.label("VRAM view");
        ui.add(
            VramView::new(Arc::clone(&vram_renderer), gfx_bufs)
                .viewed_tiles(ViewedVramTiles::SpritesOnly)
                .selection(&mut self.selected_vram_tile)
                .zoom(2.),
        );

        // Selected tile preview
        ui.add_space(tweak!(25.));
        ui.label("Selection preview");
        let px = ui.ctx().pixels_per_point();
        let zoom = tweak!(8.);
        let (rect, _response) = ui.allocate_exact_size(Vec2::splat(zoom * 8. / px), Sense::hover());

        let screen_size = rect.size() * px;
        let offset = vec2(-(self.selected_vram_tile.0 as f32), -32. - self.selected_vram_tile.1 as f32) * zoom;

        ui.painter().add(PaintCallback {
            rect,
            callback: Arc::new(CallbackFn::new(move |_info, painter| {
                vram_renderer.lock().expect("Cannot lock mutex on selected tile view's tile renderer").paint(
                    painter.gl(),
                    gfx_bufs,
                    screen_size,
                    offset,
                    zoom,
                );
            })),
        });
    }

    fn central_panel(&mut self, ui: &mut Ui, _state: &mut EditorState) {
        Frame::canvas(ui.style()).show(ui, |ui| {
            let sprite_renderer = Arc::clone(&self.sprite_renderer);
            let gfx_bufs = self.gfx_bufs;
            let px = ui.ctx().pixels_per_point();
            let zoom = tweak!(2.);
            let editing_area_size = tweak!(256.) * zoom;
            let (rect, response) = ui.allocate_exact_size(Vec2::splat(editing_area_size / px), Sense::click());
            let screen_size = rect.size() * px;
            let scale = tweak!(8.);
            let scale_pp = scale / px;

            ui.painter().add(PaintCallback {
                rect,
                callback: Arc::new(CallbackFn::new(move |_info, painter| {
                    sprite_renderer.lock().expect("Cannot lock mutex on sprite renderer").paint(
                        painter.gl(),
                        gfx_bufs,
                        screen_size,
                        Vec2::ZERO,
                        zoom,
                    );
                })),
            });

            // Hover/select tile
            let selection_rect = Rect::from_min_size(rect.left_top(), Vec2::splat(scale_pp * zoom));

            if let Some(hover_pos) = response.hover_pos() {
                let relative_pos = hover_pos - rect.left_top();
                let hovered_tile = (relative_pos / scale_pp / zoom).floor().clamp(vec2(0., 0.), vec2(31., 31.));
                let place_pos = hovered_tile * scale_pp * zoom;

                ui.painter().rect_filled(
                    selection_rect.translate(place_pos),
                    Rounding::same(tweak!(3.)),
                    Color32::from_white_alpha(tweak!(100)),
                );

                if response.secondary_clicked() {
                    self.add_selected_tile_at((hovered_tile * scale).to_pos2());
                }
            }
        });
    }
}

// Internals
impl UiSpriteMapEditor {
    fn update_cpu(&mut self, state: &mut EditorState) {
        let project = state.project.as_ref().unwrap().borrow();
        let mut cpu = Cpu::new(CheckedMem::new(project.rom.clone()));
        drop(project);
        smwe_emu::emu::decompress_sublevel(&mut cpu, self.level_num);
        println!("Updated CPU");
        state.cpu = Some(cpu);
    }

    fn update_renderers(&mut self, state: &mut EditorState) {
        let cpu = state.cpu.as_mut().unwrap();
        self.gfx_bufs.upload_palette(&self.gl, &cpu.mem.cgram);
        self.gfx_bufs.upload_vram(&self.gl, &cpu.mem.vram);
    }

    fn add_selected_tile_at(&mut self, pos: Pos2) {
        let tile_idx = (self.selected_vram_tile.0 + self.selected_vram_tile.1 * 16) as usize;
        let mut tile = self.tile_palette[tile_idx + (32 * 16)];
        tile.0[0] = pos.x.floor() as u32;
        tile.0[1] = pos.y.floor() as u32;
        self.sprite_tiles.push(tile);
        self.sprite_renderer
            .lock()
            .expect("Cannot lock mutex on sprite renderer")
            .set_tiles(&self.gl, self.sprite_tiles.clone());
    }
}
