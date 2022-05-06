use std::collections::HashMap;
use std:: f32::consts::PI;

use crate::simulation::node::Node;
use crate::simulation;

#[derive(Copy, Clone)]
pub struct Vertex {
    local_position: [f32; 2],
}
glium::implement_vertex!(Vertex, local_position);

#[derive(Copy, Clone)]
pub struct NodeAttribute {
    position: [f32; 2],
    scale_x: f32,
    scale_y: f32,
    rotation: f32,
    color: [f32; 3],
}
glium::implement_vertex!(
    NodeAttribute,
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
    coloring_mode: &ColoringMode,
    dt: f32
) -> Vec<NodeAttribute> {

    // let colors = color_from_kinetic_energy(nodes);
    let colors = match coloring_mode {
        ColoringMode::KineticEnergy => color_from_kinetic_energy(nodes),
        ColoringMode::Temperature => color_from_temperature(nodes, connections_structure, dt),
        ColoringMode::Boundary => color_from_boundary(nodes),
        ColoringMode::Pressure => color_from_pressure(nodes, connections_structure)
    };

    nodes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let radius = 0.0045 + radius_from_area(n.mass) * 0.000;

            NodeAttribute {
                position: n.position.to_array(),
                scale_x: radius,
                scale_y: radius,
                rotation: 0.0,
                color: colors[i],
            }
        })
        .collect()
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

fn color_from_pressure(
    nodes: &[Node],
    connections_structure: &[Vec<(usize, f32, f32)>]
) -> Vec<[f32; 3]> {

    simulation::pressure::pressure_per_node(nodes, connections_structure)
        .iter()
        .map(|pressure| {
            number_to_rgb(*pressure, 83600.0, 86400.0)
        })
        .collect()
}

fn color_from_temperature(
    nodes: &[Node],
    connections_structure: &[Vec<(usize, f32, f32)>],
    dt: f32
) -> Vec<[f32; 3]> {

    simulation::temperature::cached_avg_temperature_per_node(nodes, connections_structure, dt)
        .iter()
        .map(|temperature| {
            number_to_rgb(*temperature, -2000.0, 9000.0)
        })
        .collect()
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



#[derive(Copy, Clone)]
pub struct ConnectionAttribute {
    position_a: [f32; 2],
    position_b: [f32; 2],
    color: [f32; 3],
    width: f32,
}
glium::implement_vertex!(
    ConnectionAttribute,
    position_a,
    position_b,
    color,
    width,
);

pub fn draw_connections(connections: &HashMap<(usize, usize), (f32, f32)>, nodes: &[Node]) -> Vec<ConnectionAttribute> {
    connections.iter().map(|(k, _v)| {
        // let (dx, v0) = *v;
        ConnectionAttribute {
            position_a: nodes[k.0].position.to_array(),
            position_b: nodes[k.1].position.to_array(),
            color: [0.1, 0.1, 0.1],
            width: 0.0016
        }
    }).collect()
}