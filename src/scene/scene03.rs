use std::collections::HashMap;
use std::vec::Vec;

use super::objects;
use super::Scene;

pub fn generate() -> Scene {
    let object1_sx = 180;
    let object1_sy = 90;
    let spacing1 = 0.01;

    let object2_sx = 40;
    let object2_sy = 80;
    let object2_m = 15.0;
    let spacing2 = 0.01;

    let mut nodes1 = objects::build_rectangle(object1_sx, object1_sy, spacing1, -0.92, -0.925, 0.35, 0.8, 1);
    let connections_map_1 = objects::build_connections_map(&nodes1, spacing1 * 1.5, 100.0, 0);

    let mut nodes2 = objects::build_rectangle(object2_sx, object2_sy, spacing2, -0.3, 0.4, object2_m, 0.2, 2);
    let connections_map_2 = objects::build_connections_map(&nodes2, spacing2 * 1.5, 400.0, nodes1.len());

    let mut connections_map: HashMap<(usize, usize), (f32, f32)> = HashMap::new();
    connections_map.extend(connections_map_1);
    connections_map.extend(connections_map_2);

    let mut nodes = Vec::new();
    nodes.append(&mut nodes1);
    nodes.append(&mut nodes2);

    Scene {
        nodes: nodes,
        connections: connections_map,
        object_repulsion_dx: 0.015,
        object_repulsion_v0: 100.0,
    }
}