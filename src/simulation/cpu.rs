use std::collections::HashMap;

use crate::scene::{Scene, self};

use super::node::Node;

use glam::Vec2;
use rayon::prelude::*;

// https://en.wikipedia.org/wiki/Verlet_integration#Velocity_Verlet
pub fn start_integrate_velocity_verlet(dt: f32, nodes: &mut [Node]) {
    nodes.iter_mut().for_each(|n| {
        n.position += (n.velocity * dt) + (0.5 * n.current_acceleration * dt * dt);

        n.last_acceleration = n.current_acceleration;
        n.current_acceleration *= 0.0;
    });
}

pub fn end_integrate_velocity_verlet(dt: f32, nodes: &mut [Node]) {
    nodes.iter_mut().for_each(|n| {
        n.velocity += 0.5 * (n.last_acceleration + n.current_acceleration) * dt;
    });
}

// https://users.rust-lang.org/t/help-with-parallelizing-a-nested-loop/22568/2
fn lennard_jones_connections(
    nodes: &mut [Node],
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
    nodes: &mut [Node],
    connections_structure: &[Vec<(usize, f32, f32)>],
) {
    
    let acceleration_diff: Vec<Vec2> = nodes.par_iter().enumerate().map(|(i, n)| {
        connections_structure[i].iter().fold(Vec2::new(0.0, 0.0), |accum, (j, dx, v0)| {
            let dir = nodes[*j].position - n.position;
            let l = dir.length();
    
            let c = (dx / l).powi(7) - (dx / l).powi(13);
            accum + (dir.normalize() * 3.0 * (v0 / dx) * c / n.mass)
        })
    }).collect();
    
    
    nodes.iter_mut().enumerate().for_each(|(i, n)| {
        n.current_acceleration += acceleration_diff[i];
    });
}

fn lennard_jones_repulsion(scene: &mut Scene, objects_interactions: &HashMap<u32, Vec<usize>>) {
    let v0 = scene.object_repulsion_v0;
    let dx = scene.object_repulsion_dx;

    let objects: Vec<u32> = objects_interactions.keys().copied().collect();
    let nodes = &mut scene.nodes;

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

fn lennard_jones_repulsion_multithreaded_2(scene: &mut Scene, collisions_sturcture: &[Vec<usize>]) {
    let v0 = scene.object_repulsion_v0;
    let dx = scene.object_repulsion_dx;
    let nodes = &mut scene.nodes;

    let acceleration_diff: Vec<Vec2> = nodes.par_iter().enumerate().map(|(i, n)| {
        collisions_sturcture[i].iter().fold(Vec2::new(0.0, 0.0), |accum, j| {
            let dir = nodes[*j].position - n.position;
            let l = dir.length();
            let c = (dx / l).powi(13);
            accum + (dir.normalize() * 3.0 * (v0 / dx) * c / n.mass)
        })
    }).collect();

    nodes.iter_mut().enumerate().for_each(|(i, n)| {
        n.current_acceleration -= acceleration_diff[i];
    });
}

fn lennard_jones_repulsion_2(scene: &mut Scene, collisions_sturcture: &[Vec<usize>]) {
    let v0 = scene.object_repulsion_v0;
    let dx = scene.object_repulsion_dx;
    let nodes = &mut scene.nodes;

    let acceleration_diff: Vec<Vec2> = nodes.iter().enumerate().map(|(i, n)| {
        collisions_sturcture[i].iter().fold(Vec2::new(0.0, 0.0), |accum, j| {
            let dir = nodes[*j].position - n.position;
            let l = dir.length();
            let c = (dx / l).powi(13);
            accum + (dir.normalize() * 3.0 * (v0 / dx) * c / n.mass)
        })
    }).collect();

    nodes.iter_mut().enumerate().for_each(|(i, n)| {
        n.current_acceleration -= acceleration_diff[i];
    });
}

fn wall_repulsion_force_y(nodes: &mut [Node]) {
    let v0 = super::general::WALL_REPULSION_V0;
    let dx = super::general::WALL_REPULSION_DX;

    nodes.iter_mut().for_each(|n| {
        let dir = glam::vec2(n.position.x, -1.0) - n.position;
        let l = dir.length();
        let mi = n.mass;

        let c = (dx / l).powi(13);
        let v = dir.normalize() * 3.0 * (v0 / dx) * c;

        n.current_acceleration -= v / mi;
    });
}

fn gravity_force(nodes: &mut [Node]) {
    nodes.iter_mut().for_each(|n| {
        n.current_acceleration.y += super::general::GRAVITY_CONSTANT;
    });
}

fn drag_force(nodes: &mut [Node]) {
    nodes.iter_mut().for_each(|n| {
        n.current_acceleration -= n.velocity * n.velocity.length() * n.drag;
    });
}

pub fn simulate_single_thread_cpu(
    dt: f32,
    scene: &mut Scene,
    collisions_structure: &Vec<Vec<usize>>
) {

    start_integrate_velocity_verlet(dt, &mut scene.nodes);

    gravity_force(&mut scene.nodes);

    lennard_jones_connections(&mut scene.nodes, &scene.connections);
    lennard_jones_repulsion_2(scene, collisions_structure);

    wall_repulsion_force_y(&mut scene.nodes);
    drag_force(&mut scene.nodes);

    end_integrate_velocity_verlet(dt, &mut scene.nodes);
}

pub fn simulate_multi_thread_cpu(
    dt: f32,
    scene: &mut Scene,
    connections_structure: &[Vec<(usize, f32, f32)>],
    collisions_structure: &[Vec<usize>]
) {
    start_integrate_velocity_verlet(dt, &mut scene.nodes);

    gravity_force(&mut scene.nodes);

    lennard_jones_connections_multithreaded(&mut scene.nodes, connections_structure);
    lennard_jones_repulsion_multithreaded_2(scene, collisions_structure);
    
    wall_repulsion_force_y(&mut scene.nodes);
    drag_force(&mut scene.nodes);

    end_integrate_velocity_verlet(dt, &mut scene.nodes);
}

pub fn simulate_multi_thread_cpu_enchanced(
    dt: f32,
    scene: &mut Scene,
    connections_structure: &[Vec<(usize, f32, f32)>],
    collisions_structure: &[Vec<usize>]
) {
    let nodes = &mut scene.nodes;
    
    start_integrate_velocity_verlet(dt, nodes);

    let acceleration_diff: Vec<Vec2> = nodes.par_iter().enumerate().map(|(i, n)| {
        let connections: Vec2 = connections_structure[i].iter().fold(Vec2::new(0.0, 0.0), |accum, (j, dx, v0)| {
            let dir = nodes[*j].position - n.position;
            let l = dir.length();
            
            let c = (dx / l).powi(7) - (dx / l).powi(13);
            accum + (dir.normalize() * 3.0 * (v0 / dx) * c)
        });
        let repulsion: Vec2 = collisions_structure[i].iter().fold(Vec2::new(0.0, 0.0), |accum, j| {
            let V0: f32 = scene.object_repulsion_v0;
            let DX: f32 = scene.object_repulsion_dx;
            let dir = nodes[*j].position - n.position;
            let l = dir.length();
            let c = (DX / l).powi(13);
            accum + (dir.normalize() * 3.0 * (V0 / DX) * c)
        });

        let wall_repulsion: Vec2 = {
            const V0: f32 = super::general::WALL_REPULSION_V0;
            const DX: f32 = super::general::WALL_REPULSION_DX;
    
            let dir = glam::vec2(n.position.x, -1.0) - n.position;
            let l = dir.length();
    
            let c = (DX / l).powi(13);
            dir.normalize() * 3.0 * (V0 / DX) * c
        };

        let drag = n.velocity * n.velocity.length() * n.drag;
        
        let mut result = (connections - repulsion - wall_repulsion) / n.mass;
        result -= drag;
        //gravity
        result.y += super::general::GRAVITY_CONSTANT;
        

        result
    }).collect();

    nodes.iter_mut().enumerate().for_each(|(i, n)| {
        n.current_acceleration += acceleration_diff[i];
    });

    end_integrate_velocity_verlet(dt, nodes);
}