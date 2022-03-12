use std::{collections::HashMap, f32::consts::PI};

use crate::graphics;
use crate::node::Node;
use glam::Vec2;
use rayon::iter::{self, IntoParallelRefMutIterator, IndexedParallelIterator, ParallelIterator, IntoParallelRefIterator};

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

fn calculate_temperatue(nodes: &Vec<Node>, connections: &HashMap<(usize, usize), (f32, f32)>, objects: &Vec<Vec<usize>>) -> Vec<f32> {
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

                    let c = (dx / l).powi(13);
                    let v = dir.normalize() * 3.0 * (v0 / dx) * c;

                    forces[a] -= v;
                    forces[b] += v;
                }
            }
        }
    }

    let v0 = 200.0;
    let dx = 0.05;

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

    return forces.iter().enumerate().map(|(i, f)| f.dot(nodes[i].position).abs()).collect();
}

pub fn draw_scene(
    nodes: &Vec<Node>,
    connections: &HashMap<(usize, usize), (f32, f32)>,
    objects: &Vec<Vec<usize>>,
    dt: f32
) -> (Vec<graphics::Vertex>, Vec<u16>) {
    let mut verticies: Vec<graphics::Vertex> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    // graphics and window creation
    for (k, v) in connections {
        let (dx, v0) = *v;
        let (a, b) = (nodes[k.0].position, nodes[k.1].position);
        let color = ((a - b).length() - dx) * 20.0;
        graphics::add_rectangle(
            &mut verticies,
            &mut indices,
            a,
            b,
            0.007 + v0 * 0.00001,
            [0.2 + color, 0.2 + color, 0.2 + color],
        );
    }
    
    const TEMPERATURE_CACHE_SIZE: usize = 200;
    static mut TEMPERATURE_CACHE: Vec<Vec<f32>> = Vec::new();
    static mut DT_CACHE: Vec<f32> = Vec::new();
    static mut CURRENT_RECORD: usize = 0;
    unsafe {
        TEMPERATURE_CACHE.resize(nodes.len(), vec![0.0; TEMPERATURE_CACHE_SIZE]);
        TEMPERATURE_CACHE.iter_mut().for_each(|cache| cache.resize(TEMPERATURE_CACHE_SIZE, 0.0));
        DT_CACHE.resize(TEMPERATURE_CACHE_SIZE, 0.0);

        if dt > 0.0 {
            CURRENT_RECORD = (CURRENT_RECORD + 1) % TEMPERATURE_CACHE_SIZE;
            // println!("{CURRENT_RECORD} ");
            let current_temperature = calculate_temperatue(nodes, connections, objects);
            TEMPERATURE_CACHE.iter_mut().enumerate().for_each(|(node_index, cache)| {
                cache[CURRENT_RECORD] = current_temperature[node_index];
            });
            DT_CACHE[CURRENT_RECORD] = dt;
        }
    }

    let total_dt = unsafe{ 
        let dt_sum = DT_CACHE.iter().copied().sum::<f32>();
        if dt_sum > 0.0 { dt_sum } else { f32::INFINITY }
    };
    
    nodes.iter().enumerate().for_each(|(i, n)|  {
        let color: f32 = unsafe { 0.5 * TEMPERATURE_CACHE[i].iter().copied().sum::<f32>() / total_dt };

        // if (color > 10.0 || color < -10.0) {
        //     println!("{i}: {color}");
        // }
        
        // let color = n.velocity.length_squared() * 0.5 * 0.7;
        graphics::add_circle(
            &mut verticies,
            &mut indices,
            n.position.x,
            n.position.y,
            0.01 + radius_from_area(n.mass) * 0.01,
            16,
            number_to_rgb(color, -3000.0, 200000.0),
        );
    });

    (verticies, indices)
}

fn number_to_rgb(mut t: f32, min: f32, max: f32) -> [f32; 3] {
    assert!(max > min);
  
    t = if t < min { min } else { if t > max { max } else { t } };
    let n_t: f32 = (t - min) / (max - min);
    let regions: [f32; 3] = [1.0 / 4.0, (1.0 / 4.0) * 2.0, (1.0 / 4.0) * 3.0];

    return  {
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