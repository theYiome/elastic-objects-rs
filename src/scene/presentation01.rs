use std::collections::HashMap;
use std::vec::Vec;

use super::objects;
use super::Scene;

pub fn generate() -> Scene {

    let object1_sx = 22;
    let object1_sy = 16;
    let object1_st = object1_sx * object1_sy;
    let spacing1 = 0.02;

    let object2_sx = 12;
    let object2_sy = 8;
    let object2_st = object2_sx * object2_sy;

    let object3_sx = 7;
    let object3_sy = 20;
    let object3_st = object3_sx * object3_sy;

    let mut nodes1 = objects::build_rectangle(object1_sx, object1_sy, spacing1, -0.92, -0.7, 0.5, 0.0, 1);
    let connections_map_1 = objects::build_connections_map(&nodes1, spacing1 * 1.5, 50.0, 0);

    let mut nodes2 = objects::build_rectangle(object2_sx, object2_sy, spacing1, -0.72, -0.925, 1.0, 0.0, 2);
    let connections_map_2 = objects::build_connections_map(&nodes2, spacing1 * 1.5, 50.0, object1_st);

    // let mut nodes3 = objects::build_rectangle(object3_sx, object3_sy, spacing1, -1.12, -0.935, 200.0, 0.0, 3);
    // let connections_map_3 = objects::build_connections_map(&nodes3, spacing1 * 1.5, 1000.0, object1_st + object2_st);

    // let mut nodes4 = objects::build_rectangle(object3_sx, object3_sy, spacing1, -0.4, -0.935, 200.0, 0.0, 3);
    // let connections_map_4 = objects::build_connections_map(&nodes4, spacing1 * 1.5, 1000.0, object1_st + object2_st + object3_st);

    let mut connections_map: HashMap<(usize, usize), (f32, f32)> = HashMap::new();
    connections_map.extend(connections_map_1);
    connections_map.extend(connections_map_2);
    // connections_map.extend(connections_map_3);
    // connections_map.extend(connections_map_4);

    let mut nodes = Vec::new();
    nodes.append(&mut nodes1);
    nodes.append(&mut nodes2);
    // nodes.append(&mut nodes3);
    // nodes.append(&mut nodes4);

    Scene {
        nodes: nodes,
        connections: connections_map,
        object_repulsion_dx: 0.015,
        object_repulsion_v0: 100.0,
    }
}
