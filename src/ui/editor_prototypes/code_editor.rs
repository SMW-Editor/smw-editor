use egui::{TextEdit, Ui, WidgetText};
use egui_extras::{Column, TableBuilder};
use smwe_rom::disassembler::serialization::LineKind;

use crate::ui::{frame_context::EditorToolTabViewer, tool::DockableEditorTool};

#[derive(Default)]
pub struct UiCodeEditor {}

impl DockableEditorTool for UiCodeEditor {
    fn update(&mut self, ui: &mut Ui, ctx: &mut EditorToolTabViewer) {
        let min_scroll_height = ui.available_height();
        TableBuilder::new(ui)
            .min_scrolled_height(min_scroll_height)
            .columns(Column::remainder().at_least(300.), 2)
            .body(|tb| {
                let mut project = ctx.project_ref.as_mut().unwrap().borrow_mut();
                tb.rows(15., project.rom_data.disassembly.code_lines.len(), |i, mut tr| {
                    match &mut project.rom_data.disassembly.code_lines[i] {
                        LineKind::Label { label, comment } => {
                            tr.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.add(
                                        TextEdit::singleline(label).code_editor().clip_text(false).desired_width(0.),
                                    );
                                    ui.monospace(":");
                                });
                            });
                            tr.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.monospace(if comment.is_empty() { " " } else { ";" });
                                    ui.add(
                                        TextEdit::singleline(comment).code_editor().clip_text(false).desired_width(0.),
                                    );
                                });
                            });
                        }
                        LineKind::Op { op, comment } => {
                            tr.col(|ui| {
                                ui.indent("line-indent", |ui| {
                                    ui.add(TextEdit::singleline(op).code_editor().clip_text(false).desired_width(0.));
                                });
                            });
                            tr.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.monospace(if comment.is_empty() { " " } else { ";" });
                                    ui.add(
                                        TextEdit::singleline(comment).code_editor().clip_text(false).desired_width(0.),
                                    );
                                });
                            });
                        }
                        LineKind::Empty {} => {
                            tr.col(|_| {});
                            tr.col(|_| {});
                        }
                        _ => {}
                    }
                })
            });
    }

    fn title(&self) -> WidgetText {
        "Code editor".into()
    }
}
