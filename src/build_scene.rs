use crate::elastic_node::Node;
use glam::Vec2;

pub fn build_nodes(
    size_x: usize,
    size_y: usize,
    spacing: f32,
    offset_x: f32,
    offset_y: f32,
) -> Vec<Node> {
    let mut nodes = Vec::with_capacity(size_x * size_y);
    for y in 0..size_y {
        for x in 0..size_x {
            nodes.push(Node {
                position: Vec2::new(
                    offset_x + (x as f32) * spacing,
                    offset_y + (y as f32) * spacing,
                ),
                velocity: Vec2::new(0.0, 0.0),
                current_acceleration: Vec2::new(0.0, 0.0),
                last_acceleration: Vec2::new(0.0, 0.0),
                mass: 1.0,
            });
        }
    }
    return nodes;
}

use std::collections::HashMap;

pub fn build_connections_2(
    nodes: &Vec<Node>,
    search_distance: f32,
) -> HashMap<(usize, usize), f32> {
    let mut connections: Vec<Vec<(usize, f32)>> = Vec::new();
    for i in 0..nodes.len() {
        let mut row: Vec<(usize, f32)> = Vec::new();

        for j in 0..nodes.len() {
            if i == j {
                continue;
            };

            let dist = Node::distance(&nodes[i], &nodes[j]);
            if dist < search_distance {
                row.push((j, dist));
            }
        }

        connections.push(row);
    }

    let mut connections_map = HashMap::new();

    connections.iter().enumerate().for_each(|(i, arr)| {
        arr.iter().for_each(|(j, dx)| {
            let a = if i > *j { *j } else { i };
            let b = if i > *j { i } else { *j };
            connections_map.entry((a, b)).or_insert(*dx);
        });
    });

    connections_map
}