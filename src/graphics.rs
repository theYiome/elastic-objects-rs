use std::{collections::HashMap, f32::consts::PI};

use crate::node::Node;
use crate::{simulation_general};
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

/// Adds vertices and indices representing circle shape to existing container.
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

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    let delta_angle = (2.0 * PI) / nr_of_triangles as f32;

    vertices.push(Vertex {
        local_position: [0.0, 0.0],
    });
    vertices.push(Vertex {
        local_position: [0.0, 1.0],
    });

    for i in 2..nr_of_triangles + 1 {
        let angle = delta_angle * (i - 1) as f32;
        let x = angle.sin();
        let y = angle.cos();
        vertices.push(Vertex {
            local_position: [x, y],
        });
        indices.push(0);
        indices.push(i - 1);
        indices.push(i);
    }

    indices.push(0);
    indices.push(nr_of_triangles);
    indices.push(1);

    (vertices, indices)
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

fn calculate_temperature(
    nodes: &[Node],
    connections_structure: &[Vec<(usize, f32, f32)>],
    // objects_interactions: &HashMap<u32, Vec<usize>>
) -> Vec<f32> {
    let mut forces: Vec<Vec2> = vec![Vec2::new(0.0, 0.0); nodes.len()];

    nodes.iter().enumerate().for_each(|(i, n)| {
        connections_structure[i].iter().for_each(|(j, dx, v0)| {
            let dir = nodes[*j].position - n.position;
            let l = dir.length();
    
            let c = (dx / l).powi(7) - (dx / l).powi(13);
            let v = dir.normalize() * 3.0 * (v0 / dx) * c;
    
            forces[i] += v;
        });
    });


    let v0 = simulation_general::WALL_REPULSION_V0;
    let dx = simulation_general::WALL_REPULSION_DX;

    nodes.iter().enumerate().for_each(|(index, n)| {
        let dir = glam::vec2(n.position.x, -1.0) - n.position;
        let l = dir.length();

        let c = (dx / l).powi(13);
        let v = dir.normalize() * 3.0 * (v0 / dx) * c;

        forces[index] -= v;
    });

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
    Boundary,
    Pressure
}

pub fn draw_disks(
    nodes: &[Node],
    connections_structure: &[Vec<(usize, f32, f32)>],
    objects_interactions: &HashMap<u32, Vec<usize>>,
    coloring_mode: &ColoringMode,
    dt: f32
) -> Vec<InstanceAttribute> {

    // let colors = color_from_kinetic_energy(nodes);
    let colors = match coloring_mode {
        ColoringMode::KineticEnergy => color_from_kinetic_energy(nodes),
        ColoringMode::Temperature => color_from_temperature(nodes, connections_structure, objects_interactions, dt),
        ColoringMode::Boundary => color_from_boundary(nodes),
        ColoringMode::Pressure => color_from_pressure(nodes, connections_structure)
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

fn color_from_boundary(nodes: &[Node]) -> Vec<[f32; 3]> {
    let max_id = nodes.iter().max_by(|x, y| x.object_id.cmp(&y.object_id)).unwrap().object_id;
    let min_id = nodes.iter().min_by(|x, y| x.object_id.cmp(&y.object_id)).unwrap().object_id;
    nodes.iter().map(|n| {
        if !n.is_boundary { 
            [0.3, 0.3, 0.3] 
        } else {
            number_to_rgb(n.object_id as f32 * 0.95, min_id as f32, max_id as f32)
        } 
    }).collect()
}

// fn color_from_pressure2(
//     nodes: &[Node],
//     connections_structure: &[Vec<(usize, f32, f32)>]
// ) -> Vec<[f32; 3]> {

//     nodes.iter().enumerate().map(|(index, n)| {
//         if connections_structure[index].len() < 1 {
//             [0.0, 0.0, 0.0]
//         }
//         else {
//             let pressure = connections_structure[index].iter().fold(0.0, |accum, &(i, dx, v0)| {
//                 let dir = nodes[i].position - n.position;
//                 let l = dir.length();
//                 let c = (dx / l).powi(7) + (dx / l).powi(13);
//                 // let v = dir.normalize() * 3.0 * (1.0 / dx) * c;
//                 // println!("f: {}", 3.0 * (v0 / (dx * dx)) * c);
//                 accum + (3.0 * (1.0 / dx) * c).abs()
//             });
//             number_to_rgb(pressure, 2000.0, 5000.0)
//         }
//     }).collect()
// }

fn color_from_pressure(
    nodes: &[Node],
    connections_structure: &[Vec<(usize, f32, f32)>]
) -> Vec<[f32; 3]> {

    nodes.iter().enumerate().map(|(index, n)| {
        if connections_structure[index].len() < 1 {
            [0.0, 0.0, 0.0]
        }
        else {
            let dx = 0.0005;
            let top = force_derivative_near_node2(nodes, &connections_structure[index], n.position + Vec2::new(0.0, dx)).y;
            let bottom = force_derivative_near_node2(nodes, &connections_structure[index], n.position + Vec2::new(0.0, -dx)).y;
            let right = force_derivative_near_node2(nodes, &connections_structure[index], n.position + Vec2::new(dx, 0.0)).x;
            let left = force_derivative_near_node2(nodes, &connections_structure[index], n.position + Vec2::new(-dx, 0.0)).x;
            let pressure = -0.25 * (-(right - left) - (top - bottom));
            number_to_rgb(pressure, 83600.0, 86400.0)
        }
    }).collect()
}


fn color_from_temperature(
    nodes: &[Node],
    connections_structure: &[Vec<(usize, f32, f32)>],
    objects_interactions: &HashMap<u32, Vec<usize>>,
    dt: f32,
) -> Vec<[f32; 3]> {
    const TEMPERATURE_CACHE_SIZE: usize = 500;
    const RECORD_INTERVAL: f32 = 0.0005;
    const TOTAL_DT: f32 = TEMPERATURE_CACHE_SIZE as f32 * RECORD_INTERVAL;

    static mut TEMPERATURE_CACHE: Vec<Vec<f32>> = Vec::new();
    static mut CURRENT_RECORD: usize = 0;
    static mut CURRENT_DT: f32 = 0.0;
    unsafe {
        TEMPERATURE_CACHE.resize(nodes.len(), vec![0.0; TEMPERATURE_CACHE_SIZE]);
        TEMPERATURE_CACHE
            .iter_mut()
            .for_each(|cache| cache.resize(TEMPERATURE_CACHE_SIZE, 0.0));

        CURRENT_DT += dt;

        if CURRENT_DT > RECORD_INTERVAL {
            CURRENT_RECORD = (CURRENT_RECORD + 1) % TEMPERATURE_CACHE_SIZE;
            let current_temperature = calculate_temperature(nodes, connections_structure);
            TEMPERATURE_CACHE
                .iter_mut()
                .enumerate()
                .for_each(|(node_index, cache)| {
                    cache[CURRENT_RECORD] = current_temperature[node_index];
                });
            CURRENT_DT = 0.0;
        }
    }

    let energy: Vec<f32> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| unsafe { -0.5 * TEMPERATURE_CACHE[i].iter().copied().sum::<f32>() / TOTAL_DT })
        .collect();

    let avg_per_node: Vec<f32> = energy.par_iter().enumerate().map(|(i, n)| {
        let mut sum = energy[i];
        let mut node_count: usize = 0;
        connections_structure[i].iter().for_each(|&(j, dx, v0)| {
            connections_structure[j].iter().for_each(|&(k, dx, v0)| {
                connections_structure[k].iter().for_each(|&(m, dx, v0)| {
                    connections_structure[m].iter().for_each(|&(l, dx, v0)| {
                        sum += energy[l];
                        node_count += 1;
                    });
                    sum += energy[m];
                    node_count += 1;
                });
                sum += energy[k];
                node_count += 1;
            });
            sum += energy[j];
            node_count += 1;
        });
        sum / (node_count as f32 + 1.0)
    }).collect();

    avg_per_node.iter().map(|color| number_to_rgb(*color, -2000.0, 9000.0)).collect()
}

fn color_from_kinetic_energy(
    nodes: &[Node],
) -> Vec<[f32; 3]> {

    nodes
        .iter()
        .map(|n| {
            [n.velocity.length_squared() * 0.5, 0.0, 0.0]
        })
        .collect()
}

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