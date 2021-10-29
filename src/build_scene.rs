use glam::Vec2;
use elastic_node::Node;

fn build_object(size_x: usize, size_y: usize, spacing: f32, offset_x: f32, offset_y: f32) -> Vec<Node> {
    let mut object = Vec::with_capacity(size_x * size_y);
    for y in 0..size_y {
        for x in 0..size_x {
            object.push(Node {
                position: Vec2::new(offset_x + (x as f32) * spacing, offset_y + (y as f32) * spacing),
                velocity: Vec2::new(0.0, 0.0),
                mass: 1.0
            });
        }
    }
    return object;
}

fn build_connections(object: &Vec<Node>, search_distance: f32) -> Vec<Vec<usize>> {
    let mut connections: Vec<Vec<usize>> = Vec::new();
    for i in 0..object.len() {

        let mut row: Vec<usize> = Vec::new();

        for j in 0..object.len() {
            if i == j { continue };

            if Node::distance(&object[i], &object[j]) < search_distance {
                row.push(j);
            }
        }

        connections.push(row);
    }
    return connections;
}