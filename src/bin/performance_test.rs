use mylib::{simulation::{manager::{SimulationSettings, SimulationEngineEnum}, self}};


const SIMULATION_DT: f32 = 0.00002;
const SIMULATION_DURATION: f32 = 0.5;

const OBJECT_SIZES: [usize; 10] = [5, 10, 15, 25, 40, 50, 60, 75, 90, 100];

fn main() {

    let iterations = SIMULATION_DURATION/SIMULATION_DT;
    println!("Iterations: {}", iterations);

    let mut csv_writer = {
        let log_path = format!("data/performance_test_3obj.tmp.csv");
        csv::Writer::from_path(log_path).unwrap()
    };

    for object_size in OBJECT_SIZES.iter() {
        let scene = mylib::scene::three_squares::generate(*object_size);
        let node_count = scene.nodes.len();

        let simulation_settings = SimulationSettings {
            dt: SIMULATION_DT,
            steps_per_frame: 5,
            engine: SimulationEngineEnum::Cpu,
            use_grid: true,
            cell_size: scene.object_repulsion_dx * 2.5,
            log_to_csv: false,
            log_interval: 0.05,
            use_backup: false,
            backup_interval: 0.1,
            use_auto_dt: false,
            auto_dt_factor: 1.1,
        };


        let mut simulation_manager = simulation::manager::SimulationManager::new(simulation_settings, scene);
        simulation_manager.settings = simulation_settings;

        let timer_start = std::time::Instant::now();

        while simulation_manager.total_simulation_time < SIMULATION_DURATION {
            simulation_manager.update();
        }

        let elapsed_ms = timer_start.elapsed().as_millis();
        let iterations_per_second: usize = ((iterations as f32) / (elapsed_ms as f32) * 1000.0) as usize;
        println!("{}\t{}\t{}\t{:.0}", object_size, node_count, elapsed_ms, iterations_per_second);


        csv_writer
            .write_record(&[node_count.to_string(), iterations_per_second.to_string()])
            .unwrap();
    }
}