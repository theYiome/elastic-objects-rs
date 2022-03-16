use std::collections::HashMap;

use crate::node::{Node, self};

pub const object_repulsion_v0: f32 = 20.0;
pub const object_repulsion_dx: f32 = 0.01;

pub const wall_repulsion_v0: f32 = 200.0;
pub const wall_repulsion_dx: f32 = 0.05;

fn nodes_too_far(nodes: &mut Vec<Node>, connections: &mut HashMap<(usize, usize), (f32, f32)>) -> Vec<(usize, usize)> {

    let mut to_remove: Vec<(usize, usize)> = Vec::new();

    for (k, v) in connections.iter() {

        let (i, j) = *k;
        let (dx, _v0) = *v;

        let dir = nodes[j].position - nodes[i].position;
        let l = dir.length();
        if l > dx * 1.5 {
            to_remove.push(*k);
        }
    }

    to_remove
}

pub fn handle_connection_break(
    nodes: &mut Vec<Node>,
    connections: &mut HashMap<(usize, usize), (f32, f32)>,
) -> Option<HashMap<u32, Vec<usize>>> {

    let connections_to_break = nodes_too_far(nodes, connections);
    let recalculate_objects_interactions = connections_to_break.len() > 0;

    for k in connections_to_break {
        connections.iter().filter(|(&i, _j)| {
            i.0 == k.0 || i.0 == k.1 || i.1 == k.0 || i.1 == k.1
        }).for_each(|(i, _j)| {
            nodes[i.0].is_boundary = true;
            nodes[i.1].is_boundary = true;
        });
        connections.remove(&k);
    }

    if recalculate_objects_interactions {
        return Some(calculate_objects_interactions_structure(nodes));
    }

    return None;
}

pub fn calculate_objects_interactions_structure(nodes: &mut Vec<Node>) -> HashMap<u32, Vec<usize>> {
    let mut objects_interactions_structure: HashMap<u32, Vec<usize>> = HashMap::new();
    nodes.iter().enumerate().for_each(|(index, n)| {
        if n.is_boundary {
            let obj = objects_interactions_structure.get_mut(&n.object_id);
            match obj {
                Some(x) => {
                    x.push(index);
                }
                None => {
                    objects_interactions_structure.insert(n.object_id, vec![index]);
                }
            }
        }
    });

    objects_interactions_structure
}

pub fn calculate_connections_structure(connections_map: &HashMap<(usize, usize), (f32, f32)>, nodes: &Vec<Node>) -> Vec<Vec<(usize, f32, f32)>> {
    let mut connections_structure: Vec<Vec<(usize, f32, f32)>> = vec![Vec::new(); nodes.len()];
    connections_map.iter().for_each(|(k, v)| {
        connections_structure[k.0].push((k.1, v.0, v.1));
        connections_structure[k.1].push((k.0, v.0, v.1));
    });
    connections_structure
}