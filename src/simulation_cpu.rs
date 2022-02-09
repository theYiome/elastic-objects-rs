use std::collections::HashMap;

use crate::node::Node;
use rayon::prelude::*;

// https://en.wikipedia.org/wiki/Verlet_integration#Velocity_Verlet
fn start_integrate_velocity_verlet(dt: f32, nodes: &mut Vec<Node>) {
    nodes.iter_mut().for_each(|n| {
        n.position += (n.velocity * dt) + (0.5 * n.current_acceleration * dt * dt);

        n.last_acceleration = n.current_acceleration;
        n.current_acceleration *= 0.0;
    });
}

fn end_integrate_velocity_verlet(dt: f32, nodes: &mut Vec<Node>) {
    nodes.iter_mut().for_each(|n| {
        n.velocity += 0.5 * (n.last_acceleration + n.current_acceleration) * dt;
    });
}

// https://users.rust-lang.org/t/help-with-parallelizing-a-nested-loop/22568/2
fn lennard_jones_connections(
    nodes: &mut Vec<Node>,
    connections: &HashMap<(usize, usize), (f32, f32)>,
) {
    connections.keys().for_each(|(a, b)| {
        let i = *a;
        let j = *b;
        let (dx, v0) = *connections.get(&(i, j)).unwrap();

        let dir = nodes[j].position - nodes[i].position;
        let l = dir.length();
        let m = nodes[i].mass;

        let c = (dx / l).powi(7) - (dx / l).powi(13);
        let v = dir.normalize() * 3.0 * (v0 / dx) * c;

        nodes[i].current_acceleration += v / m;
        nodes[j].current_acceleration -= v / m;
    });
}

fn lennard_jones_repulsion(nodes: &mut Vec<Node>, objects: &Vec<Vec<usize>>) {
    let v0 = 20.0;
    let dx = 0.07;

    let length = objects.len();

    for i in 0..length {
        for j in i + 1..length {
            // calculate forces between each node in object "i" and object "j"
            for n_i in &objects[i] {
                for n_j in &objects[j] {
                    let a = *n_i;
                    let b = *n_j;

                    let dir = nodes[b].position - nodes[a].position;

                    let l = dir.length();
                    let mi = nodes[a].mass;
                    let mj = nodes[b].mass;

                    let c = (dx / l).powi(13);
                    let v = dir.normalize() * 3.0 * (v0 / dx) * c;

                    nodes[a].current_acceleration -= v / mi;
                    nodes[b].current_acceleration += v / mj;
                }
            }
        }
    }
}

// fn repulsion_force_simple(nodes: &mut Vec<Node>) {
//     let v0 = 500.0;
//     let dx = 0.1;

//     for i in 0..nodes.len() {
//         for j in i + 1..nodes.len() {
//             let dir = nodes[j].position - nodes[i].position;
//             let l = dir.length();
//             let mi = nodes[i].mass;
//             let mj = nodes[j].mass;

//             let c = (dx / l).powi(13);
//             let v = dir.normalize() * 3.0 * (v0 / dx) * c;

//             nodes[i].current_acceleration -= v / mi;
//             nodes[j].current_acceleration += v / mj;
//         }
//     }
// }

// fn repulsion_force_iter(nodes: &mut Vec<Node>) {
//     let v0 = 500.0;
//     let dx = 0.1;

//     for i in 0..nodes.len() {
//         let (left, right) = nodes.split_at_mut(i + 1);
//         let mut node_i = &mut left[i];

//         right.iter_mut().for_each(|node_j| {
//             let dir = node_j.position - node_i.position;
//             let l = dir.length();
//             let mi = node_i.mass;
//             let mj = node_j.mass;

//             let c = (dx / l).powi(13);
//             let v = dir.normalize() * 3.0 * (v0 / dx) * c;

//             node_i.current_acceleration -= v / mi;
//             node_j.current_acceleration += v / mj;
//         });
//     }
// }

// fn repulsion_force_par_iter(nodes: &mut Vec<Node>) {
//     let v0 = 500.0;
//     let dx = 0.1;

//     let length = nodes.len();
//     let obj2 = nodes.clone();

//     nodes.par_iter_mut().enumerate().for_each(|(i, n)| {
//         (0..length).for_each(|j| {
//             if j != i {
//                 let dir = obj2[j].position - n.position;
//                 let l = dir.length();
//                 let mi = n.mass;
//                 // let mj = obj2[j].mass;

//                 let c = (dx / l).powi(13);
//                 let v = dir.normalize() * 3.0 * (v0 / dx) * c;

//                 n.current_acceleration -= v / mi;
//             }
//         });
//     });
// }

fn wall_repulsion_force_y(nodes: &mut Vec<Node>) {
    let v0 = 200.0;
    let dx = 0.05;

    nodes.iter_mut().for_each(|n| {
        let dir = glam::vec2(n.position.x, -1.0) - n.position;
        let l = dir.length();
        let mi = n.mass;

        let c = (dx / l).powi(13);
        let v = dir.normalize() * 3.0 * (v0 / dx) * c;

        n.current_acceleration -= v / mi;
    });
}

fn gravity_force(nodes: &mut Vec<Node>) {
    nodes.iter_mut().for_each(|n| {
        n.current_acceleration.y += -9.81;
    });
}

fn drag_force(nodes: &mut Vec<Node>) {
    nodes.iter_mut().for_each(|n| {
        n.current_acceleration -= n.velocity * n.damping;
    });
}

fn bond_break(nodes: &mut Vec<Node>, connections: &mut HashMap<(usize, usize), (f32, f32)>) -> Vec<(usize, usize)> {

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

pub fn simulate_single_thread_cpu(
    dt: f32,
    nodes: &mut Vec<Node>,
    objects: &mut Vec<Vec<usize>>,
    connections: &mut HashMap<(usize, usize), (f32, f32)>,
) {
    start_integrate_velocity_verlet(dt, nodes);

    gravity_force(nodes);

    lennard_jones_connections(nodes, connections);
    lennard_jones_repulsion(nodes, objects);

    
    wall_repulsion_force_y(nodes);
    drag_force(nodes);
    
    end_integrate_velocity_verlet(dt, nodes);

    for k in bond_break(nodes, connections) {
        connections.remove(&k);
    }
}
