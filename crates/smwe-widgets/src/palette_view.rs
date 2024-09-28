use std::sync::{Arc, Mutex};

use egui::*;
use egui_glow::{glow::Buffer, CallbackFn};
use inline_tweak::tweak;
use smwe_render::palette_renderer::{PaletteRenderer, PaletteUniforms};

#[derive(Debug)]
pub struct PaletteView<'s> {
    renderer:        Arc<Mutex<PaletteRenderer>>,
    palette_buf:     Buffer,
    viewed_palettes: ViewedPalettes,
    selection:       Option<SelectionType<'s>>,
    size:            Vec2,
}

#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum ViewedPalettes {
    All            = 0,
    BackgroundOnly = 1,
    SpritesOnly    = 2,
}

#[derive(Debug)]
pub enum SelectionType<'s> {
    Row(&'s mut u32),
    Cell(&'s mut (u32, u32)),
}

impl<'s> PaletteView<'s> {
    pub fn new(renderer: Arc<Mutex<PaletteRenderer>>, palette_buf: Buffer, size: Vec2) -> Self {
        Self { renderer, palette_buf, viewed_palettes: ViewedPalettes::All, selection: None, size }
    }

    pub fn viewed_rows(mut self, viewed_rows: ViewedPalettes) -> Self {
        self.viewed_palettes = viewed_rows;
        self
    }

    pub fn selection(mut self, selection: SelectionType<'s>) -> Self {
        self.selection = Some(selection);
        self
    }
}

impl Widget for PaletteView<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let palette_renderer = Arc::clone(&self.renderer);
        let uniforms =
            PaletteUniforms { palette_buf: self.palette_buf, viewed_palettes: self.viewed_palettes as u32 };
        let (view_rect, mut response) = ui.allocate_exact_size(self.size, Sense::click());

        ui.painter().add(PaintCallback {
            rect:     view_rect,
            callback: Arc::new(CallbackFn::new(move |_info, painter| {
                palette_renderer.lock().expect("Cannot lock mutex on palette renderer").paint(painter.gl(), &uniforms);
            })),
        });

        let cell_width = self.size.x / 16.;
        let row_count = match self.viewed_palettes {
            ViewedPalettes::All => 16,
            _ => 8,
        };
        match self.selection {
            Some(SelectionType::Cell((_x, _y))) => {
                unimplemented!()
            }
            Some(SelectionType::Row(row)) => {
                let row_size = vec2(self.size.x, self.size.x / 16.);

                if let Some(hover_pos) = response.hover_pos() {
                    let relative_pos = hover_pos - view_rect.left_top();
                    let hovered_row = (relative_pos.y / cell_width).floor().clamp(0., (row_count - 1) as f32);
                    let highlight_rect =
                        Rect::from_min_size(view_rect.min + vec2(0., hovered_row * cell_width), row_size);
                    ui.painter().rect_filled(highlight_rect, Rounding::ZERO, Color32::from_white_alpha(tweak!(100)));

                    if response.clicked_by(PointerButton::Primary) && *row != hovered_row as _ {
                        response.mark_changed();
                        *row = hovered_row as _;
                    }
                }

                let selection_rect = Rect::from_min_size(view_rect.min + vec2(0., *row as f32 * cell_width), row_size);
                ui.painter().rect_stroke(
                    selection_rect,
                    Rounding::same(2.),
                    Stroke::new(2., Color32::from_rgba_premultiplied(200, 100, 30, 100)),
                );
            }
            _ => {}
        }

        response
    }
}
