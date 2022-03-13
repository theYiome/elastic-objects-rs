use std::collections::HashMap;

use crate::node::Node;

pub const object_repulsion_v0: f32 = 20.0;
pub const object_repulsion_dx: f32 = 0.07;

pub const wall_repulsion_v0: f32 = 200.0;
pub const wall_repulsion_dx: f32 = 0.05;

fn nodes_too_far(nodes: &mut Vec<Node>, connections: &mut HashMap<(usize, usize), (f32, f32)>) -> Vec<(usize, usize)> {

    let mut to_remove: Vec<(usize, usize)> = Vec::new();

    for (k, v) in connections.iter() {

        let (i, j) = *k;
        let (dx, v0) = *v;

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
    objects: &mut Vec<Vec<usize>>,
    connections: &mut HashMap<(usize, usize), (f32, f32)>,
) {
    for k in nodes_too_far(nodes, connections) {
        connections.remove(&k);
    }
}
