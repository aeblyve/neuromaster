use bimap::BiMap;
use kiss3d::camera::*;
use kiss3d::event::{Action, WindowEvent};
use kiss3d::light::Light;
use kiss3d::nalgebra::Point3;
use kiss3d::nalgebra::{UnitQuaternion, Vector3};
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use macroquad::prelude::*;
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::EdgeIndex;
use rust_nmap::parse_nmap_xml_bytes;
use std::collections::HashMap;

mod gui;
mod simulation;

// TODO: text on a mouse event

fn main() {
    //let full_parse = parse_nmap_xml_bytes(include_bytes!("../assets/scan.xml")).unwrap();
    let full_parse = parse_nmap_xml_bytes(include_bytes!("../assets/huge.xml")).unwrap();
    let mut simulation = simulation::build_simulation(full_parse).unwrap();
    let mut node_map = HashMap::<NodeIndex, SceneNode>::new();
    //let mut node_scene_map = BiMap::<NodeIndex, SceneNode>::new();

    let mut window = Window::new("Neuroquad");
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
        scene_node.set_color(1.0, 0.0, 0.0);
        scene_node.set_points_size(10.0);
        scene_node.set_lines_width(1.0);
        scene_node.set_surface_rendering_activation(false);
        scene_node
    }

    let mut last_pos = kiss3d::nalgebra::Point2::new(0.0f32, 0.0f32);
    while window.render_with_camera(&mut camera) {
        for event in window.events().iter() {
            match event.value {
                WindowEvent::FramebufferSize(x, y) => {
                    println!("Frame buffer is {}x{}", x, y);
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
                    let selected_node_index = simulation::find_closest_intersection(
                        ray_origin,
                        ray_direction,
                        &simulation,
                    )
                    .unwrap();
                    let mut scene_node = node_map.get_mut(&selected_node_index).unwrap();
                    scene_node.set_color(0.0, 0.0, 1.0);
                }
                WindowEvent::CursorPos(x, y, _modif) => {
                    last_pos = kiss3d::nalgebra::Point2::new(x as f32, y as f32);
                }
                _ => {}
            }
        }

        //let mut ui = window.conrod_ui_mut().set_widgets();
        //gui::gui(&mut ui, &ids, &mut gui::ApplicationState::new());

        simulation.update(0.035);
        let graph = simulation.get_graph();
        for node_index in graph.node_indices() {
            let node_weight = graph.node_weight(node_index).unwrap();
            let scene_node = node_map.get_mut(&node_index).unwrap();
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
