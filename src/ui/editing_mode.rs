use egui::{CursorIcon, PointerButton, Pos2, Rect, Response};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum EditingMode {
    Draw,
    Erase,
    Move,
    Probe,
    Select,
}

pub enum Selection {
    Click(Option<Pos2>),
    Drag(Option<Rect>),
}

impl EditingMode {
    pub fn hover_cursor_icon(self) -> CursorIcon {
        // todo: custom icons
        match self {
            Self::Draw => CursorIcon::Default,
            Self::Erase => CursorIcon::Default,
            Self::Move => CursorIcon::Default,
            Self::Probe => CursorIcon::Default,
            Self::Select => CursorIcon::Default,
        }
    }

    pub fn inserted(self, response: &Response) -> bool {
        match self {
            Self::Move => response.double_clicked_by(PointerButton::Primary),
            Self::Draw => response.clicked_by(PointerButton::Primary) || response.dragged_by(PointerButton::Primary),
            _ => false,
        }
    }

    pub fn moved(self, response: &Response) -> bool {
        match self {
            Self::Move => response.dragged_by(PointerButton::Primary),
            _ => false,
        }
    }

    pub fn selected(self, response: &Response) -> Option<Selection> {
        match self {
            Self::Move => {
                response.clicked_by(PointerButton::Primary).then(|| Selection::Click(response.interact_pointer_pos()))
            }
            Self::Select => response.dragged_by(PointerButton::Primary).then(|| {
                Selection::Drag(match response.ctx.input(|i| (i.pointer.press_origin(), i.pointer.interact_pos())) {
                    (Some(origin), Some(current)) => Some(Rect::from_two_pos(origin, current)),
                    _ => None,
                })
            }),
            _ => None,
        }
    }

    pub fn erased(self, response: &Response) -> bool {
        match self {
            Self::Erase => response.clicked_by(PointerButton::Primary) || response.dragged_by(PointerButton::Primary),
            _ => false,
        }
    }

    pub fn probed(self, response: &Response) -> bool {
        match self {
            Self::Probe => response.clicked_by(PointerButton::Primary),
            _ => false,
        }
    }
}
