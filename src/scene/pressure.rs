use std::collections::HashMap;
use std::vec::Vec;

use super::objects;
use super::Scene;

pub fn generate() -> Scene {

    let object1_sx = 50;
    let object1_sy = 50;
    let spacing1 = 0.04;

    let mut nodes1 = objects::build_rectangle(object1_sx, object1_sy, spacing1, -1.0, -0.9, 0.01, 100.0, 1);
    let connections_map_1 = objects::build_connections_map(&nodes1, spacing1 * 1.5, 0.1, 0);


    let mut connections_map: HashMap<(usize, usize), (f32, f32)> = HashMap::new();
    connections_map.extend(connections_map_1);

    let mut nodes = Vec::new();
    nodes.append(&mut nodes1);

    Scene {
        nodes: nodes,
        connections: connections_map,
        object_repulsion_dx: 0.05,
        object_repulsion_v0: 100.0,
    }
}
