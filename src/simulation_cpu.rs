use std::collections::HashMap;

use crate::{node::Node, simulation_general};
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
        let m_i = nodes[i].mass;
        let m_j = nodes[j].mass;

        let c = (dx / l).powi(7) - (dx / l).powi(13);
        let v = dir.normalize() * 3.0 * (v0 / dx) * c;

        nodes[i].current_acceleration += v / m_i;
        nodes[j].current_acceleration -= v / m_j;
    });
}

fn lennard_jones_connections_multithreaded(
    nodes: &mut Vec<Node>,
    connections_structure: &Vec<Vec<(usize, f32, f32)>>,
) {
    
    let nodes_copy = nodes.clone();
    
    nodes.par_iter_mut().enumerate().for_each(|(i, n)| {
        connections_structure[i].iter().for_each(|(j, dx, v0)| {
            let dir = nodes_copy[*j].position - n.position;
            let l = dir.length();
    
            let c = (dx / l).powi(7) - (dx / l).powi(13);
            let v = dir.normalize() * 3.0 * (v0 / dx) * c;
    
            n.current_acceleration += v / n.mass;
        });
    });
}

fn lennard_jones_repulsion(nodes: &mut Vec<Node>, objects_interactions: &HashMap<u32, Vec<usize>>) {
    let v0 = simulation_general::object_repulsion_v0;
    let dx = simulation_general::object_repulsion_dx;

    let objects: Vec<u32> = objects_interactions.keys().copied().collect();

    for obj_i in 0..objects.len() {
        for obj_j in obj_i+1..objects.len() {
            objects_interactions[&objects[obj_i]].iter().for_each(|i| {
                let a = *i;
                objects_interactions[&objects[obj_j]].iter().for_each(|j| {
                    let b = *j;
                    let dir = nodes[b].position - nodes[a].position;

                    let l = dir.length();
                    let m_a = nodes[a].mass;
                    let m_b = nodes[b].mass;
    
                    let c = (dx / l).powi(13);
                    let v = dir.normalize() * 3.0 * (v0 / dx) * c;
    
                    nodes[a].current_acceleration -= v / m_a;
                    nodes[b].current_acceleration += v / m_b;
                });
            });
        }
    }
}

fn wall_repulsion_force_y(nodes: &mut Vec<Node>) {
    let v0 = simulation_general::wall_repulsion_v0;
    let dx = simulation_general::wall_repulsion_dx;

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
        n.current_acceleration -= n.velocity * n.drag;
    });
}

pub fn simulate_single_thread_cpu(
    dt: f32,
    nodes: &mut Vec<Node>,
    connections: &HashMap<(usize, usize), (f32, f32)>,
    objects_interactions: &HashMap<u32, Vec<usize>>
) {
    start_integrate_velocity_verlet(dt, nodes);

    gravity_force(nodes);

    lennard_jones_connections(nodes, connections);
    lennard_jones_repulsion(nodes, objects_interactions);

    wall_repulsion_force_y(nodes);
    drag_force(nodes);

    end_integrate_velocity_verlet(dt, nodes);
}

pub fn simulate_multi_thread_cpu(
    dt: f32,
    nodes: &mut Vec<Node>,
    connections_structure: &Vec<Vec<(usize, f32, f32)>>,
    objects_interactions: &HashMap<u32, Vec<usize>>
) {
    start_integrate_velocity_verlet(dt, nodes);

    gravity_force(nodes);

    lennard_jones_connections_multithreaded(nodes, connections_structure);
    lennard_jones_repulsion(nodes, objects_interactions);

    wall_repulsion_force_y(nodes);
    drag_force(nodes);

    end_integrate_velocity_verlet(dt, nodes);
}