mod components;
mod editing;
mod highlighting;
mod input;
mod internals;
mod layout;

use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use egui::*;
use glow::Context;
use smwe_emu::{emu::CheckedMem, rom::Rom, Cpu};
use smwe_math::coordinates::{OnCanvas, OnGrid, OnScreen};
use smwe_render::{
    gfx_buffers::GfxBuffers,
    palette_renderer::PaletteRenderer,
    tile_renderer::{Tile, TileRenderer},
};
use smwe_widgets::vram_view::{VramSelectionMode, VramView};

use crate::ui::editing_mode::EditingMode;

pub struct UiSpriteMapEditor {
    gl:               Arc<Context>,
    cpu:              Cpu,
    tile_palette:     Vec<Tile>,
    vram_renderer:    Arc<Mutex<TileRenderer>>,
    sprite_renderer:  Arc<Mutex<TileRenderer>>,
    palette_renderer: Arc<Mutex<PaletteRenderer>>,
    gfx_bufs:         GfxBuffers,

    level_num:           u16,
    vram_selection_mode: VramSelectionMode,
    editing_mode:        EditingMode,
    always_show_grid:    bool,

    debug_selection_bounds: bool,

    initialized:            bool,
    tile_size_px:           f32,
    zoom:                   f32,
    pixels_per_point:       f32,
    grid_size:              OnGrid<Vec2>,
    hovering_selected_tile: bool,
    selection_bounds:       Option<OnCanvas<Rect>>,
    selection_offset:       Option<OnScreen<Vec2>>,

    selected_vram_tile:           (u32, u32),
    selected_palette:             u32,
    sprite_tiles:                 Vec<Tile>,
    last_inserted_tile:           OnCanvas<Pos2>,
    selected_sprite_tile_indices: HashSet<usize>,
}

impl UiSpriteMapEditor {
    pub fn new(gl: Arc<Context>, rom: Arc<Rom>) -> Self {
        let (vram_renderer, tile_palette) = VramView::new_renderer(&gl);
        let sprite_renderer = TileRenderer::new(&gl);
        let palette_renderer = PaletteRenderer::new(&gl);
        let gfx_bufs = GfxBuffers::new(&gl);
        Self {
            gl,
            cpu: Cpu::new(CheckedMem::new(rom)),
            sprite_tiles: Vec::new(),
            vram_renderer: Arc::new(Mutex::new(vram_renderer)),
            sprite_renderer: Arc::new(Mutex::new(sprite_renderer)),
            palette_renderer: Arc::new(Mutex::new(palette_renderer)),
            gfx_bufs,

            level_num: 0,
            vram_selection_mode: VramSelectionMode::SingleTile,
            editing_mode: EditingMode::Move(None),
            always_show_grid: false,

            debug_selection_bounds: false,

            initialized: false,
            tile_size_px: 8.,
            zoom: 3.,
            pixels_per_point: 0.,
            grid_size: OnGrid::<Vec2>::splat(31.),
            hovering_selected_tile: false,
            selection_bounds: None,
            selection_offset: None,

            selected_vram_tile: (0, 0),
            selected_palette: 0,
            tile_palette,
            last_inserted_tile: OnCanvas::<Pos2>::new(-1., -1.),
            selected_sprite_tile_indices: HashSet::new(),
        }
    }

    fn destroy(&self) {
        self.vram_renderer.lock().expect("Cannot lock mutex on VRAM renderer").destroy(&self.gl);
        self.sprite_renderer.lock().expect("Cannot lock mutex on sprite renderer").destroy(&self.gl);
        self.palette_renderer.lock().expect("Cannot lock mutex on palette renderer").destroy(&self.gl);
    }
}
