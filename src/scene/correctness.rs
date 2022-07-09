use std::collections::HashMap;
use std::vec::Vec;

use super::objects;
use super::Scene;

pub fn generate() -> Scene {

    let object1_sx = 8;
    let object1_sy = 8;
    let object1_st = object1_sx * object1_sy;
    let spacing1 = 0.08;

    let object2_sx = 6;
    let object2_sy = 4;

    let mut nodes1 = objects::build_rectangle(object1_sx, object1_sy, spacing1, -0.1, -0.925, 1.0, 0.0, 1);
    let connections_map_1 = objects::build_connections_map(&nodes1, spacing1 * 1.5, 70.0, 0);

    let mut nodes2 = objects::build_rectangle(object2_sx, object2_sy, spacing1, 0.0, -0.08, 1.0, 0.0, 2);
    let connections_map_2 = objects::build_connections_map(&nodes2, spacing1 * 1.5, 70.0, object1_st);

    let mut connections_map: HashMap<(usize, usize), (f32, f32)> = HashMap::new();
    connections_map.extend(connections_map_1);
    connections_map.extend(connections_map_2);

    let mut nodes = Vec::new();
    nodes.append(&mut nodes1);
    nodes.append(&mut nodes2);

    Scene {
        nodes: nodes,
        connections: connections_map,
        object_repulsion_dx: 0.06,
        object_repulsion_v0: 100.0,
    }
}
