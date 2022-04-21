use crate::node::{Node};
use glam::Vec2;

pub fn build_rectangle(
    size_x: usize,
    size_y: usize,
    spacing: f32,
    offset_x: f32,
    offset_y: f32,
    mass: f32,
    damping: f32,
    object_id: u32
) -> Vec<Node> {
    let mut nodes = Vec::with_capacity(size_x * size_y);
    for y in 0..size_y {
        for x in 0..size_x {

            let is_boundary = if y == size_y - 1 || y == 0 || x == size_x - 1 || x == 0 { true } else { false };

            nodes.push(Node {
                position: Vec2::new(
                    offset_x + (x as f32) * spacing,
                    offset_y + (y as f32) * spacing,
                ),
                velocity: Vec2::new(0.0, 0.0),
                current_acceleration: Vec2::new(0.0, 0.0),
                last_acceleration: Vec2::new(0.0, 0.0),
                mass: mass,
                drag: damping,
                object_id: object_id,
                is_boundary: is_boundary
            });
        }
    }
    return nodes;
}

pub fn build_circle(
    layers: usize,
    spacing: f32,
    offset_x: f32,
    offset_y: f32,
    mass: f32,
    damping: f32,
    object_id: u32
) -> Vec<Node> {
    let mut nodes = Vec::new();
    for layer in 0..layers {
        let r = spacing * layer as f32;
        let even_node_count = (2.0 * PI * r / spacing) as usize;
        let nodes_in_layer = if even_node_count > 0 {even_node_count} else {1};

        let is_boundary = if layer == layers - 1 { true } else { false };

        for n in 0..nodes_in_layer {
            let angle = ((2.0 * PI) / nodes_in_layer as f32) * n as f32;
            nodes.push(Node {
                position: Vec2::new(offset_x + angle.cos() * r, offset_y + angle.sin() * r),
                velocity: Vec2::new(0.0, 0.0),
                current_acceleration: Vec2::new(0.0, 0.0),
                last_acceleration: Vec2::new(0.0, 0.0),
                mass: mass,
                drag: damping,
                object_id: object_id,
                is_boundary: is_boundary
            });
        }
    }
    return nodes;
}

use std::{collections::HashMap, f32::consts::PI};

pub fn build_connections_map(
    nodes: &Vec<Node>,
    search_distance: f32,
    v0: f32,
    offset: usize,
) -> HashMap<(usize, usize), (f32, f32)> {
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
            connections_map
                .entry((a + offset, b + offset))
                .or_insert((*dx, v0));
        });
    });

    connections_map
}

pub fn count_neighbours(
    connections_map: HashMap<(usize, usize), f32>,
    node_count: usize,
) -> Vec<usize> {
    let mut counts: Vec<usize> = Vec::new();
    counts.resize_with(node_count, || 0);
    connections_map.keys().for_each(|(i, j)| {
        counts[*i] += 1;
        counts[*j] += 1;
    });

    counts
}

pub fn get_boundary_nodes(
    nodes: &Vec<Node>,
    search_distance: f32,
    offset: usize,
) -> Vec<usize> {
    let mut counts: Vec<usize> = Vec::new();
    counts.resize_with(nodes.len(), || 0);

    for i in 0..nodes.len() {
        for j in 0..nodes.len() {
            if i == j {
                continue;
            };

            if Node::distance(&nodes[i], &nodes[j]) < search_distance {
                counts[i] += 1;
            }
        }
    }

    let mut bonudary_nodes: Vec<usize> = Vec::new();

    for i in 0..nodes.len() {
        if counts[i] < 4 {
            bonudary_nodes.push(i + offset)
        }
    }

    bonudary_nodes
}
