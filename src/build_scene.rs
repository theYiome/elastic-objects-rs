use glam::Vec2;
use crate::elastic_node::{Node};

pub fn build_object(size_x: usize, size_y: usize, spacing: f32, offset_x: f32, offset_y: f32) -> Vec<Node> {
    let mut object = Vec::with_capacity(size_x * size_y);
    for y in 0..size_y {
        for x in 0..size_x {
            object.push(Node {
                position: Vec2::new(offset_x + (x as f32) * spacing, offset_y + (y as f32) * spacing),
                velocity: Vec2::new(0.0, 0.0),
                mass: 1.0
            });
        }
    }
    return object;
}

pub fn build_connections(object: &Vec<Node>, search_distance: f32) -> Vec<Vec<usize>> {
    let mut connections: Vec<Vec<usize>> = Vec::new();
    for i in 0..object.len() {

        let mut row: Vec<usize> = Vec::new();

        for j in 0..object.len() {
            if i == j { continue };

            if Node::distance(&object[i], &object[j]) < search_distance {
                row.push(j);
            }
        }

        connections.push(row);
    }
    return connections;
}

pub fn simulate(dt: f32, object: &mut Vec<Node>, connections: &Vec<Vec<usize>>) {

    for (i, list) in connections.iter().enumerate() {
        for j in list {
            let dir = object[*j].position - object[i].position;
            let r = dir.length();
            let d = 0.22;
            let f = (d - r) * (d - r) * 0.1 * {if d > r {-1.0} else {1.0}};
            let m = object[i].mass;
            object[i].velocity += dir.normalize() * f / m;
        }
    }

    for i in 0..object.len() {
        for j in i+1..object.len() {
            let dir = object[j].position - object[i].position;
            let r = dir.length();
            let f = 0.00002 / (r * r);
            object[i].velocity -= dir.normalize() * f;
            object[j].velocity += dir.normalize() * f;
        }
    }

    for i in 0..object.len() {
        object[i].velocity *= 1.0 - dt;
    }

    for n in object {
        n.position += n.velocity * dt;
    }
}