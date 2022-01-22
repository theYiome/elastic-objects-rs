use crate::elastic_node::Node;
use glam::Vec2;
use rayon::prelude::*;

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
    object.iter_mut().for_each(|n| {
        n.position += (n.velocity * dt) + (0.5 * n.current_acceleration * dt * dt);

        n.last_acceleration = n.current_acceleration;
        n.current_acceleration *= 0.0;
    });
}

fn end_integrate_velocity_verlet(dt: f32, object: &mut Vec<Node>) {
    object.iter_mut().for_each(|n| {
        n.velocity += 0.5 * (n.last_acceleration + n.current_acceleration) * dt;
    });
}

// https://users.rust-lang.org/t/help-with-parallelizing-a-nested-loop/22568/2
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



fn repulsion_force_simple(object: &mut Vec<Node>) {
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

fn repulsion_force_stack_overflow(object: &mut Vec<Node>) {
    let v0 = 500.0;
    let dx = 0.1;

    for i in 0..object.len() {
        let (left, right) = object.split_at_mut(i + 1);
        let mut node_i = &mut left[i];

        right.iter_mut().for_each(|node_j| {
            let dir = node_j.position - node_i.position;
            let l = dir.length();
            let mi = node_i.mass;
            let mj = node_j.mass;

            let c = (dx / l).powi(13);
            let v = dir.normalize() * 3.0 * (v0 / dx) * c;

            node_i.current_acceleration -= v / mi;
            node_j.current_acceleration += v / mj;
        });
    }
}


// fn repulsion_force_stack_overflow_2(object: &mut Vec<Node>) {
//     let v0 = 500.0;
//     let dx = 0.1;

//     let lenght = object.len();
//     (0..lenght).par_bridge().for_each(|i| {
//         let (left, right) = object.split_at_mut(i + 1);
//         let mut node_i = &mut left[i];

//         right.iter_mut().for_each(|node_j| {
//             let dir = node_j.position - node_i.position;
//             let l = dir.length();
//             let mi = node_i.mass;
//             let mj = node_j.mass;

//             let c = (dx / l).powi(13);
//             let v = dir.normalize() * 3.0 * (v0 / dx) * c;

//             node_i.current_acceleration -= v / mi;
//             node_j.current_acceleration += v / mj;
//         });
//     });
// }


// fn repulsion_force_stack_overflow_par(object: &mut Vec<Node>) {
//     let v0 = 500.0;
//     let dx = 0.1;

//     for i in 0..object.len() {
//         let (left, right) = object.split_at_mut(i + 1);
//         let mut node_i = &mut left[i];

//         right.iter_mut().par_bridge().for_each(|node_j| {
//             let dir = node_j.position - node_i.position;
//             let l = dir.length();
//             let mi = node_i.mass;
//             let mj = node_j.mass;

//             let c = (dx / l).powi(13);
//             let v = dir.normalize() * 3.0 * (v0 / dx) * c;

//             node_i.current_acceleration -= v / mi;
//             node_j.current_acceleration += v / mj;
//         });
//     }
// }
fn repulsion_force4(object: &mut Vec<Node>) {
    let v0 = 500.0;
    let dx = 0.1;

    let length = object.len();
    let obj2 = object.clone();

    object.par_iter_mut()
        .enumerate()
        .for_each(|(i, n)| {
            (0..length).for_each(|j| {
                if j != i {
                    let dir = obj2[j].position - n.position;
                    let l = dir.length();
                    let mi = n.mass;
                    // let mj = obj2[j].mass;
        
                    let c = (dx / l).powi(13);
                    let v = dir.normalize() * 3.0 * (v0 / dx) * c;
        
                    n.current_acceleration -= v / mi;
                }
                // object[j].current_acceleration += v / mj;
            });
        });
        // .zip(object.par_iter().enumerate())
        // .for_each(|((i, a), (j, b))| {
        // if j > i {
        //     //
        // } 
}


// fn repulsion_force(object: &mut [Node]) {
//     for i in 0..object.len() {
//         let (left, right) = object.split_at_mut(i + 1);
//         let mut node_i = &mut left[i];
//         right.iter_mut().par_bridge().for_each(|node_j| {
            
//             let dir = node_j.position - node_i.position;
//             let l = dir.length();
//             let mi = node_i.mass;
//             let mj = node_j.mass;

//             let c = (dx / l).powi(13);
//             let v = dir.normalize() * 3.0 * (v0 / dx) * c;

//             // object[i].current_acceleration -= v / mi;
//             // object[j].current_acceleration += v / mj;

//             node_i.current_acceleration.fetch_sub(v / mi, Relaxed);
//             node_j.current_acceleration.fetch_add(v / mj, Relaxed);
//         });
//     }
// }

// fn repulsion_force7(object: &mut Vec<Node>) {
//     let v0 = 500.0;
//     let dx = 0.1;

//     let length = object.len();

//     (0..length).par_bridge().for_each(|i| {
//         (i+1..length).for_each(|j| {
//             let dir = object[j].position - object[i].position;
//             let l = dir.length();
//             let mi = object[i].mass;
//             let mj = object[j].mass;

//             let c = (dx / l).powi(13);
//             let v = dir.normalize() * 3.0 * (v0 / dx) * c;

//             object[i].current_acceleration -= v / mi;
//             object[j].current_acceleration += v / mj;
//         });
//     });
    
// }

fn wall_repulsion_force_y(object: &mut Vec<Node>) {
    let v0 = 200.0;
    let dx = 0.1;

    object.iter_mut().for_each(|n| {
        let dir = glam::vec2(n.position.x, -1.05) - n.position;
        let l = dir.length();
        let mi = n.mass;

        let c = (dx / l).powi(13);
        let v = dir.normalize() * 3.0 * (v0 / dx) * c;
        
        n.current_acceleration -= v / mi;
    });
}

fn wall_repulsion_force_x0(object: &mut Vec<Node>) {
    let v0 = 100.0;
    let dx = 0.1;

    object.iter_mut().for_each(|n| {
        let dir = glam::vec2(-1.05, n.position.y) - n.position;
        let l = dir.length();
        let mi = n.mass;

        let c = (dx / l).powi(13);
        let v = dir.normalize() * 3.0 * (v0 / dx) * c;

        n.current_acceleration -= v / mi;
    });
}



fn wall_repulsion_force_x1(object: &mut Vec<Node>) {
    let v0 = 100.0;
    let dx = 0.1;

    object.iter_mut().for_each(|n| {
        let dir = glam::vec2(1.05, n.position.y) - n.position;
        let l = dir.length();
        let mi = n.mass;

        let c = (dx / l).powi(13);
        let v = dir.normalize() * 3.0 * (v0 / dx) * c;

        n.current_acceleration -= v / mi;
    });
}

fn gravity_force(object: &mut Vec<Node>) {
    object.iter_mut().for_each(|n| {
        n.current_acceleration += glam::vec2(0.0, -10.0);
    });
}

fn drag_force(object: &mut Vec<Node>) {
    object.iter_mut().for_each(|n| {
        n.current_acceleration -= n.velocity * 0.9;
    });
}

pub fn simulate(dt: f32, object: &mut Vec<Node>, connections: &Vec<Vec<usize>>) {
    start_integrate_velocity_verlet(dt, object);

    gravity_force(object);

    attraction_force(object, connections);

    // repulsion_force_stack_overflow(object);
    repulsion_force4(object);

    wall_repulsion_force_y(object);
    wall_repulsion_force_x0(object);
    wall_repulsion_force_x1(object);

    drag_force(object);

    end_integrate_velocity_verlet(dt, object);
}
