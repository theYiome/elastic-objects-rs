use std::collections::HashMap;
use std::vec::Vec;

use super::objects;
use super::Scene;

pub fn generate() -> Scene {
    let object1_sx = 180;
    let object1_sy = 90;
    // let object1_st = object1_sx * object1_sy;
    let spacing1 = 0.01;

    // let object2_sx = 40;
    // let object2_sy = 80;
    // let object2_st = object2_sx * object2_sy;
    // let object2_m = 15.0;
    let spacing2 = 0.01;

    // let mut nodes1 = build_scene::build_circle(8, spacing2, -0.12, -0.4, 30.0, 0.5, 1);
    let mut nodes1 = objects::build_rectangle(object1_sx, object1_sy, spacing1, -0.92, -0.925, 0.35, 0.8, 1);
    let connections_map_1 = objects::build_connections_map(&nodes1, spacing1 * 1.5, 100.0, 0);

    // let mut nodes2 = build_scene::build_rectangle(object2_sx, object2_sy, spacing2, -0.3, 0.4, object2_m, 0.2, 2);
    let mut nodes2 = objects::build_circle(15, spacing2, -0.12, 0.6, 30.0, 0.2, 2);
    let connections_map_2 = objects::build_connections_map(&nodes2, spacing2 * 1.5, 400.0, nodes1.len());

    let mut connections_map: HashMap<(usize, usize), (f32, f32)> = HashMap::new();
    connections_map.extend(connections_map_1);
    connections_map.extend(connections_map_2);

    let mut nodes = Vec::new();
    nodes.append(&mut nodes1);
    nodes.append(&mut nodes2);

    // let scene = Scene {
    //     nodes: nodes.clone(),
    //     connections: connections_map.clone()
    // };

    // let f = std::fs::File::create("scenes/default.bincode").unwrap();
    // bincode::serialize_into(f, &scene).unwrap();

    // let f2 = std::fs::File::open("scenes/default.bincode").unwrap();
    // let decoded: Scene = bincode::deserialize_from(f2).unwrap();
    // // let encoded: Vec<u8> = bincode::serialize(&scene).unwrap();
    // // // 8 bytes for the length of the vector, 4 bytes per float.
    // // // assert_eq!(encoded.len(), 8 + 4 * 4);
    // // let mut decoded: Scene = bincode::deserialize(&encoded[..]).unwrap();
    // // // decoded.nodes[0].position.x = 0.0;
    // assert_eq!(scene, decoded);

    // (nodes, connections_map)

    Scene {
        nodes: nodes,
        connections: connections_map
    }
}