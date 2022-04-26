use crate::{build_scene, node::Node};
use std::collections::HashMap;
use std::vec::Vec;
use rayon::collections::binary_heap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
struct Scene {
    nodes: Vec<Node>,
    connections: HashMap<(usize, usize), (f32, f32)>
}

pub fn standard_scene() -> (Vec<Node>, HashMap<(usize, usize), (f32, f32)>) {
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
    let mut nodes1 = build_scene::build_rectangle(object1_sx, object1_sy, spacing1, -0.92, -0.925, 0.35, 0.8, 1);
    let connections_map_1 = build_scene::build_connections_map(&nodes1, spacing1 * 1.5, 100.0, 0);

    // let mut nodes2 = build_scene::build_rectangle(object2_sx, object2_sy, spacing2, -0.3, 0.4, object2_m, 0.2, 2);
    let mut nodes2 = build_scene::build_circle(15, spacing2, -0.12, 0.6, 30.0, 0.2, 2);
    let connections_map_2 = build_scene::build_connections_map(&nodes2, spacing2 * 1.5, 400.0, nodes1.len());

    let mut connections_map: HashMap<(usize, usize), (f32, f32)> = HashMap::new();
    connections_map.extend(connections_map_1);
    connections_map.extend(connections_map_2);

    let mut nodes = Vec::new();
    nodes.append(&mut nodes1);
    nodes.append(&mut nodes2);

    let scene = Scene {
        nodes: nodes.clone(),
        connections: connections_map.clone()
    };

    let f = std::fs::File::create("scenes/default.bincode").unwrap();
    bincode::serialize_into(f, &scene).unwrap();

    let f2 = std::fs::File::open("scenes/default.bincode").unwrap();
    let decoded: Scene = bincode::deserialize_from(f2).unwrap();
    // let encoded: Vec<u8> = bincode::serialize(&scene).unwrap();
    // // 8 bytes for the length of the vector, 4 bytes per float.
    // // assert_eq!(encoded.len(), 8 + 4 * 4);
    // let mut decoded: Scene = bincode::deserialize(&encoded[..]).unwrap();
    // // decoded.nodes[0].position.x = 0.0;
    assert_eq!(scene, decoded);

    (nodes, connections_map)
}


pub fn performance_test_scene(object_size: usize) -> (Vec<Node>, HashMap<(usize, usize), (f32, f32)>, Vec<Vec<usize>>) {
    let spacing = 0.6 / object_size as f32;

    let mut objects: Vec<Vec<usize>> = Vec::new();
    let mut nodes = build_scene::build_rectangle(object_size, object_size, spacing, -0.5, -0.7, 1.0, 0.0, 1);

    let end_of_first = object_size * object_size;
    let end_of_second = object_size * object_size * 2;

    {
        let mut obj: Vec<usize> = Vec::new();
        for i in 0..end_of_first {
            obj.push(i);
        }
        objects.push(obj);
    }

    {
        let mut nodes2 = build_scene::build_rectangle(object_size, object_size, spacing, -0.4, 0.2, 1.0, 0.0, 2);
        nodes.append(&mut nodes2);
        {
            let mut obj: Vec<usize> = Vec::new();
            for i in end_of_first..end_of_second {
                obj.push(i);
            }
            objects.push(obj);
        }
    }

    let connections_map = build_scene::build_connections_map(&nodes, spacing * 1.1, 100.0, 0);

    (nodes, connections_map, objects)
}


pub fn performance_test_scene_optimized(object_size: usize) -> (Vec<Node>, HashMap<(usize, usize), (f32, f32)>, Vec<Vec<usize>>) {
    let spacing = 0.6 / object_size as f32;

    let mut objects: Vec<Vec<usize>> = Vec::new();

    let mut nodes = build_scene::build_rectangle(object_size, object_size, spacing, -0.5, -0.7, 1.0, 0.0, 1);
    objects.push(build_scene::get_boundary_nodes(&nodes, spacing * 1.1, 0));

    let end_of_first = object_size * object_size;
    // let end_of_second = object_size * object_size * 2;
    println!(
        "{} -> {}",
        object_size,
        build_scene::get_boundary_nodes(&nodes, spacing * 1.1, 0).len()
    );

    {
        let mut nodes2 = build_scene::build_rectangle(object_size, object_size, spacing, -0.4, 0.2, 1.0, 0.0, 2);
        objects.push(build_scene::get_boundary_nodes(&nodes2, spacing * 1.1, end_of_first));
        nodes.append(&mut nodes2);
    }

    let connections_map = build_scene::build_connections_map(&nodes, spacing * 1.1, 100.0, 0);
    
    (nodes, connections_map, objects)
}