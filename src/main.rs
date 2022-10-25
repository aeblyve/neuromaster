use kiss3d::scene::SceneNode;
use macroquad::prelude::*;
use nalgebra::Point3;
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::EdgeIndex;
use rust_nmap::parse_nmap_xml_bytes;
use std::collections::HashMap;

use kiss3d::light::Light;
use kiss3d::nalgebra::{UnitQuaternion, Vector3};
use kiss3d::window::Window;

mod simulation;

fn main() {
    let full_parse = parse_nmap_xml_bytes(include_bytes!("../assets/scan.xml")).unwrap();
    let mut simulation = simulation::build_simulation(full_parse).unwrap();
    let mut node_map = HashMap::<NodeIndex, SceneNode>::new();

    let mut window = Window::new("Neuroquad");
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

    while window.render() {
        simulation.update(0.035);
        let graph = simulation.get_graph();
        for node_index in graph.node_indices() {
            let node_weight = graph.node_weight(node_index).unwrap();
            let scene_node = node_map.get_mut(&node_index).unwrap();
            let translation = nalgebra::Translation3::new(
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
