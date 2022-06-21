use crate::scene::Scene;

use super::{general::Grid};
use crate::simulation;
#[cfg(feature = "opencl3")]
use crate::gpu::gpu::SimulationEngine;

#[derive(PartialEq, Clone, Copy)]
pub enum SimulationEngineEnum {
    Cpu,
    CpuMultithread,
    CpuMultithreadSingleKernel,
    OpenCl,
    None,
}

#[derive(Clone, Copy)]
pub struct SimulationSettings {
    pub dt: f32,
    pub steps_per_frame: u32,
    pub engine: SimulationEngineEnum,
    pub use_grid: bool,
    pub cell_size: f32,
    pub log_to_csv: bool,
    pub log_interval: f32,
    pub use_backup: bool,
    pub backup_interval: f32,
    pub use_auto_dt: bool,
    pub auto_dt_factor: f32,
}

pub struct SimulationManager {
    pub scene: Scene,
    pub scene_backup: Scene,
    pub current_backup_dt: f32,
    pub total_simulation_time: f32,
    pub connections_structure: Vec<Vec<(usize, f32, f32)>>,
    pub collisions_structure: Vec<Vec<usize>>,
    pub grid: Grid,
    pub settings: SimulationSettings,
    #[cfg(feature = "opencl3")] pub opencl_simulation_engine: SimulationEngine,
}

pub const MAX_DT: f32 = 0.00005;

impl SimulationManager {
    pub fn new(simulation_settings: SimulationSettings, scene: Scene) -> Self {

        let connections_structure = simulation::general::calculate_connections_structure(&scene.connections, &scene.nodes);
        let collisions_structure = simulation::general::calculate_collisions_structure_simple(&scene.nodes);
        let grid = simulation::general::Grid::new(&scene.nodes, simulation_settings.cell_size);

        #[cfg(feature = "opencl3")]
        let mut opencl_simulation_engine =  {
            let mut engine = simulation::gpu::gpu::SimulationEngine::new();
            engine.update_node_buffer(&scene.nodes);
            engine.update_connection_buffer(&connections_structure);
            engine.update_collision_buffer(&collisions_structure);
            engine
        };

        return SimulationManager {
            scene: scene.clone(),
            scene_backup: scene,
            current_backup_dt: 0.0,
            total_simulation_time: 0.0,
            connections_structure: connections_structure,
            collisions_structure: collisions_structure,
            grid: grid,
            settings: simulation_settings,
            #[cfg(feature = "opencl3")] opencl_simulation_engine: opencl_simulation_engine
        };
    }

    pub fn grid_check(&mut self) {
        if self.settings.use_grid {
            unsafe {
                static mut LAST_ITERATION_USE_GRID: bool = false;
                if self.settings.use_grid != LAST_ITERATION_USE_GRID && self.settings.use_grid == false {
                    self.collisions_structure = simulation::general::calculate_collisions_structure_simple(&self.scene.nodes);
                }
                LAST_ITERATION_USE_GRID = self.settings.use_grid;
            }
        }
    }

    pub fn connection_break(&mut self) {
        if simulation::general::handle_connection_break(&mut self.scene.nodes, &mut self.scene.connections) {
            self.connections_structure = simulation::general::calculate_connections_structure(&self.scene.connections, &self.scene.nodes);
            #[cfg(feature = "opencl3")]
            if self.settings.engine == SimulationEngineEnum::OpenCl {
                self.opencl_simulation_engine.update_connection_buffer(&self.connections_structure);
            }

            if !self.settings.use_grid {
                self.collisions_structure = simulation::general::calculate_collisions_structure_simple(&self.scene.nodes);
                #[cfg(feature = "opencl3")]
                if self.settings.engine == SimulationEngineEnum::OpenCl {
                    self.opencl_simulation_engine.update_collision_buffer(&self.collisions_structure);
                }
            }
        }
    }

    pub fn update_grid(&mut self) {
        if self.settings.use_grid {
            self.grid = simulation::general::Grid::new(&self.scene.nodes, self.settings.cell_size);
            self.collisions_structure = simulation::general::calculate_collisions_structure_with_grid(&self.scene.nodes, &self.grid);
            #[cfg(feature = "opencl3")]
            if self.settings.engine == SimulationEngineEnum::OpenCl {
                self.opencl_simulation_engine.update_collision_buffer(&self.collisions_structure);
            }
        }
    }

    pub fn next_step(&mut self) {
        match self.settings.engine {
            SimulationEngineEnum::Cpu => {
                for _i in 0..self.settings.steps_per_frame {
                    simulation::cpu::simulate_single_thread_cpu(
                        self.settings.dt,
                        &mut self.scene,
                        &self.collisions_structure
                    );
                }
            }
            SimulationEngineEnum::CpuMultithread => {
                for _i in 0..self.settings.steps_per_frame {
                    simulation::cpu::simulate_multi_thread_cpu(
                        self.settings.dt,
                        &mut self.scene,
                        &self.connections_structure,
                        &self.collisions_structure
                    );
                }
            }
            SimulationEngineEnum::CpuMultithreadSingleKernel => {
                for _i in 0..self.settings.steps_per_frame {
                    simulation::cpu::simulate_multi_thread_cpu_enchanced(
                        self.settings.dt,
                        &mut self.scene,
                        &self.connections_structure,
                        &self.collisions_structure
                    );
                }
            }
            #[cfg(feature = "opencl3")]
            SimulationEngineEnum::OpenCl => {
                for _i in 0..self.settings.steps_per_frame {
                    self.opencl_simulation_engine.simulate_opencl(
                        self.settings.dt,
                        &mut self.scene,
                    );
                }
            }
            _ => {}
        }
    }

    fn update_backup(&mut self) {
        if self.settings.use_backup {
            self.current_backup_dt += self.last_step_dt();
            if self.current_backup_dt > self.settings.backup_interval {
                self.current_backup_dt = 0.0;
                self.restore_if_broken();
            }

        }    
    }

    pub fn is_broken(&self) -> bool {
        let mut broken = false;

        for node in &self.scene.nodes {
            if !node.position.x.is_finite() || !node.position.y.is_finite() {
                broken = true;
                break;
            }
        }

        broken
    }

    pub fn restore_if_broken(&mut self) {
        if self.is_broken() {
            println!("Error detected, restoring scene");
            self.scene = self.scene_backup.clone();
            self.connections_structure = simulation::general::calculate_connections_structure(&self.scene.connections, &self.scene.nodes);
            self.grid = simulation::general::Grid::new(&self.scene.nodes, self.settings.cell_size);
            self.collisions_structure = simulation::general::calculate_collisions_structure_simple(&self.scene.nodes);
            self.settings.dt *= 0.5;
            #[cfg(feature = "opencl3")]
            if self.settings.engine == SimulationEngineEnum::OpenCl {
                self.opencl_simulation_engine.update_connection_buffer(&self.connections_structure);
                self.opencl_simulation_engine.update_collision_buffer(&self.collisions_structure);
            }
        }
        else {
            self.scene_backup = self.scene.clone();
            if self.settings.use_auto_dt {
                self.settings.dt *= self.settings.auto_dt_factor;
                if self.settings.dt > MAX_DT {
                    self.settings.dt = MAX_DT;
                }
            }
        }
    }

    pub fn last_step_dt(&self) -> f32 {
        self.settings.dt * self.settings.steps_per_frame as f32
    }

    pub fn update(&mut self) {
        self.grid_check();
        self.connection_break();
        self.update_grid();
        self.next_step();
        self.update_backup();
        
        self.total_simulation_time += self.last_step_dt();
    }

}