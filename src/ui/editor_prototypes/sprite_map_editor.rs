use std::sync::{Arc, Mutex};

use egui::*;
use egui_glow::CallbackFn;
use egui_phosphor as icons;
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

use crate::ui::{editing_mode::EditingMode, tool::DockableEditorTool, EditorState};

pub struct UiSpriteMapEditor {
    gl:              Arc<Context>,
    sprite_tiles:    Vec<Tile>,
    tile_palette:    Vec<Tile>,
    vram_renderer:   Arc<Mutex<TileRenderer>>,
    sprite_renderer: Arc<Mutex<TileRenderer>>,
    gfx_bufs:        GfxBuffers,

    level_num:    u16,
    editing_mode: EditingMode,

    initialized: bool,
    scale:       f32,
    zoom:        f32,

    selected_vram_tile: (u32, u32),
    last_inserted_tile: Pos2,
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
            editing_mode: EditingMode::Insert,
            initialized: false,
            scale: 8.,
            zoom: 2.,
            selected_vram_tile: (0, 0),
            last_inserted_tile: pos2(-1., -1.),
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
            self.level_selector(ui, state);

            ui.add_space(tweak!(62.) * ui.ctx().pixels_per_point());
            ui.separator();

            self.editing_mode_selector(ui, state);
        });
    }

    fn level_selector(&mut self, ui: &mut Ui, state: &mut EditorState) {
        let level_switcher = ValueSwitcher::new(&mut self.level_num, "Level", ValueSwitcherButtons::MinusPlus)
            .range(0..=0x1FF)
            .hexadecimal(3, false, true);

        if ui.add(level_switcher).changed() {
            self.update_cpu(state);
            self.update_renderers(state);
        }
    }

    fn editing_mode_selector(&mut self, ui: &mut Ui, _state: &mut EditorState) {
        let editing_mode_data = [
            (icons::CURSOR, "Insert mode", "Insert tile on double-click, select on drag.", EditingMode::Insert),
            (icons::RECTANGLE, "Select mode", "Left-click and drag to select tiles.", EditingMode::Select),
            (icons::PENCIL, "Draw mode", "Insert tiles while left mouse button is pressed.", EditingMode::Draw),
            (icons::ERASER, "Erase mode", "Delete tiles while left mouse button is pressed.", EditingMode::Erase),
            (icons::EYEDROPPER, "Probe mode", "Pick a tile from the canvas on left-click.", EditingMode::Probe),
        ];

        for (icon, mode_name, mode_desc, mode) in editing_mode_data.into_iter() {
            let button = if self.editing_mode == mode {
                Button::new(icon).fill(Color32::from_rgb(tweak!(200), tweak!(30), tweak!(70)))
            } else {
                Button::new(icon)
            };
            if ui
                .add(button)
                .on_hover_ui_at_pointer(|ui| {
                    ui.strong(mode_name);
                    ui.label(mode_desc);
                })
                .clicked()
            {
                self.editing_mode = mode;
            }
        }
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
            let editing_area_size = tweak!(256.) * self.zoom;
            let (rect, response) = ui.allocate_exact_size(Vec2::splat(editing_area_size / px), Sense::click_and_drag());
            let screen_size = rect.size() * px;
            let scale_pp = self.scale / px;
            let zoom = self.zoom;

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
            let selection_rect = Rect::from_min_size(rect.left_top(), Vec2::splat(scale_pp * self.zoom));

            if let Some(hover_pos) = response.hover_pos() {
                let relative_pos = hover_pos - rect.left_top();
                let hovered_tile = (relative_pos / scale_pp / self.zoom).floor();
                let hovered_tile = hovered_tile.clamp(vec2(0., 0.), vec2(31., 31.));

                ui.painter().rect_filled(
                    selection_rect.translate(hovered_tile * scale_pp * self.zoom),
                    Rounding::same(tweak!(3.)),
                    Color32::from_white_alpha(tweak!(100)),
                );

                if self.editing_mode.inserted(&response) {
                    let place_pos = (hovered_tile * self.scale).to_pos2();
                    if self.last_inserted_tile != place_pos {
                        self.add_selected_tile_at(place_pos);
                        self.last_inserted_tile = place_pos;
                    }
                }

                if self.editing_mode.selected(&response) {
                    // todo
                }

                if self.editing_mode.erased(&response) {
                    let pixel_pos = relative_pos / self.zoom * px;
                    self.delete_tiles_at(pixel_pos.to_pos2());
                }

                if self.editing_mode.probed(&response) {
                    // todo
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

    fn upload_tiles(&self) {
        self.sprite_renderer
            .lock()
            .expect("Cannot lock mutex on sprite renderer")
            .set_tiles(&self.gl, self.sprite_tiles.clone());
    }

    fn add_selected_tile_at(&mut self, pos: Pos2) {
        let tile_idx = (self.selected_vram_tile.0 + self.selected_vram_tile.1 * 16) as usize;
        let mut tile = self.tile_palette[tile_idx + (32 * 16)];
        tile.0[0] = pos.x.floor() as u32;
        tile.0[1] = pos.y.floor() as u32;
        self.sprite_tiles.push(tile);
        self.upload_tiles();
    }

    fn delete_tiles_at(&mut self, pos: Pos2) {
        self.sprite_tiles.retain(|tile| {
            let min = pos2(tile.0[0] as f32, tile.0[1] as f32);
            let size = Vec2::splat(self.scale);
            !Rect::from_min_size(min, size).contains(pos)
        });
        self.upload_tiles();
    }
}
