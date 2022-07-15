use glam::Vec2;
use crate::scene::Scene;

use super::node::Node;

fn force_derivative_near_node(nodes: &[Node], connections_structure: &[(usize, f32, f32)], point: Vec2) -> Vec2 {
    connections_structure.iter().fold(Vec2::new(0.0, 0.0), |accum, &(i, dx, v0)| {
        let dir = nodes[i].position - point;
        let l = dir.length();
        let c = -7.0 * (dx / l).powi(8) + 13.0 * (dx / l).powi(14);
        let v = dir.normalize() * 3.0 * (v0 / (dx * dx)) * c;
        accum + v
    })
}

fn force_derivative_near_node2(nodes: &[Node], connections_structure: &[(usize, f32, f32)], point: Vec2) -> Vec2 {
    connections_structure.iter().fold(Vec2::new(0.0, 0.0), |accum, &(i, dx, v0)| {
        let dir = nodes[i].position - point;
        let l = dir.length();
        let c = (dx / l).powi(7) + (dx / l).powi(13);
        let v = dir.normalize() * 3.0 * (v0 / dx) * c;
        accum + v
    })
}

pub fn pressure_per_node(
    nodes: &[Node],
    connections_structure: &[Vec<(usize, f32, f32)>]
) -> Vec<f32> {

    let dx = 0.0005;
    nodes.iter().enumerate().map(|(index, n)| {
        let top = force_derivative_near_node2(nodes, &connections_structure[index], n.position + Vec2::new(0.0, dx)).y;
        let bottom = force_derivative_near_node2(nodes, &connections_structure[index], n.position + Vec2::new(0.0, -dx)).y;
        let right = force_derivative_near_node2(nodes, &connections_structure[index], n.position + Vec2::new(dx, 0.0)).x;
        let left = force_derivative_near_node2(nodes, &connections_structure[index], n.position + Vec2::new(-dx, 0.0)).x;
        let pressure = -0.25 * (-(right - left) - (top - bottom));
        pressure
    }).collect()
}

pub fn max_pressure(
    nodes: &[Node],
    connections_structure: &[Vec<(usize, f32, f32)>]
) -> f32 {
    pressure_per_node(nodes, connections_structure).iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b))
}