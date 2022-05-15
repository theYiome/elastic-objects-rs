#[cfg(feature = "rust-gpu-tools")]
pub mod gpu {
    use glam::Vec2;
    use crate::simulation::{node::Node, cpu::{start_integrate_velocity_verlet, end_integrate_velocity_verlet}};

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

    use opencl3::{command_queue::{CommandQueue, CL_QUEUE_PROFILING_ENABLE, CL_QUEUE_OUT_OF_ORDER_EXEC_MODE_ENABLE}, types::cl_bool};
    use opencl3::context::Context;
    use opencl3::device::{get_all_devices, Device, CL_DEVICE_TYPE_GPU};
    use opencl3::kernel::{ExecuteKernel, Kernel};
    use opencl3::memory::{Buffer, CL_MEM_READ_ONLY, CL_MEM_WRITE_ONLY};
    use opencl3::program::Program;
    use opencl3::types::{CL_BLOCKING, CL_NON_BLOCKING};
    use std::ptr;

    const PROGRAM_SOURCE: &str = include_str!("../kernels/simulation.cl");
    const KERNEL_NAME: &str = "main";
    const BLANK_BUFFER_SIZE: usize = 1;
    const WRITE_TYPE: cl_bool = CL_NON_BLOCKING;

    pub struct SimulationEngine {
        context: Context,
        command_queue: CommandQueue,
        kernel: Kernel,
        node_count: usize,
        node_buffer: Buffer<Node>,
        collision_index_buffer: Buffer<usize>,
        collision_buffer: Buffer<usize>,
        connection_index_buffer: Buffer<usize>,
        connection_buffer: Buffer<(usize, f32, f32)>,
        result_buffer: Buffer<Vec2>,
    }

    impl SimulationEngine {

        pub fn new() -> SimulationEngine {
            // Find a usable device for this application
            let device_id = *get_all_devices(CL_DEVICE_TYPE_GPU)
                .unwrap()
                .first()
                .expect("No OpenCL GPU found");
        
            let device = Device::new(device_id);

            // Create a Context on an OpenCL device
            let context = Context::from_device(&device).expect("Context::from_device failed");

            // Create a command_queue on the Context's device
            let queue = CommandQueue::create_with_properties(
                &context,
                context.default_device(),
                CL_QUEUE_OUT_OF_ORDER_EXEC_MODE_ENABLE,
                0
            ).expect("CommandQueue::create failed");

            // Build the OpenCL program source and create the kernel.
            let program = Program::create_and_build_from_source(&context, PROGRAM_SOURCE, "")
                .expect("Program::create_and_build_from_source failed");
            let kernel = Kernel::create(&program, KERNEL_NAME).expect("Kernel::create failed");

            let node_buffer = Buffer::<Node>::create(
                &context,
                CL_MEM_READ_ONLY,
                BLANK_BUFFER_SIZE,
                ptr::null_mut(),
            ).unwrap();

            let collision_index_buffer = Buffer::<usize>::create(
                &context,
                CL_MEM_READ_ONLY,
                BLANK_BUFFER_SIZE,
                ptr::null_mut(),
            ).unwrap();

            let collision_buffer = Buffer::<usize>::create(
                &context,
                CL_MEM_READ_ONLY,
                BLANK_BUFFER_SIZE,
                ptr::null_mut(),
            ).unwrap();

            let connection_index_buffer = Buffer::<usize>::create(
                &context,
                CL_MEM_READ_ONLY,
                BLANK_BUFFER_SIZE,
                ptr::null_mut(),
            ).unwrap();

            let connection_buffer = Buffer::<(usize, f32, f32)>::create(
                &context,
                CL_MEM_READ_ONLY,
                BLANK_BUFFER_SIZE,
                ptr::null_mut(),
            ).unwrap();

            let result_buffer = Buffer::<Vec2>::create(
                &context,
                CL_MEM_WRITE_ONLY,
                BLANK_BUFFER_SIZE,
                ptr::null_mut(),
            ).unwrap();

            Self {
                context,
                command_queue: queue,
                kernel,
                node_count: 0,
                node_buffer,
                collision_index_buffer,
                collision_buffer,
                connection_index_buffer,
                connection_buffer,
                result_buffer,
            }
        }

        pub fn write_node_buffer(&mut self, nodes: &[Node]) {
            self.command_queue.enqueue_write_buffer(&mut self.node_buffer, WRITE_TYPE, 0, &nodes, &[]).unwrap();
        }

        pub fn update_node_buffer(&mut self, nodes: &[Node]) {
            self.node_count = nodes.len();
            self.node_buffer = Buffer::<Node>::create(
                &self.context,
                CL_MEM_READ_ONLY,
                self.node_count,
                ptr::null_mut(),
            ).unwrap();
            self.result_buffer = Buffer::<Vec2>::create(
                &self.context,
                CL_MEM_WRITE_ONLY,
                self.node_count,
                ptr::null_mut(),
            ).unwrap();
            self.write_node_buffer(nodes);
        }

        // pub fn write_collision_buffer(&mut self, data: &[Vec<usize>]) {
        //     let (flat, index) = flat_with_indexes(&data);
        //     self.command_queue.enqueue_write_buffer(&mut self.collision_buffer, WRITE_TYPE, 0, &flat, &[]).unwrap();
        //     self.command_queue.enqueue_write_buffer(&mut self.collision_index_buffer, WRITE_TYPE, 0, &index, &[]).unwrap();
        // }

        pub fn update_collision_buffer(&mut self, data: &[Vec<usize>]) {
            let (flat, index) = flat_with_indexes(&data);

            if flat.len() > 0 {
                self.collision_buffer = Buffer::<usize>::create(
                    &self.context,
                    CL_MEM_READ_ONLY,
                    flat.len(),
                    ptr::null_mut(),
                ).unwrap();
                self.command_queue.enqueue_write_buffer(&mut self.collision_buffer, WRITE_TYPE, 0, &flat, &[]).unwrap();
            }

            self.collision_index_buffer = Buffer::<usize>::create(
                &self.context,
                CL_MEM_READ_ONLY,
                index.len(),
                ptr::null_mut(),
            ).unwrap();
            self.command_queue.enqueue_write_buffer(&mut self.collision_index_buffer, WRITE_TYPE, 0, &index, &[]).unwrap();
        }

        // pub fn write_connection_buffer(&mut self, data: &[Vec<(usize, f32, f32)>]) {
        //     let (flat, index) = flat_with_indexes(&data);
        //     self.command_queue.enqueue_write_buffer(&mut self.connection_buffer, WRITE_TYPE, 0, &flat, &[]).unwrap();
        //     self.command_queue.enqueue_write_buffer(&mut self.connection_index_buffer, WRITE_TYPE, 0, &index, &[]).unwrap();
        // }

        pub fn update_connection_buffer(&mut self, data: &[Vec<(usize, f32, f32)>]) {
            let (flat, index) = flat_with_indexes(&data);

            if flat.len() > 0 {
                self.connection_buffer = Buffer::<(usize, f32, f32)>::create(
                    &self.context,
                    CL_MEM_READ_ONLY,
                    flat.len(),
                    ptr::null_mut(),
                ).unwrap();
                self.command_queue.enqueue_write_buffer(&mut self.connection_buffer, WRITE_TYPE, 0, &flat, &[]).unwrap();
            }

            self.connection_index_buffer = Buffer::<usize>::create(
                &self.context,
                CL_MEM_READ_ONLY,
                index.len(),
                ptr::null_mut(),
            ).unwrap();
            self.command_queue.enqueue_write_buffer(&mut self.connection_index_buffer, WRITE_TYPE, 0, &index, &[]).unwrap();
        }

        fn run_kernel(&self) -> Vec<Vec2> {

            self.command_queue.finish().unwrap();

            let kernel_event = ExecuteKernel::new(&self.kernel)
                .set_arg(&self.node_count)
                .set_arg(&self.node_buffer)
                .set_arg(&self.collision_index_buffer)
                .set_arg(&self.collision_buffer)
                .set_arg(&self.connection_index_buffer)
                .set_arg(&self.connection_buffer)
                .set_arg(&self.result_buffer)
                .set_global_work_size(self.node_count)
                .enqueue_nd_range(&self.command_queue).unwrap();

            // Create a results array to hold the results from the OpenCL device
            // and enqueue a read command to read the device buffer into the array
            // after the kernel event completes.
            let mut result: Vec<Vec2> = vec![Vec2::new(0.0, 0.0); self.node_count];
            
            self.command_queue.enqueue_read_buffer(
                &self.result_buffer, 
                CL_BLOCKING, 
                0, 
                &mut result, 
                &vec![kernel_event.get()]
            ).unwrap();

            // Calculate the kernel duration, from the kernel_event

            // let start_time = kernel_event.profiling_command_start().unwrap();
            // let end_time = kernel_event.profiling_command_end().unwrap();
            // let duration = end_time - start_time;
            // println!("kernel execution duration (ns): {}", duration);

            result
        }
        
        pub fn simulate_opencl(
            &mut self,
            dt: f32,
            nodes: &mut [Node],
        ) {
    
            start_integrate_velocity_verlet(dt, nodes);

            {
                self.write_node_buffer(nodes);

                let result = self.run_kernel();
                assert_eq!(result.len(), nodes.len());

                nodes.iter_mut().enumerate().for_each(|(i, n)| {
                    n.current_acceleration += result[i];
                });
            }
    
            end_integrate_velocity_verlet(dt, nodes);
        }
    }
}
