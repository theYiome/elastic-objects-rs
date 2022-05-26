use super::objects;
use super::Scene;

pub fn generate(object_size: usize) -> Scene {
    let spacing = 0.6 / object_size as f32;
    let mut nodes = objects::build_rectangle(object_size, object_size, spacing, -0.5, -0.7, 1.0, 0.0, 1);
    let mut nodes2 = objects::build_rectangle(object_size, object_size, spacing, -0.4, 0.2, 1.0, 0.0, 2);
    nodes.append(&mut nodes2);

    let connections_map = objects::build_connections_map(&nodes, spacing * 1.1, 100.0, 0);

    Scene {
        nodes,
        connections: connections_map,
        object_repulsion_dx: 0.2,
        object_repulsion_v0: 100.0,
    }
}