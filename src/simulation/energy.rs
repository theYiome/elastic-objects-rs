use crate::{simulation::node::Node, scene::Scene};
use std::collections::HashMap;
use rayon::prelude::*;

const M_01: f32 = 1.12246204831;

fn object_repulsion_energy(scene: &Scene) -> f32 {
    let v0 = scene.object_repulsion_v0;
    let dx = scene.object_repulsion_dx;
    let sigma = (1.0 / M_01) * dx;

    scene.nodes.par_iter().enumerate().fold(||0.0, |acc_i, (i, node_i)| {
        let energy_for_node_i = scene.nodes.iter()
        .enumerate().filter(|(j, _node_j)| i >= *j)
        .fold(0.0, |acc_j, (_j, node_j)| {
            if node_i.object_id == node_j.object_id {
                // no repulsion between the same object nodes
                // takes care of counting node energy with itself
                acc_j
            } else {
                let dist = (node_j.position - node_i.position).length();
                let inner = sigma / dist;
                acc_j + v0 * inner.powf(12.0)
            }
        });

        acc_i + energy_for_node_i
    }).sum()
}

fn bond_energy(nodes: &[Node], connections: &HashMap<(usize, usize), (f32, f32)>) -> f32 {
    connections.keys().copied().fold(0.0, |acc, (a, b)| {
        let dist = (nodes[b].position - nodes[a].position).length();

        let (dx, v0) = *connections.get(&(a, b)).unwrap();
        let sigma = (1.0 / M_01) * dx;
        let inner = sigma / dist;

        acc + v0 * (inner.powf(12.0) - inner.powf(6.0))
    })
}

fn wall_repulsion_energy(nodes: &[Node]) -> f32 {
    let v0 = super::general::WALL_REPULSION_V0;
    let dx = super::general::WALL_REPULSION_DX;
    let m01 = 1.12246204831;
    let sigma = (1.0 / M_01) * dx;

    nodes.iter().fold(0.0, |acc, n| {
        let dist = (-1.0 - n.position.y).abs();
        let inner = sigma / dist;

        acc + v0 * inner.powf(12.0)
    })
}

fn gravity_energy(nodes: &[Node]) -> f32 {
    static GRAVITY_CONST: f32 = 9.81;
    nodes.iter().fold(0.0, |acc, n| {
        acc + GRAVITY_CONST * n.mass * (n.position.y + 0.5)
    })
}

fn kinetic_energy(nodes: &[Node]) -> f32 {
    nodes.iter().fold(0.0, |acc, n| {
        acc + n.velocity.length_squared() * n.mass * 0.5
    })
}

pub fn calculate_total_energy(scene: &Scene) -> (f32, f32, f32, f32, f32) {
    let total_kinetic: f32 = kinetic_energy(&scene.nodes);
    let total_gravity: f32 = gravity_energy(&scene.nodes);
    let total_lennjon: f32 = bond_energy(&scene.nodes, &scene.connections);
    let total_wallrep: f32 = wall_repulsion_energy(&scene.nodes);
    let total_objrepu: f32 = object_repulsion_energy(scene);

    (
        total_kinetic,
        total_gravity,
        total_lennjon,
        total_wallrep,
        total_objrepu,
    )
}
