use std::collections::HashMap;

use crate::node::Node;
use rayon::prelude::*;

// https://en.wikipedia.org/wiki/Verlet_integration#Velocity_Verlet
fn start_integrate_velocity_verlet(dt: f32, nodes: &mut Vec<Node>) {
    nodes.iter_mut().for_each(|n| {
        n.position += (n.velocity * dt) + (0.5 * n.current_acceleration * dt * dt);

        n.last_acceleration = n.current_acceleration;
        n.current_acceleration *= 0.0;
    });
}

fn end_integrate_velocity_verlet(dt: f32, nodes: &mut Vec<Node>) {
    nodes.iter_mut().for_each(|n| {
        n.velocity += 0.5 * (n.last_acceleration + n.current_acceleration) * dt;
    });
}

// https://users.rust-lang.org/t/help-with-parallelizing-a-nested-loop/22568/2
fn lennard_jones_connections(nodes: &mut Vec<Node>, connections: &HashMap<(usize, usize), (f32, f32)>) {

    connections.keys().for_each(|(a, b)| {
        let i = *a;
        let j = *b;
        let (dx, v0) = *connections.get(&(i, j)).unwrap();

        let dir = nodes[j].position - nodes[i].position;
        let l = dir.length();
        let m = nodes[i].mass;

        let c = (dx / l).powi(7) - (dx / l).powi(13);
        let v = dir.normalize() * 3.0 * (v0 / dx) * c;

        nodes[i].current_acceleration += v / m;
        nodes[j].current_acceleration -= v / m;
    });
}

fn lennard_jones_repulsion(nodes: &mut Vec<Node>, objects: &Vec<Vec<usize>>) {
    let v0 = 20.0;
    let dx = 0.1;

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
                    let mi = nodes[a].mass;
                    let mj = nodes[b].mass;

                    let c = (dx / l).powi(13);
                    let v = dir.normalize() * 3.0 * (v0 / dx) * c;

                    nodes[a].current_acceleration -= v / mi;
                    nodes[b].current_acceleration += v / mj;
                }
            }
        }
    }
}

fn repulsion_force_simple(nodes: &mut Vec<Node>) {
    let v0 = 500.0;
    let dx = 0.1;

    for i in 0..nodes.len() {
        for j in i + 1..nodes.len() {
            let dir = nodes[j].position - nodes[i].position;
            let l = dir.length();
            let mi = nodes[i].mass;
            let mj = nodes[j].mass;

            let c = (dx / l).powi(13);
            let v = dir.normalize() * 3.0 * (v0 / dx) * c;

            nodes[i].current_acceleration -= v / mi;
            nodes[j].current_acceleration += v / mj;
        }
    }
}

fn repulsion_force_iter(nodes: &mut Vec<Node>) {
    let v0 = 500.0;
    let dx = 0.1;

    for i in 0..nodes.len() {
        let (left, right) = nodes.split_at_mut(i + 1);
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

fn wall_repulsion_force_y(nodes: &mut Vec<Node>) {
    let v0 = 200.0;
    let dx = 0.05;

    nodes.iter_mut().for_each(|n| {
        let dir = glam::vec2(n.position.x, -1.0) - n.position;
        let l = dir.length();
        let mi = n.mass;

        let c = (dx / l).powi(13);
        let v = dir.normalize() * 3.0 * (v0 / dx) * c;

        n.current_acceleration -= v / mi;
    });
}

fn gravity_force(nodes: &mut Vec<Node>) {
    nodes.iter_mut().for_each(|n| {
        n.current_acceleration += glam::vec2(0.0, -9.81);
    });
}

fn drag_force(nodes: &mut Vec<Node>) {
    nodes.iter_mut().for_each(|n| {
        n.current_acceleration -= n.velocity * 0.9;
    });
}

pub fn simulate_gpu_accelerated(
    dt: f32,
    nodes: &mut Vec<Node>,
    objects: &mut Vec<Vec<usize>>,
    connections: &HashMap<(usize, usize), (f32, f32)>,
) {
    start_integrate_velocity_verlet(dt, nodes);

    gravity_force(nodes);

    lennard_jones_connections(nodes, connections);
    lennard_jones_repulsion(nodes, objects);

    wall_repulsion_force_y(nodes);
    // drag_force(nodes);

    end_integrate_velocity_verlet(dt, nodes);
}

use crate::node::Node;
use glam::Vec2;
use rust_gpu_tools::{cuda, opencl, program_closures, Device, GPUError, Program};

/// Returns a `Program` that runs on CUDA.
fn cuda(device: &Device) -> Program {
    // The kernel was compiled with:
    // nvcc -fatbin -gencode=arch=compute_52,code=sm_52 -gencode=arch=compute_60,code=sm_60 -gencode=arch=compute_61,code=sm_61 -gencode=arch=compute_70,code=sm_70 -gencode=arch=compute_75,code=sm_75 -gencode=arch=compute_75,code=compute_75 --x cu add.cl
    let cuda_kernel = include_bytes!("./kernels/simulation.fatbin");
    let cuda_device = device.cuda_device().unwrap();
    let cuda_program = cuda::Program::from_bytes(cuda_device, cuda_kernel).unwrap();

    Program::Cuda(cuda_program)
}

/// Returns a `Program` that runs on OpenCL.
fn opencl(device: &Device) -> Program {
    let opencl_kernel = include_str!("./kernels/simulation.cl");
    let opencl_device = device.opencl_device().unwrap();
    let opencl_program = opencl::Program::from_opencl(opencl_device, opencl_kernel).unwrap();
    Program::Opencl(opencl_program)
}

pub fn simulate_opencl(
    nodes: &Vec<Node>,
    program: &Program,
    connections_keys: &Vec<(u32, u32)>,
    connections_vals: &Vec<(f32, f32)>,
    iterations: u32,
    dt: f32,
) -> Vec<Node> {
    let closures = program_closures!(|program, _args| -> Result<Vec<Node>, GPUError> {
        // Make sure the input data has the same length.
        let length = nodes.len();
        let dt_div = if dt != 0.0 { (1.0 / dt) as u32 } else { 0 };
        // println!("{}", dt_div);

        // Copy the data to the GPU.
        let node_buffer = program.create_buffer_from_slice(&nodes)?;
        let connections_keys_buffer = program.create_buffer_from_slice(&connections_keys)?;
        let connections_vals_buffer = program.create_buffer_from_slice(&connections_vals)?;

        // The result buffer has the same length as the input buffers.
        // let result_buffer = unsafe { program.create_buffer::<u32>(length)? };

        // Get the kernel.
        let kernel = program.create_kernel("mainkernel", 1, nodes.len())?;

        // Execute the kernel.
        kernel
            .arg(&(length as u32))
            .arg(&node_buffer)
            .arg(&(connections_keys.len() as u32))
            .arg(&connections_keys_buffer)
            .arg(&connections_vals_buffer)
            .arg(&iterations)
            .arg(&dt_div)
            .run()?;

        // Get the resulting data.
        let mut result: Vec<Node> = Vec::new();
        result.resize(
            length,
            Node {
                position: Vec2::new(0.0, 0.0),
                velocity: Vec2::new(0.0, 0.0),
                current_acceleration: Vec2::new(0.0, 0.0),
                last_acceleration: Vec2::new(0.0, 0.0),
                mass: 1.0,
                damping: 0.0,
            },
        );
        program.read_into_buffer(&node_buffer, &mut result)?;

        Ok(result)
    });

    let result = program.run(closures, ()).unwrap();
    result
}
