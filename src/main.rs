#![feature(fn_traits)]

use bimap::BiMap;
use fdg_sim::petgraph::graph::NodeIndex;
use fdg_sim::petgraph::stable_graph::EdgeIndex;
use kiss3d::camera::*;
use kiss3d::conrod;
use kiss3d::event::{Action, WindowEvent};
use kiss3d::light::Light;
use kiss3d::nalgebra::Point3;
use kiss3d::nalgebra::{UnitQuaternion, Vector3};
use kiss3d::resource::TextureManager;
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use macroquad::prelude::*;
use rust_nmap::parse_nmap_xml_bytes;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use fdg_sim::Simulation;
use kiss3d::conrod::color::Color;
use kiss3d::conrod::position::Positionable;
use kiss3d::conrod::widget_ids;

use crate::simulation::{OsGuess, SimpleHost};

mod simulation;

const SELECTED_COLOR: (f32, f32, f32) = (0.0, 0.0, 1.0);
const DEFAULT_COLOR: (f32, f32, f32) = (1.0, 0.0, 0.0);

trait WindowExt {
    fn alloc_conrod_texture(&mut self, bytes: &[u8], name: &str) -> kiss3d::conrod::image::Id;
}

impl WindowExt for Window {
    fn alloc_conrod_texture(&mut self, bytes: &[u8], name: &str) -> kiss3d::conrod::image::Id {
        TextureManager::get_global_manager(|tm| tm.add_image_from_memory(bytes, name));
        self.conrod_texture_id(name).unwrap()
    }
}

trait SceneNodeExt {
    fn paint_default(&mut self);
    fn paint_selected(&mut self);
}

impl SceneNodeExt for SceneNode {
    fn paint_default(&mut self) {
        self.set_color(DEFAULT_COLOR.0, DEFAULT_COLOR.1, DEFAULT_COLOR.2);
    }

    fn paint_selected(&mut self) {
        self.set_color(SELECTED_COLOR.0, SELECTED_COLOR.1, SELECTED_COLOR.2);
    }
}

fn main() {
    let full_parse = parse_nmap_xml_bytes(include_bytes!("../assets/bigger.xml")).unwrap();
    //let full_parse = parse_nmap_xml_bytes(include_bytes!("../assets/huge.xml")).unwrap();
    let simulation = simulation::build_simulation(full_parse).unwrap();
    let mut node_map = HashMap::<NodeIndex, SceneNode>::new();

    let mut window = Window::new("Neuromaster");

    let tux_texture = window.alloc_conrod_texture(include_bytes!("../assets/tux.png"), "tux");
    let puffy_texture = window.alloc_conrod_texture(include_bytes!("../assets/puffy.png"), "puffy");
    let daemon_texture =
        window.alloc_conrod_texture(include_bytes!("../assets/daemon.png"), "daemon");

    let mut camera = kiss3d::camera::ArcBall::new(Point3::new(0.0f32, 0.0, -1.0), Point3::origin());
    window.set_light(Light::StickToCamera);

    // build nodes
    for node_index in simulation.get_graph().node_indices() {
        let scene_node = wireframe_sphere(&mut window);
        node_map.insert(node_index, scene_node);
    }

    // saves resources AND looks very "Neuromancer"
    fn wireframe_sphere(window: &mut Window) -> SceneNode {
        let mut scene_node = window.add_sphere(1.0);
        scene_node.paint_default();
        scene_node.set_points_size(10.0);
        scene_node.set_lines_width(1.0);
        scene_node.set_surface_rendering_activation(false);
        scene_node
    }

    let ids = Ids::new(window.conrod_ui_mut().widget_id_generator());
    window.conrod_ui_mut().theme = theme();

    let mut application_state = ApplicationState::new(
        simulation,
        node_map,
        tux_texture,
        puffy_texture,
        daemon_texture,
    );

    let mut last_pos = kiss3d::nalgebra::Point2::new(0.0f32, 0.0f32);
    {
        let mut ui = window.conrod_ui_mut().set_widgets();
        application_state.gui(&mut ui, &ids);
    }
    while window.render_with_camera(&mut camera) {
        for event in window.events().iter() {
            match event.value {
                WindowEvent::FramebufferSize(x, y) => {
                    println!("Frame buffer is {}x{}. Resizing.", x, y);
                    let mut ui = window.conrod_ui_mut().set_widgets();
                    application_state.gui(&mut ui, &ids);
                }
                WindowEvent::MouseButton(button, Action::Press, modif) => {
                    println!("Mouse press event on {:?} with {:?}", button, modif);
                    let window_size = kiss3d::nalgebra::Vector2::new(
                        window.size()[0] as f32,
                        window.size()[1] as f32,
                    );
                    let (ray_origin, ray_direction) = camera.unproject(&last_pos, &window_size);

                    println!(
                        "Created ray with origin {} and direction {}",
                        ray_origin, ray_direction
                    );
                    application_state.select_nearest_intersection(ray_origin, ray_direction);
                    let mut ui = window.conrod_ui_mut().set_widgets();
                    application_state.gui(&mut ui, &ids);
                }
                WindowEvent::CursorPos(x, y, _modif) => {
                    last_pos = kiss3d::nalgebra::Point2::new(x as f32, y as f32);
                }
                _ => {}
            }
        }

        application_state.simulation.update(0.035);
        let graph = application_state.simulation.get_graph();
        for node_index in graph.node_indices() {
            let node_weight = graph.node_weight(node_index).unwrap();
            let scene_node = application_state.node_map.get_mut(&node_index).unwrap();
            let translation = kiss3d::nalgebra::Translation3::new(
                node_weight.location.x,
                node_weight.location.y,
                node_weight.location.z,
            );
            scene_node.set_local_translation(translation);

            for neighbor_index in graph.neighbors(node_index) {
                let neighbor_weight = &graph.node_weight(neighbor_index).unwrap();
                window.draw_line(
                    &Point3::new(
                        node_weight.location.x,
                        node_weight.location.y,
                        node_weight.location.z,
                    ),
                    &Point3::new(
                        neighbor_weight.location.x,
                        neighbor_weight.location.y,
                        neighbor_weight.location.z,
                    ),
                    &Point3::new(0.0, 1.0, 0.0),
                );
            }
        }
    }
}

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
        os_image,
    }
}

pub struct ApplicationState {
    pub simulation: Simulation<SimpleHost, ()>,
    node_selected: Option<fdg_sim::petgraph::graph::NodeIndex>,
    node_map: HashMap<NodeIndex, SceneNode>,
    selected_os_texture: Option<kiss3d::conrod::image::Id>,
    tux_texture: kiss3d::conrod::image::Id,
    puffy_texture: kiss3d::conrod::image::Id,
    daemon_texture: kiss3d::conrod::image::Id,
}

impl ApplicationState {
    pub fn new(
        simulation: Simulation<SimpleHost, ()>,
        node_map: HashMap<NodeIndex, SceneNode>,
        tux_texture: kiss3d::conrod::image::Id,
        puffy_texture: kiss3d::conrod::image::Id,
        daemon_texture: kiss3d::conrod::image::Id,
    ) -> Self {
        ApplicationState {
            simulation,
            node_selected: None,
            node_map,
            selected_os_texture: None,
            tux_texture,
            puffy_texture,
            daemon_texture,
        }
    }

    pub fn select_nearest_intersection(
        &mut self,
        ray_origin: Point3<f32>,
        ray_direction: Vector3<f32>,
    ) {
        let int = self.find_closest_intersection(ray_origin, ray_direction);
        self.set_selected_node(int);
    }

    /// Return the required IP for the current selected node, if it exists.
    pub fn get_selected_ip(&self) -> Option<String> {
        self.node_selected.map(|n| {
            self.simulation
                .get_graph()
                .node_weight(n)
                .unwrap()
                .data
                .main_addr
                .to_string()
        })
    }

    /// Return the optional OsGuess for the current selected node, if it exists.
    pub fn get_selected_os(&self) -> Option<Option<OsGuess>> {
        match self.node_selected {
            None => None,
            Some(n) => {
                let guess = self
                    .simulation
                    .get_graph()
                    .node_weight(n)
                    .unwrap()
                    .data
                    .os_guess
                    .clone();
                Some(guess)
            }
        }
    }

    pub fn set_os_texture(&mut self) {
        let setting = match self.get_selected_os() {
            None => None,
            Some(guess) => match guess.unwrap() {
                OsGuess::LINUX(_) => Some(self.tux_texture),
                OsGuess::FREEBSD(_) => Some(self.daemon_texture),
                OsGuess::OPENBSD(_) => Some(self.puffy_texture),
                OsGuess::OTHER(_) => None,
            },
        };
        self.selected_os_texture = setting;
    }

    /// Set the selected node(index) to the given one. Paints scene nodes accordingly.
    pub fn set_selected_node(
        &mut self,
        selected_node: Option<fdg_sim::petgraph::graph::NodeIndex>,
    ) {
        if self.node_selected.is_some() {
            self.node_map
                .get_mut(&self.node_selected.unwrap())
                .unwrap()
                .paint_default();
        }

        self.node_selected = selected_node;
        if self.node_selected.is_some() {
            let graph = self.simulation.get_graph();
            let node_weight = graph.node_weight(selected_node.unwrap()).unwrap();
            self.node_map
                .get_mut(&self.node_selected.unwrap())
                .unwrap()
                .paint_selected();
        }
        self.set_os_texture();
    }

    pub fn gui(&mut self, ui: &mut conrod::UiCell, ids: &Ids) {
        use conrod::{widget, Colorable, Labelable, Sizeable, Widget};
        use std::iter::once;

        const MARGIN: conrod::Scalar = 10.0;

        widget::Canvas::new()
            .pad(MARGIN)
            .align_right()
            .w(200.0)
            .scroll_kids_vertically()
            .set(ids.canvas, ui);

        let ip = self.get_selected_ip();
        if ip.is_some() {
            for event in widget::TextBox::new(&ip.unwrap())
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

        if self.selected_os_texture.is_some() {
            widget::Image::new(self.selected_os_texture.unwrap())
                .w_h(144.0, 144.0)
                .down(20.0)
                .align_middle_x_of(ids.canvas)
                .set(ids.os_image, ui);
        }
    }

    /// Given a ray with origin and direction, find the closest node (modeled as a sphere centered on node.location) in the simulation intersecting the ray, if it exists.
    pub fn find_closest_intersection(
        &self,
        ray_origin: Point3<f32>,
        ray_direction: Vector3<f32>,
    ) -> Option<NodeIndex> {
        let radius = 1.0; // magic number for now - might change with amount of edges?
        let graph = self.simulation.get_graph();

        let mut least_distance = f32::MAX;
        let mut closest_node: Option<NodeIndex> = None;

        for node_index in graph.node_indices() {
            let node_weight = graph.node_weight(node_index).unwrap();
            let sphere_center = Point3::new(
                node_weight.location.x,
                node_weight.location.y,
                node_weight.location.z,
            );
            let difference: Vector3<f32> = ray_origin - sphere_center;
            let difference_sqr = difference.dot(&difference);
            let p = ray_direction.dot(&difference);

            let determinant = p * p - difference_sqr + radius * radius;
            println!(
                "For the sphere centered at {}, determinant is {}",
                sphere_center, determinant
            );

            if determinant < 0.0 {
                continue; // no (real) intersection
            } else if determinant.abs() < f32::EPSILON {
                // one intersection, log it
                let distance = ray_direction.scale(-1.0).dot(&difference);
                least_distance = least_distance.min(distance);
                if distance < least_distance {
                    least_distance = distance;
                    closest_node = Some(node_index);
                }
            } else {
                // two intersections, log the closest one
                let distance1 = ray_direction.scale(-1.0).dot(&difference) - determinant.sqrt();
                let distance2 = ray_direction.scale(-1.0).dot(&difference) + determinant.sqrt();
                let distance = distance1.min(distance2);

                if distance < least_distance {
                    least_distance = distance;
                    closest_node = Some(node_index);
                }
            }
        }
        closest_node
    }
}
