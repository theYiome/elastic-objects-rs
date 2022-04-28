#[cfg(feature = "rust-gpu-tools")]
mod gpu {
    use std::collections::HashMap;
    use super::node::Node;
    use glam::Vec2;
    
    use rust_gpu_tools::{cuda, opencl, program_closures, Device, GPUError, Program};
    
    fn cuda(device: &Device) -> Program {
        // The kernel was compiled with:
        // nvcc -fatbin -gencode=arch=compute_52,code=sm_52 -gencode=arch=compute_60,code=sm_60 -gencode=arch=compute_61,code=sm_61 -gencode=arch=compute_70,code=sm_70 -gencode=arch=compute_75,code=sm_75 -gencode=arch=compute_75,code=compute_75 --x cu add.cl
        let cuda_kernel = include_bytes!("./kernels/simulation.fatbin");
        let cuda_device = device.cuda_device().unwrap();
        let cuda_program = cuda::Program::from_bytes(cuda_device, cuda_kernel).unwrap();
    
        Program::Cuda(cuda_program)
    }
    
    pub fn create_opencl_program() -> Program {
        let device = *rust_gpu_tools::Device::all().first().unwrap();
        let opencl_kernel = include_str!("./kernels/simulation.cl");
        let opencl_device = device.opencl_device().unwrap();
        let opencl_program = opencl::Program::from_opencl(opencl_device, opencl_kernel).unwrap();
        Program::Opencl(opencl_program)
    }
    
    pub fn simulate_opencl(
        nodes: &Vec<Node>,
        program: &Program,
        connections_map: &HashMap<(usize, usize), (f32, f32)>,
        iterations: u32,
        dt: f32,
    ) -> Vec<Node> {
    
        let mut connections_keys: Vec<(u32, u32)> = Vec::new();
        let mut connections_vals: Vec<(f32, f32)> = Vec::new();
        for (k1, k2) in connections_map {
            connections_keys.push((k1.0 as u32, k1.1 as u32));
            connections_vals.push(*k2);
        }
    
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
                    drag: 0.0,
                    object_id: 1,
                    is_boundary: false
                },
            );
            program.read_into_buffer(&node_buffer, &mut result)?;
    
            Ok(result)
        });
    
        let result = program.run(closures, ()).unwrap();
        result
    }
}
