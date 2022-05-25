use std::collections::HashMap;
use std::vec::Vec;

use super::objects;
use super::Scene;

pub fn generate() -> Scene {

    let object1_sx = 24;
    let object1_sy = 12;
    let object1_st = object1_sx * object1_sy;
    let spacing1 = 0.08;

    let spacing2 = 0.075;

    let mut nodes1 = objects::build_rectangle(object1_sx, object1_sy, spacing1, -0.92, -0.925, 1.0, 0.0, 1);
    let connections_map_1 = objects::build_connections_map(&nodes1, spacing1 * 1.5, 70.0, 0);

    let mut nodes2 = objects::build_circle(4, spacing2, -0.12, 0.8, 2.0, 0.0, 2);
    let connections_map_2 = objects::build_connections_map(&nodes2, spacing2 * 1.5, 300.0, object1_st);

    let mut connections_map: HashMap<(usize, usize), (f32, f32)> = HashMap::new();
    connections_map.extend(connections_map_1);
    connections_map.extend(connections_map_2);

    let mut nodes = Vec::new();
    nodes.append(&mut nodes1);
    nodes.append(&mut nodes2);

    Scene {
        nodes: nodes,
        connections: connections_map
    }
}
