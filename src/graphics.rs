use std::{collections::HashMap, f32::consts::PI};

use crate::node::Node;
use crate::{graphics, simulation_general};
use glam::Vec2;
use rayon::prelude::*;

#[derive(Copy, Clone)]
pub struct Vertex {
    local_position: [f32; 2],
}
glium::implement_vertex!(Vertex, local_position);

#[derive(Copy, Clone)]
pub struct InstanceAttribute {
    position: [f32; 2],
    scale_x: f32,
    scale_y: f32,
    rotation: f32,
    color: [f32; 3],
}
glium::implement_vertex!(
    InstanceAttribute,
    position,
    scale_x,
    scale_y,
    rotation,
    color
);

/// Adds verticies and indices representing circle shape to existing conainer.
/// `radius` must be greater than `0.0`
///
/// `nr_of_triangles` specifies accuracy of the circle, must be at least 3 or higher.
///
/// 3 => isosceles triangle
///
/// 4 => `PI/2` tilted square
///
/// 5 => pentagon
///
/// 6 => hexagon
///
pub fn disk_mesh(nr_of_triangles: u16) -> (Vec<Vertex>, Vec<u16>) {
    assert!(nr_of_triangles > 2);

    let mut verticies: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    let delta_angle = (2.0 * PI) / nr_of_triangles as f32;

    verticies.push(Vertex {
        local_position: [0.0, 0.0],
    });
    verticies.push(Vertex {
        local_position: [0.0, 1.0],
    });

    for i in 2..nr_of_triangles + 1 {
        let angle = delta_angle * (i - 1) as f32;
        let x = angle.sin();
        let y = angle.cos();
        verticies.push(Vertex {
            local_position: [x, y],
        });
        indices.push(0);
        indices.push(i - 1);
        indices.push(i);
    }

    indices.push(0);
    indices.push(nr_of_triangles);
    indices.push(1);

    (verticies, indices)
}

pub fn square_mesh() -> (Vec<Vertex>, Vec<u16>) {
    let mut verticies: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    verticies.push(Vertex {
        local_position: [-0.5, 0.5],
    });
    verticies.push(Vertex {
        local_position: [-0.5, -0.5],
    });
    verticies.push(Vertex {
        local_position: [0.5, -0.5],
    });
    verticies.push(Vertex {
        local_position: [0.5, 0.5],
    });

    indices.push(0);
    indices.push(1);
    indices.push(2);

    indices.push(0);
    indices.push(3);
    indices.push(2);

    (verticies, indices)
}

pub fn radius_from_area(area: f32) -> f32 {
    (area / PI).sqrt()
}

fn calculate_temperatue(
    nodes: &Vec<Node>,
    connections: &HashMap<(usize, usize), (f32, f32)>,
    objects_interactions: &HashMap<u32, Vec<usize>>
) -> Vec<f32> {
    let mut forces: Vec<Vec2> = vec![Vec2::new(0.0, 0.0); nodes.len()];

    connections.keys().for_each(|(a, b)| {
        let i = *a;
        let j = *b;
        let (dx, v0) = *connections.get(&(i, j)).unwrap();

        let dir = nodes[j].position - nodes[i].position;
        let l = dir.length();

        let c = (dx / l).powi(7) - (dx / l).powi(13);
        let v = dir.normalize() * 3.0 * (v0 / dx) * c;

        forces[i] += v;
        forces[j] -= v;
    });

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
                    let c = (dx / l).powi(13);
                    let v = dir.normalize() * 3.0 * (v0 / dx) * c;
    
                    forces[a] += v;
                    forces[b] -= v;
                });
            });
        }
    }

    let v0 = simulation_general::wall_repulsion_v0;
    let dx = simulation_general::wall_repulsion_dx;

    nodes.iter().enumerate().for_each(|(index, n)| {
        let dir = glam::vec2(n.position.x, -1.0) - n.position;
        let l = dir.length();

        let c = (dx / l).powi(13);
        let v = dir.normalize() * 3.0 * (v0 / dx) * c;

        forces[index] -= v;
    });

    // nodes.iter().enumerate().for_each(|(index, n)| {
    //     forces[index] -= n.velocity * n.drag;
    //     forces[index].y += -9.81;
    // });

    forces
        .iter()
        .enumerate()
        .map(|(i, f)| f.dot(nodes[i].position))
        .collect()
}

#[derive(PartialEq)]
pub enum ColoringMode {
    KineticEnergy,
    Temperature,
    Boundary
}

pub fn draw_disks(
    nodes: &Vec<Node>,
    connections: &HashMap<(usize, usize), (f32, f32)>,
    objects_interactions: &HashMap<u32, Vec<usize>>,
    coloring_mode: &ColoringMode,
    dt: f32
) -> Vec<InstanceAttribute> {

    // let colors = color_from_kinetic_energy(nodes);
    let colors = match coloring_mode {
        ColoringMode::KineticEnergy => color_from_kinetic_energy(nodes),
        ColoringMode::Temperature => color_from_temperature(nodes, connections, objects_interactions, dt),
        ColoringMode::Boundary => nodes.iter().map(|n| if n.is_boundary { [0.0, 0.0, 0.0] } else { [0.3, 0.3, 0.3] } ).collect()
    };

    nodes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let radius = 0.0045 + radius_from_area(n.mass) * 0.000;

            InstanceAttribute {
                position: n.position.to_array(),
                scale_x: radius,
                scale_y: radius,
                rotation: 0.0,
                color: colors[i],
            }
        })
        .collect()
}

fn color_from_temperature(
    nodes: &Vec<Node>,
    connections: &HashMap<(usize, usize), (f32, f32)>,
    objects_interactions: &HashMap<u32, Vec<usize>>,
    dt: f32,
) -> Vec<[f32; 3]> {
    const TEMPERATURE_CACHE_SIZE: usize = 200;
    static mut TEMPERATURE_CACHE: Vec<Vec<f32>> = Vec::new();
    static mut DT_CACHE: Vec<f32> = Vec::new();
    static mut CURRENT_RECORD: usize = 0;
    unsafe {
        TEMPERATURE_CACHE.resize(nodes.len(), vec![0.0; TEMPERATURE_CACHE_SIZE]);
        TEMPERATURE_CACHE
            .iter_mut()
            .for_each(|cache| cache.resize(TEMPERATURE_CACHE_SIZE, 0.0));
        DT_CACHE.resize(TEMPERATURE_CACHE_SIZE, 0.0);

        if dt > 0.0 {
            CURRENT_RECORD = (CURRENT_RECORD + 1) % TEMPERATURE_CACHE_SIZE;
            let current_temperature = calculate_temperatue(nodes, connections, objects_interactions);
            TEMPERATURE_CACHE
                .iter_mut()
                .enumerate()
                .for_each(|(node_index, cache)| {
                    cache[CURRENT_RECORD] = current_temperature[node_index];
                });
            DT_CACHE[CURRENT_RECORD] = dt;
        }
    }

    let total_dt = unsafe {
        let dt_sum = DT_CACHE.iter().copied().sum::<f32>();
        if dt_sum > 0.0 {
            dt_sum
        } else {
            f32::INFINITY
        }
    };

    nodes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let color: f32 = unsafe { -0.5 * TEMPERATURE_CACHE[i].iter().copied().sum::<f32>() / total_dt };
            number_to_rgb(color, -1000.0, 8000.0)
        })
        .collect()
}

fn color_from_kinetic_energy(
    nodes: &Vec<Node>,
) -> Vec<[f32; 3]> {

    nodes
        .iter()
        .map(|n| {
            [n.velocity.length_squared() * 0.5 * 0.7, 0.0, 0.0]
        })
        .collect()
}

//     for (k, v) in connections {
//         let (dx, v0) = *v;
//         let (a, b) = (nodes[k.0].position, nodes[k.1].position);
//         let color = ((a - b).length() - dx) * 20.0;
//         graphics::add_rectangle(
//             &mut verticies,
//             &mut indices,
//             a,
//             b,
//             0.007 + v0 * 0.00001,
//             [0.2 + color, 0.2 + color, 0.2 + color],
//         );
//     }

fn number_to_rgb(mut t: f32, min: f32, max: f32) -> [f32; 3] {
    assert!(max > min);

    t = if t < min {
        min
    } else {
        if t > max {
            max
        } else {
            t
        }
    };
    let n_t: f32 = (t - min) / (max - min);
    let regions: [f32; 3] = [1.0 / 4.0, (1.0 / 4.0) * 2.0, (1.0 / 4.0) * 3.0];

    return {
        if n_t <= regions[0] {
            [0.0, 4.0 * n_t, 1.0]
        } else if n_t > regions[0] && n_t <= regions[1] {
            [0.0, 1.0, 2.0 - 4.0 * n_t]
        } else if n_t > regions[1] && n_t <= regions[2] {
            [2.0 - 4.0 * (1.0 - n_t), 1.0, 0.0]
        } else {
            [1.0, 4.0 * (1.0 - n_t), 0.0]
        }
    };
}
