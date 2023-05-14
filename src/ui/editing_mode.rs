use egui::{CursorIcon, PointerButton, Response};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum EditingMode {
    Draw,
    Erase,
    Insert,
    Paint,
    Probe,
    Select,
}

impl EditingMode {
    pub fn hover_cursor_icon(self) -> CursorIcon {
        // todo: custom icons
        match self {
            Self::Draw => CursorIcon::Default,
            Self::Erase => CursorIcon::Default,
            Self::Insert => CursorIcon::Default,
            Self::Paint => CursorIcon::Default,
            Self::Probe => CursorIcon::Default,
            Self::Select => CursorIcon::Default,
        }
    }

    pub fn inserted(self, response: &Response) -> bool {
        match self {
            Self::Insert => response.double_clicked_by(PointerButton::Primary),
            Self::Draw | Self::Paint => {
                response.clicked_by(PointerButton::Primary) || response.dragged_by(PointerButton::Primary)
            }
            _ => false,
        }
    }

    pub fn moved(self, response: &Response) -> bool {
        match self {
            Self::Insert => response.dragged_by(PointerButton::Primary),
            _ => false,
        }
    }

    pub fn selected(self, response: &Response) -> bool {
        match self {
            Self::Select => response.dragged_by(PointerButton::Primary),
            _ => false,
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
