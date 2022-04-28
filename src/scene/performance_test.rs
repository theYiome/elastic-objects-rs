use crate::simulation::energy;
use super::generate_scene;


pub fn run_performace_test(object_size: usize, dt: f32, simulation_time: f32, optimized: bool) {

    let (nodes, connections_map, objects) = if optimized {
        generate_scene::performance_test_scene_optimized(object_size)
    } else {
        generate_scene::performance_test_scene(object_size)
    };

    // loging to csv file
    let file_postfix = if optimized {"optimized"} else {"unoptimized"};
    
    let log_path_energy = format!("data/{}x{}_energy_{}.csv", object_size, object_size, file_postfix);
    let mut csv_energy_writer = csv::Writer::from_path(log_path_energy).unwrap();

    let log_path_performance = format!("data/{}x{}_performance_test_{}.csv", object_size, object_size, file_postfix);
    let performance_test_file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(log_path_performance)
        .unwrap();
    let mut csv_performance_writer = csv::Writer::from_writer(performance_test_file);

    let mut total_symulation_time: f32 = 0.0;
    let mut current_log_dt = 0.0;

    let now = std::time::Instant::now();

    let steps_per_frame: u32 = 100;

    while total_symulation_time < simulation_time / (object_size * object_size) as f32 {
        
        for _i in 0..steps_per_frame {
            // simulation_cpu::simulate_single_thread_cpu(dt, &mut nodes, &mut connections_map);
        }

        total_symulation_time += dt * steps_per_frame as f32;

        current_log_dt += dt * steps_per_frame as f32;
        {
            let log_dt = 0.01;
            if current_log_dt > log_dt {
                let (kinetic, gravity, lennjon, wallrep, objrepu) = energy::calculate_total_energy(&nodes, &connections_map);

                csv_energy_writer
                    .write_record(&[
                        total_symulation_time.to_string(),
                        kinetic.to_string(),
                        gravity.to_string(),
                        lennjon.to_string(),
                        wallrep.to_string(),
                        objrepu.to_string(),
                    ])
                    .unwrap();

                current_log_dt = 0.0;
            }
        }
    }

    let iterations_per_second =
        (total_symulation_time / dt) / (now.elapsed().as_millis() as f32 / 1000.0);

    csv_performance_writer
        .write_record(&[object_size.to_string(), iterations_per_second.to_string()])
        .unwrap();
}