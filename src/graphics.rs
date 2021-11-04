use std::{f32::consts::PI};

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
pub fn add_circle(verticies: &mut Vec<Vertex>, indices: &mut Vec<u16>, center_x: f32, center_y: f32, radius: f32, nr_of_triangles: u16) {
    assert!(nr_of_triangles > 2);
    assert!(radius > 0.0);

    let zero_index: u16 = match indices.iter().max() {
        Some(m) => m + 1,
        None => 0
    };

    let delta_angle = (2.0 * PI) / nr_of_triangles as f32;
    
    // circle center vertex, index (zero_index + 0)
    verticies.push(Vertex {
        position: [center_x, center_y],
        col: [0.3, 0.0, 0.0]
    });

    // index (zero_index + 1)
    verticies.push(Vertex {
        position: [center_x, center_y + radius],
        col: [0.0, 0.0, 0.0]
    });

    for i in 2..nr_of_triangles+1 {
        let angle = delta_angle * (i - 1) as f32;
        let x = angle.sin() * radius;
        let y = angle.cos() * radius;

        let vert = Vertex {
            position: [center_x + x, center_y + y],
            col: [0.0, 0.0, 0.0]
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