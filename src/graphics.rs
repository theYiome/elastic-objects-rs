use std::{collections::HashMap, f32::consts::PI};

use crate::graphics;
use crate::node::Node;
use glam::Vec2;

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 2],
    col: [f32; 3],
}

glium::implement_vertex!(Vertex, position, col);

/// Adds verticies and indices representing circle shape to existing conainer.
/// `radius` must be greater than `0.0`
///
/// ## `nr_of_triangles`
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
pub fn add_circle(
    verticies: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    center_x: f32,
    center_y: f32,
    radius: f32,
    nr_of_triangles: u16,
    color: [f32; 3],
) {
    assert!(nr_of_triangles > 2);
    assert!(radius >= 0.0);

    let zero_index: u16 = match indices.iter().max() {
        Some(m) => m + 1,
        None => 0,
    };

    let delta_angle = (2.0 * PI) / nr_of_triangles as f32;

    // circle center vertex, index (zero_index + 0)
    verticies.push(Vertex {
        position: [center_x, center_y],
        col: color,
    });

    // index (zero_index + 1)
    verticies.push(Vertex {
        position: [center_x, center_y + radius],
        col: color,
    });

    for i in 2..nr_of_triangles + 1 {
        let angle = delta_angle * (i - 1) as f32;
        let x = angle.sin() * radius;
        let y = angle.cos() * radius;

        let vert = Vertex {
            position: [center_x + x, center_y + y],
            col: color,
        };

        verticies.push(vert);
        indices.push(zero_index);
        indices.push(zero_index + i - 1);
        indices.push(zero_index + i);
    }

    indices.push(zero_index);
    indices.push(zero_index + nr_of_triangles);
    indices.push(zero_index + 1);
}

pub fn add_rectangle(
    verticies: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    point_a: Vec2,
    point_b: Vec2,
    height: f32,
    color: [f32; 3],
) {
    assert!(height >= 0.0);

    let zero_index: u16 = match indices.iter().max() {
        Some(m) => m + 1,
        None => 0,
    };

    let unit_vector = (point_b - point_a).normalize();
    let perpendicular_unit_vector = Vec2::new(unit_vector.y, -unit_vector.x);

    verticies.push(Vertex {
        position: (point_a + perpendicular_unit_vector * (height / 2.0)).to_array(),
        col: color,
    });

    verticies.push(Vertex {
        position: (point_a - perpendicular_unit_vector * (height / 2.0)).to_array(),
        col: color,
    });

    verticies.push(Vertex {
        position: (point_b + perpendicular_unit_vector * (height / 2.0)).to_array(),
        col: color,
    });

    verticies.push(Vertex {
        position: (point_b - perpendicular_unit_vector * (height / 2.0)).to_array(),
        col: color,
    });

    indices.push(zero_index);
    indices.push(zero_index + 1);
    indices.push(zero_index + 2);

    indices.push(zero_index + 1);
    indices.push(zero_index + 2);
    indices.push(zero_index + 3);
}

pub fn radius_from_area(area: f32) -> f32 {
    (area / PI).sqrt()
}

pub fn draw_scene(
    nodes: &Vec<Node>,
    connections: &HashMap<(usize, usize), (f32, f32)>,
) -> (Vec<graphics::Vertex>, Vec<u16>) {
    let mut verticies: Vec<graphics::Vertex> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();
    // graphics and window creation

    for (k, v) in connections {
        let (dx, v0) = *v;
        let (a, b) = (nodes[k.0].position, nodes[k.1].position);
        let color = ((a - b).length() - dx) * 50.0;
        graphics::add_rectangle(
            &mut verticies,
            &mut indices,
            a,
            b,
            0.007 + v0 * 0.00001,
            [0.2 + color, 0.2 + color, 0.2 + color],
        );
    }

    for n in nodes {
        let color = n.velocity.length_squared() * 0.5 * 0.5;
        graphics::add_circle(
            &mut verticies,
            &mut indices,
            n.position.x,
            n.position.y,
            0.01 + radius_from_area(n.mass) * 0.01,
            20,
            [color, 0.0, 0.0],
        );
    }

    (verticies, indices)
}
