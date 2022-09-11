use std::collections::HashMap;
use std::vec::Vec;

use super::objects;
use super::Scene;

pub fn generate() -> Scene {

    let object1_sx = 42;
    let object1_sy = 10;
    let object1_st = object1_sx * object1_sy;
    let spacing1 = 0.01;

    let object2_sx = 12;
    let object2_sy = 8;
    let object2_st = object2_sx * object2_sy;

    let object3_sx = 7;
    let object3_sy = 20;
    let object3_st = object3_sx * object3_sy;

    let mut nodes = Vec::new();
    let mut connections_map: HashMap<(usize, usize), (f32, f32)> = HashMap::new();
    let damping = 30.0;

    let mut nodes1 = objects::build_rectangle(object1_sx, object1_sy, spacing1, -0.91, -0.81, 0.5, damping, 1);
    let connections_map_1 = objects::build_connections_map(&nodes1, spacing1 * 1.5, 5.0, nodes.len());
    nodes.append(&mut nodes1);
    connections_map.extend(connections_map_1);
    
    let mut nodes2 = objects::build_rectangle(10, 10, spacing1, -0.92, -0.92, 5.0, damping, 2);
    let connections_map_2 = objects::build_connections_map(&nodes2, spacing1 * 1.5, 3.0, nodes.len());
    nodes.append(&mut nodes2);
    connections_map.extend(connections_map_2);
    
    let mut nodes3 = objects::build_rectangle(10, 10, spacing1, -0.58, -0.92, 5.0, damping, 3);
    let connections_map_3 = objects::build_connections_map(&nodes3, spacing1 * 1.5, 3.0, nodes.len());
    nodes.append(&mut nodes3);
    connections_map.extend(connections_map_3);

    let mut nodes4 = objects::build_circle(6, spacing1, -0.71, -0.88, 3.0, damping, 4);
    let connections_map_4 = objects::build_connections_map(&nodes4, spacing1 * 1.5, 20.0, nodes.len());
    nodes.append(&mut nodes4);
    connections_map.extend(connections_map_4);


    let mut nodes5 = objects::build_circle(8, spacing1, -0.71, -0.57, 5.0, damping / 2.0, 5);
    let connections_map_5 = objects::build_connections_map(&nodes5, spacing1 * 1.5, 50.0, nodes.len());
    nodes.append(&mut nodes5);
    connections_map.extend(connections_map_5);

    Scene {
        nodes: nodes,
        connections: connections_map,
        object_repulsion_dx: 0.01,
        object_repulsion_v0: 100.0,
    }
}
