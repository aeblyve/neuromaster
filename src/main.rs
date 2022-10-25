use kiss3d::scene::SceneNode;
use macroquad::prelude::*;
use rust_nmap::parse_nmap_xml_bytes;
use std::collections::HashMap;

use kiss3d::light::Light;
use kiss3d::nalgebra::{UnitQuaternion, Vector3};
use kiss3d::window::Window;

mod simulation;

fn main() {
    let full_parse = parse_nmap_xml_bytes(include_bytes!("../assets/scan.xml")).unwrap();
    let mut simulation = simulation::build_simulation(full_parse).unwrap();
    let mut scene_map = HashMap::<String, SceneNode>::new();

    let mut window = Window::new("Neuroquad");
    window.set_light(Light::StickToCamera);

    for node in simulation.get_graph().node_weights() {
        let scene_node = wireframe_sphere(&mut window);
        scene_map.insert(node.name.clone(), scene_node);
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

    while window.render() {
        simulation.update(0.035);
        for node in simulation.get_graph().node_weights() {
            let scene_node = scene_map.get_mut(&node.name).unwrap();
            let translation =
                nalgebra::Translation3::new(node.location.x, node.location.y, node.location.z);
            scene_node.set_local_translation(translation);
        }
    }
}
