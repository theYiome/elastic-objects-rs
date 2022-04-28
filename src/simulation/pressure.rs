use glam::Vec2;
use super::node::Node;

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

pub fn pressure_per_node(
    nodes: &[Node],
    connections_structure: &[Vec<(usize, f32, f32)>]
) -> Vec<f32> {

    let dx = 0.0005;
    nodes.iter().enumerate().map(|(index, n)| {
        let top = force_derivative_near_node2(nodes, &connections_structure[index], n.position + Vec2::new(0.0, dx)).y;
        let bottom = force_derivative_near_node2(nodes, &connections_structure[index], n.position + Vec2::new(0.0, -dx)).y;
        let right = force_derivative_near_node2(nodes, &connections_structure[index], n.position + Vec2::new(dx, 0.0)).x;
        let left = force_derivative_near_node2(nodes, &connections_structure[index], n.position + Vec2::new(-dx, 0.0)).x;
        let pressure = -0.25 * (-(right - left) - (top - bottom));
        pressure
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