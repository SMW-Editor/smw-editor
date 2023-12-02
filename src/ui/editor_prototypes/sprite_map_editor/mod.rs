mod central_panel;
mod highlighting;
mod internals;
mod keyboard_shortcuts;
mod left_panel;
mod menu_bar;
mod sprite_tiles;

use std::{
    collections::HashSet,
    rc::Rc,
    sync::{Arc, Mutex},
};

use egui::*;
use glow::Context;
use smwe_emu::{emu::CheckedMem, rom::Rom, Cpu};
use smwe_math::coordinates::*;
use smwe_render::{
    gfx_buffers::GfxBuffers,
    palette_renderer::PaletteRenderer,
    tile_renderer::{Tile, TileRenderer},
};
use smwe_widgets::vram_view::{VramSelectionMode, VramView};
use sprite_tiles::SpriteTiles;

use crate::{
    ui::{editing_mode::EditingMode, tool::DockableEditorTool},
    undo::UndoableData,
};

pub struct UiSpriteMapEditor {
    gl:                Rc<Context>,
    cpu:               Cpu,
    tile_palette:      Vec<Tile>,
    vram_renderer:     Arc<Mutex<TileRenderer>>,
    sprite_renderer:   Arc<Mutex<TileRenderer>>,
    palette_renderer:  Arc<Mutex<PaletteRenderer>>,
    gfx_bufs:          GfxBuffers,
    state_needs_reset: bool,

    level_num:           u16,
    vram_selection_mode: VramSelectionMode,
    editing_mode:        EditingMode,
    always_show_grid:    bool,

    #[cfg(debug_assertions)]
    debug_selection_bounds: bool,

    tile_size_px:           f32,
    zoom:                   f32,
    pixels_per_point:       f32,
    grid_size:              OnGrid<Vec2>,
    hovering_selected_tile: bool,
    selection_bounds:       Option<OnCanvas<Rect>>,
    selection_offset:       Option<OnScreen<Vec2>>,

    selected_vram_tile:           (u32, u32),
    selected_palette:             u32,
    sprite_tiles:                 UndoableData<SpriteTiles>,
    selected_sprite_tile_indices: HashSet<usize>,
}

impl UiSpriteMapEditor {
    pub fn new(gl: Rc<Context>, rom: Arc<Rom>) -> Self {
        let (vram_renderer, tile_palette) = VramView::new_renderer(&gl);
        let sprite_renderer = TileRenderer::new(&gl);
        let palette_renderer = PaletteRenderer::new(&gl);
        let gfx_bufs = GfxBuffers::new(&gl);
        Self {
            gl,
            cpu: Cpu::new(CheckedMem::new(rom)),
            tile_palette,
            vram_renderer: Arc::new(Mutex::new(vram_renderer)),
            sprite_renderer: Arc::new(Mutex::new(sprite_renderer)),
            palette_renderer: Arc::new(Mutex::new(palette_renderer)),
            gfx_bufs,
            state_needs_reset: true,

            level_num: 0,
            vram_selection_mode: VramSelectionMode::SingleTile,
            editing_mode: EditingMode::Move(None),
            always_show_grid: false,

            #[cfg(debug_assertions)]
            debug_selection_bounds: false,

            tile_size_px: 8.,
            zoom: 3.,
            pixels_per_point: 0.,
            grid_size: OnGrid::splat(31.),
            hovering_selected_tile: false,
            selection_bounds: None,
            selection_offset: None,

            selected_vram_tile: (0, 0),
            selected_palette: 0,
            sprite_tiles: UndoableData::new(SpriteTiles(Vec::new())),
            selected_sprite_tile_indices: HashSet::new(),
        }
    }

    fn destroy(&self) {
        self.vram_renderer.lock().expect("Cannot lock mutex on VRAM renderer").destroy(&self.gl);
        self.sprite_renderer.lock().expect("Cannot lock mutex on sprite renderer").destroy(&self.gl);
        self.palette_renderer.lock().expect("Cannot lock mutex on palette renderer").destroy(&self.gl);
    }
}

impl DockableEditorTool for UiSpriteMapEditor {
    fn update(&mut self, ui: &mut Ui) {
        self.reset_state(ui.ctx());
        self.handle_input(ui);
        self.layout(ui);
    }

    fn title(&self) -> WidgetText {
        "Sprite Tile Editor".into()
    }

    fn on_closed(&mut self) {
        self.destroy();
    }
}

impl UiSpriteMapEditor {
    const MAX_ZOOM: f32 = 5.0;
    const MIN_ZOOM: f32 = 1.0;

    pub(super) fn layout(&mut self, ui: &mut Ui) {
        TopBottomPanel::top("sprite_map_editor.top_panel").show_inside(ui, |ui| {
            menu::bar(ui, |ui| self.menu_bar(ui));
        });
        SidePanel::left("sprite_map_editor.left_panel").resizable(false).show_inside(ui, |ui| self.left_panel(ui));
        CentralPanel::default().show_inside(ui, |ui| self.central_panel(ui));
    }
}
