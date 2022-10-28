use kiss3d::conrod;
use kiss3d::conrod::color::Color;
use kiss3d::conrod::position::Positionable;
use kiss3d::conrod::widget_ids;
use kiss3d::light::Light;
use kiss3d::window::Window;
use std::path::Path;

pub fn theme() -> conrod::Theme {
    use conrod::position::{Align, Direction, Padding, Position, Relative};
    conrod::Theme {
        name: "Demo Theme".to_string(),
        padding: Padding::none(),
        x_position: Position::Relative(Relative::Align(Align::Start), None),
        y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
        background_color: conrod::color::DARK_CHARCOAL,
        shape_color: conrod::color::LIGHT_CHARCOAL,
        border_color: conrod::color::BLACK,
        border_width: 0.0,
        label_color: conrod::color::WHITE,
        font_id: None,
        font_size_large: 26,
        font_size_medium: 18,
        font_size_small: 12,
        widget_styling: conrod::theme::StyleMap::default(),
        mouse_drag_threshold: 0.0,
        double_click_threshold: std::time::Duration::from_millis(500),
    }
}

widget_ids! {
    pub struct Ids {
        canvas, // backdrop for other widgets
    }
}

pub struct ApplicationState {
    ip_selected: Option<String>,
}

impl ApplicationState {
    pub fn new() -> Self {
        ApplicationState { ip_selected: None }
    }
}

pub fn gui(ui: &mut conrod::UiCell, ids: &Ids, state: &mut ApplicationState) {
    use conrod::{widget, Colorable, Labelable, Sizeable, Widget};
    use std::iter::once;

    const MARGIN: conrod::Scalar = 30.0;

    widget::Canvas::new()
        .pad(MARGIN)
        .align_right()
        .w(600.0)
        .scroll_kids_vertically()
        .set(ids.canvas, ui);
}
