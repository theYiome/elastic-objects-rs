use crate::{build_scene, node::Node};
use std::collections::HashMap;
use std::vec::Vec;

pub fn standard_scene() -> (Vec<Node>, HashMap<(usize, usize), (f32, f32)>, Vec<Vec<usize>>) {
    let mut objects: Vec<Vec<usize>> = Vec::new();

    let object1_sx = 24;
    let object1_sy = 12;
    let object1_st = object1_sx * object1_sy;
    let spacing1 = 0.08;

    let object2_sx = 4;
    let object2_sy = 4;
    let object2_st = object2_sx * object2_sy;
    let object2_m = 15.0;
    let spacing2 = 0.075;

    let mut nodes1 = build_scene::build_rectangle(object1_sx, object1_sy, spacing1, -0.92, -0.925, 1.0, 5.0);
    let mut connections_map_1 = build_scene::build_connections_map(&nodes1, spacing1 * 1.5, 70.0, 0);
    {
        let mut obj: Vec<usize> = Vec::new();
        for i in 0..object1_st {
            obj.push(i);
        }
        objects.push(obj);
    }

    let mut nodes2 = build_scene::build_circle(4, spacing2, -0.12, 0.8, 8.0, 0.0);
        // build_scene::build_rectangle(object2_sx, object2_sy, spacing2, -0.12, 0.8, object2_m, 1.0);
    
    let connections_map_2 = build_scene::build_connections_map(&nodes2, spacing2 * 1.5, 500.0, object1_st);
    {
        let mut obj: Vec<usize> = Vec::new();
        for i in object1_st..object1_st + nodes2.len() {
            obj.push(i);
        }
        objects.push(obj);
    }

    let mut connections_map: HashMap<(usize, usize), (f32, f32)> = HashMap::new();
    connections_map.extend(connections_map_1);
    connections_map.extend(connections_map_2);

    let mut nodes = Vec::new();
    nodes.append(&mut nodes1);
    nodes.append(&mut nodes2);

    (nodes, connections_map, objects)
}


pub fn performance_test_scene(object_size: usize) -> (Vec<Node>, HashMap<(usize, usize), (f32, f32)>, Vec<Vec<usize>>) {
    let spacing = 0.6 / object_size as f32;

    let mut objects: Vec<Vec<usize>> = Vec::new();
    let mut nodes = build_scene::build_rectangle(object_size, object_size, spacing, -0.5, -0.7, 1.0, 0.0);

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
        let mut nodes2 = build_scene::build_rectangle(object_size, object_size, spacing, -0.4, 0.2, 1.0, 0.0);
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

    let mut nodes = build_scene::build_rectangle(object_size, object_size, spacing, -0.5, -0.7, 1.0, 0.0);
    objects.push(build_scene::get_boundary_nodes(&nodes, spacing * 1.1, 0));

    let end_of_first = object_size * object_size;
    let end_of_second = object_size * object_size * 2;
    println!(
        "{} -> {}",
        object_size,
        build_scene::get_boundary_nodes(&nodes, spacing * 1.1, 0).len()
    );

    {
        let mut nodes2 = build_scene::build_rectangle(object_size, object_size, spacing, -0.4, 0.2, 1.0, 0.0);
        objects.push(build_scene::get_boundary_nodes(&nodes2, spacing * 1.1, end_of_first));
        nodes.append(&mut nodes2);
    }

    let connections_map = build_scene::build_connections_map(&nodes, spacing * 1.1, 100.0, 0);
    
    (nodes, connections_map, objects)
}