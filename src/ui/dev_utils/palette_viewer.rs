use egui::*;
use inline_tweak::tweak;
use itertools::Itertools;
use num_enum::TryFromPrimitive;
use smwe_rom::graphics::palette::{ColorPalette, OverworldState};
use smwe_widgets::flipbook::{AnimationState, Flipbook};

use crate::ui::{frame_context::EditorToolTabViewer, tool::DockableEditorTool};

#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, TryFromPrimitive)]
pub enum PaletteContext {
    Level     = 0,
    Overworld = 1,
}

enum PaletteImage {
    Static(TextureHandle),
    Animated(AnimationState),
}

pub struct UiPaletteViewer {
    palette_context:   PaletteContext,
    palette_image:     Option<PaletteImage>,
    // Level viewer
    level_num:         i32,
    // Overworld viewer
    submap_num:        i32,
    special_completed: bool,
}

impl Default for UiPaletteViewer {
    fn default() -> Self {
        UiPaletteViewer {
            palette_context:   PaletteContext::Level,
            palette_image:     None,
            level_num:         0,
            submap_num:        0,
            special_completed: false,
        }
    }
}

impl DockableEditorTool for UiPaletteViewer {
    fn update(&mut self, ui: &mut Ui, ctx: &mut EditorToolTabViewer) {
        if self.palette_image.is_none() {
            self.update_palette_image(ui, ctx);
        }

        TopBottomPanel::top("palette_selectors_panel").show_inside(ui, |ui| {
            self.context_selector(ui, ctx);
            match self.palette_context {
                PaletteContext::Level => self.selectors_level(ui, ctx),
                PaletteContext::Overworld => self.selectors_overworld(ui, ctx),
            }
        });
        ui.centered_and_justified(|ui| self.display_palette(ui));
    }

    fn title(&self) -> WidgetText {
        "Color palettes".into()
    }
}

impl UiPaletteViewer {
    fn context_selector(&mut self, ui: &mut Ui, ctx: &mut EditorToolTabViewer) {
        let mut context_changed = false;
        let mut context_raw = self.palette_context as usize;
        let context_names = ["Level", "Overworld"];

        ComboBox::from_label("Context").selected_text(context_names[context_raw]).show_ui(ui, |ui| {
            for (context_idx, &context_name) in context_names.iter().enumerate() {
                if ui.selectable_value(&mut context_raw, context_idx, context_name).changed() {
                    context_changed = true;
                }
            }
        });

        self.palette_context = PaletteContext::try_from(context_raw).unwrap_or(PaletteContext::Level);

        if context_changed {
            self.update_palette_image(ui, ctx);
        }
    }

    fn selectors_level(&mut self, ui: &mut Ui, ctx: &mut EditorToolTabViewer) {
        let level_count = {
            let project = ctx.project_ref.as_ref().unwrap().borrow();
            project.rom_data.levels.len() as i32
        };

        ui.horizontal(|ui| {
            let drag_level_num =
                DragValue::new(&mut self.level_num).clamp_range(0..=level_count - 1).hexadecimal(3, false, true);
            if ui.add(drag_level_num).changed() {
                log::info!("Showing color palette for level {:X}", self.level_num);
                self.update_palette_image(ui, ctx);
            }
            ui.label("Level number");
        });
    }

    fn selectors_overworld(&mut self, ui: &mut Ui, ctx: &mut EditorToolTabViewer) {
        if ui.checkbox(&mut self.special_completed, "Special world completed").changed() {
            log::info!(
                "Showing color palette for {}",
                if self.special_completed { "post-special world" } else { "pre-special world" }
            );
            self.update_palette_image(ui, ctx);
        }

        let submap_count = {
            let project = ctx.project_ref.as_ref().unwrap().borrow();
            project.rom_data.color_palettes.ow_specific_set.layer2_indices.len() as i32
        };

        ui.horizontal(|ui| {
            let drag_submap_num =
                DragValue::new(&mut self.submap_num).clamp_range(0..=submap_count - 1).hexadecimal(1, false, true);
            if ui.add(drag_submap_num).changed() {
                log::info!("Showing color palette for submap {:X}", self.submap_num);
                self.update_palette_image(ui, ctx);
            }
            ui.label("Submap number");
        });
    }

    fn update_palette_image(&mut self, ui: &mut Ui, ctx: &mut EditorToolTabViewer) {
        let project = ctx.project_ref.as_ref().unwrap().borrow();
        let rom = &project.rom_data;
        self.palette_image = Some(match self.palette_context {
            PaletteContext::Level => {
                let header = &rom.levels[self.level_num as usize].primary_header;
                let palette = &rom.color_palettes.get_level_palette(header).unwrap();
                let frames = palette
                    .animated
                    .iter()
                    .map(|&animated_color| {
                        let mut image = ColorImage::new([16, 16], Color32::BLACK);
                        for y in 0..=0xF {
                            for x in 0..=0xF {
                                if y == 6 && x == 4 {
                                    image[(x, y)] = Color32::from(animated_color);
                                } else {
                                    let color = palette.get_color_at(y, x).unwrap();
                                    image[(x, y)] = Color32::from(color);
                                }
                            }
                        }
                        image
                    })
                    .collect_vec();
                PaletteImage::Animated(
                    AnimationState::from_frames(frames, "palette-image", ui.ctx())
                        .expect("Cannot assemble animation for palette image"),
                )
            }
            PaletteContext::Overworld => {
                let ow_state =
                    if self.special_completed { OverworldState::PostSpecial } else { OverworldState::PreSpecial };
                let palette = &rom.color_palettes.get_submap_palette(self.submap_num as usize, ow_state).unwrap();
                let mut image = ColorImage::new([16, 16], Color32::BLACK);
                for y in 0..=0xF {
                    for x in 0..=0xF {
                        let color = palette.get_color_at(y, x).unwrap();
                        image[(x, y)] = Color32::from(color);
                    }
                }

                PaletteImage::Static(ui.ctx().load_texture("palette-image", image, TextureOptions::NEAREST))
            }
        });
    }

    fn display_palette(&mut self, ui: &mut Ui) {
        const CELL_SIZE: f32 = 20.0;
        let label_size = tweak!(18.0);
        let (mut rect, _) =
            ui.allocate_exact_size(Vec2::splat(16.0 * CELL_SIZE + label_size), Sense::focusable_noninteractive());

        // Row and column labels
        for col in 0..=0xF {
            let mut cpos = rect.min;
            let mut rpos = rect.min;
            cpos.x += (1 + col) as f32 * CELL_SIZE + tweak!(3.0);
            rpos.y += (1 + col) as f32 * CELL_SIZE;
            ui.painter().text(
                cpos,
                Align2::LEFT_TOP,
                format!("{col:X}"),
                FontId::proportional(label_size),
                ui.visuals().text_color(),
            );
            ui.painter().text(
                rpos,
                Align2::LEFT_TOP,
                format!("{col:X}"),
                FontId::proportional(label_size),
                ui.visuals().text_color(),
            );
        }

        // Palette image
        rect.min += Vec2::splat(label_size);
        let image = match self.palette_image.as_mut().unwrap() {
            PaletteImage::Static(handle) => {
                Shape::image(handle.id(), rect, Rect::from_x_y_ranges(0.0..=1.0, 0.0..=1.0), Color32::WHITE)
            }
            PaletteImage::Animated(animation) => {
                Flipbook::new(animation, rect.size()).fps(tweak!(15.0)).looped(true).to_shape(ui, rect)
            }
        };
        ui.painter().add(image);
    }
}
