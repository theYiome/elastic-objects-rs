use std::collections::HashMap;

use crate::node::{Node};

fn object_repulsion_energy(nodes: &[Node], objects: &Vec<Vec<usize>>) -> f32 {
    let mut total_object_repulsion_energy = 0.0;

    let length = objects.len();
    for i in 0..length {
        for j in i + 1..length {
            // calculate energy between each node in object "i" and object "j"
            for n_i in &objects[i] {
                for n_j in &objects[j] {
                    let a = *n_i;
                    let b = *n_j;

                    let v0 = 20.0;
                    let dx = 0.1;

                    let dist = (nodes[b].position - nodes[a].position).length();
                    let m01 = 1.12246204831;

                    let sigma = (1.0 / m01) * dx;
                    let inner = sigma / dist;

                    total_object_repulsion_energy += v0 * inner.powf(12.0);
                }
            }
        }
    }

    return total_object_repulsion_energy;
}

fn bond_energy(nodes: &[Node], connections: &HashMap<(usize, usize), (f32, f32)>) -> f32 {
    let mut total_bond_energy = 0.0;

    connections.keys().for_each(|(a, b)| {
        let n1 = &nodes[*a];
        let n2 = &nodes[*b];
        let dist = (n2.position - n1.position).length();
        let (dx, v0) = *connections.get(&(*a, *b)).unwrap();

        let m01 = 1.12246204831;
        let sigma = (1.0 / m01) * dx;
        let inner = sigma / dist;

        total_bond_energy += v0 * (inner.powf(12.0) - inner.powf(6.0));
    });

    return total_bond_energy;
}

fn wall_repulsion_energy(nodes: &[Node]) -> f32 {
    let mut total_wall_repulsion_energy = 0.0;

    nodes.iter().for_each(|n| {
        let v0 = 200.0;
        let dx = 0.05;

        let dist = (-1.0 - n.position.y).abs();
        let m01 = 1.12246204831;
        let sigma = (1.0 / m01) * dx;
        let inner = sigma / dist;

        total_wall_repulsion_energy += v0 * inner.powf(12.0);
    });

    return total_wall_repulsion_energy;
}

fn gravity_energy(nodes: &[Node]) -> f32 {
    let mut total_gravity_energy = 0.0;
    nodes.iter().enumerate().for_each(|(i, n1)| {
        total_gravity_energy += n1.mass * 9.81 * (n1.position.y + 0.5);
    });
    return total_gravity_energy;
}

fn kinetic_energy(nodes: &[Node]) -> f32 {
    let mut total_kinetic_energy = 0.0;
    nodes.iter().enumerate().for_each(|(i, n1)| {
        total_kinetic_energy += n1.velocity.length_squared() * n1.mass * 0.5;
    });
    return total_kinetic_energy;
}

pub fn calculate_total_energy(
    nodes: &[Node],
    connections: &HashMap<(usize, usize), (f32, f32)>,
    objects: &Vec<Vec<usize>>,
) -> (f32, f32, f32, f32, f32) {
    let total_kinetic: f32 = kinetic_energy(nodes);
    let total_gravity: f32 = gravity_energy(nodes);
    let total_lennjon: f32 = bond_energy(nodes, connections);
    let total_wallrep: f32 = wall_repulsion_energy(nodes);
    let total_objrepu: f32 = object_repulsion_energy(nodes, objects);

    (
        total_kinetic,
        total_gravity,
        total_lennjon,
        total_wallrep,
        total_objrepu,
    )
}
