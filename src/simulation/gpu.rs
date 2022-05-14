#[cfg(feature = "rust-gpu-tools")]
pub mod gpu {
    use glam::Vec2;
    
    use rust_gpu_tools::{opencl, program_closures, GPUError, Program};

    use crate::simulation::{node::Node, cpu::{start_integrate_velocity_verlet, end_integrate_velocity_verlet}};
    
    pub fn create_opencl_program() -> Program {
        let device = *rust_gpu_tools::Device::all().first().unwrap();
        let opencl_kernel = include_str!("../kernels/simulation.cl");
        let opencl_device = device.opencl_device().unwrap();
        let opencl_program = opencl::Program::from_opencl(opencl_device, opencl_kernel).unwrap();
        Program::Opencl(opencl_program)
    }

    pub fn flat_with_indexes<T: Copy>(nested_slice: &[Vec<T>]) -> (Vec<T>, Vec<usize>) {
        let flat: Vec<T> = nested_slice.iter().flatten().copied().collect();
        let mut indexes: Vec<usize> = Vec::with_capacity(nested_slice.len());

        let mut current_index = 0;
        nested_slice.iter().enumerate().for_each(|(i, _)| {
            let new_index = current_index + nested_slice[i].len();
            indexes.push(new_index);
            current_index = new_index;
        });

        (flat, indexes)
    }
    
    pub fn simulate_opencl(
        dt: f32,
        nodes: &mut [Node],
        flat_collisions: &[usize],
        collisions_indexes: &[usize],
        flat_connections: &[(usize, f32, f32)],
        connections_indexes: &[usize],
        program: &Program
    ) {

        start_integrate_velocity_verlet(dt, nodes);

        {       
            let fun = program_closures!(|program, _args| -> Result<Vec<Vec2>, GPUError> {
                // Copy the data to the GPU.
                let node_buffer = program.create_buffer_from_slice(&nodes)?;
                let collision_indexes_buffer = program.create_buffer_from_slice(&collisions_indexes)?;
                let collision_structure_buffer = program.create_buffer_from_slice(&flat_collisions)?;

                let connection_indexes_buffer = program.create_buffer_from_slice(&connections_indexes)?;
                let connection_structure_buffer = program.create_buffer_from_slice(&flat_connections)?;
    
                let mut result: Vec<Vec2> = vec![Vec2::new(0.0, 0.0); nodes.len()];
                let result_buffer = program.create_buffer_from_slice(&result)?;
    
                // Get the kernel.
                let block_size = 1024;
                let block_count = nodes.len() / block_size + 1;
                let kernel = program.create_kernel("main", block_size, block_count)?;
        
                // Execute the kernel.
                kernel
                    .arg(&(nodes.len() as u32))
                    .arg(&node_buffer)
                    .arg(&collision_indexes_buffer)
                    .arg(&collision_structure_buffer)
                    .arg(&connection_indexes_buffer)
                    .arg(&connection_structure_buffer)
                    .arg(&result_buffer)
                    .run()?;
        
                // Get the resulting data.
                program.read_into_buffer(&result_buffer, &mut result)?;
        
                Ok(result)
            });
        
            let result = program.run(fun, ()).unwrap();
    
            nodes.iter_mut().enumerate().for_each(|(i, n)| {
                n.current_acceleration += result[i];
            });
        }

        end_integrate_velocity_verlet(dt, nodes);
    }
}
