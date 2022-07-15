use std::collections::HashMap;
use std::vec::Vec;

use super::objects;
use super::Scene;

pub fn generate() -> Scene {

    let object1_sx = 50;
    let object1_sy = 50;
    let spacing1 = 0.04;

    let object2_sx = 15;
    let object2_sy = 15;
    let spacing2 = 0.04;

    let mut nodes1 = objects::build_rectangle(object1_sx, object1_sy, spacing1, -1.0, -0.9, 0.01, 100.0, 1);
    let connections_map_1 = objects::build_connections_map(&nodes1, spacing1 * 1.5, 0.1, 0);

    let mut nodes2 = objects::build_rectangle(object2_sx, object2_sy, spacing2, -0.75, 1.4, 0.02, 20.0, 2);
    let connections_map_2 = objects::build_connections_map(&nodes2, spacing2 * 1.5, 0.1, object1_sx * object1_sy);

    let mut connections_map: HashMap<(usize, usize), (f32, f32)> = HashMap::new();
    connections_map.extend(connections_map_1);
    connections_map.extend(connections_map_2);

    let mut nodes = Vec::new();
    nodes.append(&mut nodes1);
    nodes.append(&mut nodes2);

    Scene {
        nodes: nodes,
        connections: connections_map,
        object_repulsion_dx: 0.03,
        object_repulsion_v0: 100.0,
    }
}
