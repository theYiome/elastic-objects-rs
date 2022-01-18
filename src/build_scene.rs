use crate::elastic_node::Node;
use glam::Vec2;

pub fn build_object(
    size_x: usize,
    size_y: usize,
    spacing: f32,
    offset_x: f32,
    offset_y: f32,
) -> Vec<Node> {
    let mut object = Vec::with_capacity(size_x * size_y);
    for y in 0..size_y {
        for x in 0..size_x {
            object.push(Node {
                position: Vec2::new(
                    offset_x + (x as f32) * spacing,
                    offset_y + (y as f32) * spacing,
                ),
                velocity: Vec2::new(0.0, 0.0),
                current_acceleration: Vec2::new(0.0, 0.0),
                last_acceleration: Vec2::new(0.0, 0.0),
                mass: 1.0,
            });
        }
    }
    return object;
}

pub fn build_connections(object: &Vec<Node>, search_distance: f32) -> Vec<Vec<usize>> {
    let mut connections: Vec<Vec<usize>> = Vec::new();
    for i in 0..object.len() {
        let mut row: Vec<usize> = Vec::new();

        for j in 0..object.len() {
            if i == j {
                continue;
            };

            if Node::distance(&object[i], &object[j]) < search_distance {
                row.push(j);
            }
        }

        connections.push(row);
    }
    return connections;
}

// https://en.wikipedia.org/wiki/Verlet_integration#Velocity_Verlet
fn start_integrate_velocity_verlet(dt: f32, object: &mut Vec<Node>) {
    for n in object {
        n.position += (n.velocity * dt) + (0.5 * n.current_acceleration * dt * dt);

        n.last_acceleration = n.current_acceleration;
        n.current_acceleration *= 0.0;
    }
}

fn end_integrate_velocity_verlet(dt: f32, object: &mut Vec<Node>) {
    for n in object {
        n.velocity += 0.5 * (n.last_acceleration + n.current_acceleration) * dt;
    }
}

fn attraction_force(object: &mut Vec<Node>, connections: &Vec<Vec<usize>>) {
    let v0 = 500.0;
    let dx = 0.1;

    for (i, list) in connections.iter().enumerate() {
        for j in list {
            let dir = object[*j].position - object[i].position;
            let l = dir.length();
            let m = object[i].mass;

            let c = (dx / l).powi(7);
            let v = dir.normalize() * 3.0 * (v0 / dx) * c;

            object[i].current_acceleration += v / m;
        }
    }
}

fn repulsion_force(object: &mut Vec<Node>) {
    let v0 = 500.0;
    let dx = 0.1;

    for i in 0..object.len() {
        for j in i + 1..object.len() {
            let dir = object[j].position - object[i].position;
            let l = dir.length();
            let mi = object[i].mass;
            let mj = object[j].mass;

            let c = (dx / l).powi(13);
            let v = dir.normalize() * 3.0 * (v0 / dx) * c;

            object[i].current_acceleration -= v / mi;
            object[j].current_acceleration += v / mj;
        }
    }
}

fn wall_repulsion_force_y(object: &mut Vec<Node>) {
    let v0 = 200.0;
    let dx = 0.1;

    for n in object.iter_mut() {
        let dir = glam::vec2(n.position.x, -1.05) - n.position;
        let l = dir.length();
        let mi = n.mass;

        let c = (dx / l).powi(13);
        let v = dir.normalize() * 3.0 * (v0 / dx) * c;

        n.current_acceleration -= v / mi;
    }
}

fn wall_repulsion_force_x0(object: &mut Vec<Node>) {
    let v0 = 100.0;
    let dx = 0.1;

    for n in object.iter_mut() {
        let dir = glam::vec2(-1.05, n.position.y) - n.position;
        let l = dir.length();
        let mi = n.mass;

        let c = (dx / l).powi(13);
        let v = dir.normalize() * 3.0 * (v0 / dx) * c;

        n.current_acceleration -= v / mi;
    }
}

fn wall_repulsion_force_x1(object: &mut Vec<Node>) {
    let v0 = 100.0;
    let dx = 0.1;

    for n in object.iter_mut() {
        let dir = glam::vec2(1.05, n.position.y) - n.position;
        let l = dir.length();
        let mi = n.mass;

        let c = (dx / l).powi(13);
        let v = dir.normalize() * 3.0 * (v0 / dx) * c;

        n.current_acceleration -= v / mi;
    }
}


fn gravity_force(object: &mut Vec<Node>) {
    for n in object.iter_mut() {
        n.current_acceleration +=  glam::vec2(0.0, -10.0);
    }
}

fn drag_force(object: &mut Vec<Node>) {
    for n in object.iter_mut() {
        n.current_acceleration -= n.velocity * 0.9;
    }
}

pub fn simulate(dt: f32, object: &mut Vec<Node>, connections: &Vec<Vec<usize>>) {
    start_integrate_velocity_verlet(dt, object);

    gravity_force(object);

    attraction_force(object, connections);

    repulsion_force(object);

    wall_repulsion_force_y(object);
    wall_repulsion_force_x0(object);
    wall_repulsion_force_x1(object);

    drag_force(object);

    end_integrate_velocity_verlet(dt, object);
}
