use fdg_sim::petgraph::graph::NodeIndex;
use fdg_sim::Simulation;
use kiss3d::conrod;
use kiss3d::conrod::color::Color;
use kiss3d::conrod::position::Positionable;
use kiss3d::conrod::widget_ids;
use kiss3d::light::Light;
use kiss3d::window::Window;
use std::path::Path;

use crate::simulation::SimpleHost;

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
        ip_text,
    }
}

pub struct ApplicationState {
    pub simulation: Simulation<SimpleHost, ()>,
    pub node_selected: Option<fdg_sim::petgraph::graph::NodeIndex>,
}

impl ApplicationState {
    pub fn new(simulation: Simulation<SimpleHost, ()>) -> Self {
        ApplicationState {
            simulation,
            node_selected: None,
        }
    }

    pub fn selected_ip(&self) -> String {
        let graph = self.simulation.get_graph();

        match self.node_selected {
            None => String::from(""),
            Some(n) => graph.node_weight(n).unwrap().data.main_addr.to_string(),
        }
    }
}

pub fn gui(ui: &mut conrod::UiCell, ids: &Ids, state: &mut ApplicationState) {
    use conrod::{widget, Colorable, Labelable, Sizeable, Widget};
    use std::iter::once;

    const MARGIN: conrod::Scalar = 30.0;

    widget::Canvas::new()
        .pad(MARGIN)
        .align_right()
        .w(200.0)
        .scroll_kids_vertically()
        .set(ids.canvas, ui);

    // foo
    for event in widget::TextBox::new(&state.selected_ip())
        .mid_top_of(ids.canvas)
        .align_middle_x_of(ids.canvas)
        .padded_w_of(ids.canvas, MARGIN)
        .h(40.0)
        .set(ids.ip_text, ui)
    {
        use conrod::widget::text_box::Event;
        match event {
            Event::Enter => {}
            Event::Update(s) => {}
        }
    }
}
