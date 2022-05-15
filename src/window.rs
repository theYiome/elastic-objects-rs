use std::ops::RangeInclusive;

use glam::Vec2;
use glium::glutin::event_loop;
use glium::{glutin, Surface};
use glutin::event::ElementState;
use glutin::{
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

use crate::graphics;
use crate::scene::Scene;
use crate::simulation;
use crate::rendering;

#[derive(PartialEq)]
enum SimulationEngine {
    Cpu,
    CpuMultithread,
    CpuMultithreadSingleKernel,
    OpenCl,
    None,
}

pub struct SimulationSettings {
    pub dt: f32,
    pub steps_per_frame: u32,
    engine: SimulationEngine,
    pub use_grid: bool,
    pub cell_size: f32,
}

pub struct RenderingSettings {
    pub coloring_mode: graphics::ColoringMode,
    gui_active: bool,
    pub draw_nodes: bool,
    pub draw_connections: bool,
    pub draw_grid: bool,
    pub zoom: f32,
    pub camera_position: Vec2,
}


const USE_GRID: bool = false;

pub fn run_with_gui(mut scene: Scene) {
    let mut simulation_settings = SimulationSettings {
        dt: 0.0,
        steps_per_frame: 5,
        engine: SimulationEngine::None,
        use_grid: USE_GRID,
        cell_size: simulation::general::OBJECT_REPULSION_DX * 2.5
    };

    let mut rendering_settings = RenderingSettings {
        coloring_mode: graphics::ColoringMode::KineticEnergy,
        gui_active: true,
        draw_nodes: true,
        draw_connections: true,
        draw_grid: true,
        zoom: 0.55,
        camera_position: Vec2::new(0.0, 0.0),
    };

    let mut connections_structure = simulation::general::calculate_connections_structure(&scene.connections, &scene.nodes);
    let mut grid = simulation::general::Grid::new(&scene.nodes, simulation_settings.cell_size);
    // let mut collisions_structure = simulation::general::calculate_collisions_structure_with_grid(&scene.nodes, &grid);
    let mut collisions_structure = simulation::general::calculate_collisions_structure_simple(&scene.nodes);

    let initial_window_width: u32 = 1280;
    let initial_window_height: u32 = 720;
    let event_loop = glutin::event_loop::EventLoop::new();
    let display = {
        let wb = glutin::window::WindowBuilder::new()
            .with_inner_size(glutin::dpi::LogicalSize {
                width: initial_window_width,
                height: initial_window_height,
            })
            .with_title("rover-controller-app-rs");

        let cb = glutin::ContextBuilder::new().with_depth_buffer(24);

        glium::Display::new(wb, cb, &event_loop).unwrap()
    };

    let mut egui = egui_glium::EguiGlium::new(&display);
    let scene_renderer = rendering::SceneRenderer::new(&display);
    // loging to csv file
    // let log_path = "data/log.csv";
    // let mut csv_writer = csv::Writer::from_path(log_path).unwrap();

    let mut total_symulation_time: f32 = 0.0;
    let mut current_log_dt = 0.0;
    let mut current_fps: u32 = 0;
    let mut fps_counter: u32 = 0;

    #[cfg(feature = "rust-gpu-tools")]
    let mut opencl_simulation_engine =  {
        let mut engine = simulation::gpu::gpu::SimulationEngine::new();
        engine.update_node_buffer(&scene.nodes);
        engine.update_connection_buffer(&connections_structure);
        engine.update_collision_buffer(&collisions_structure);
        engine
    };

    // let mut objects_interactions: HashMap<u32, Vec<usize>> = simulation::general::calculate_objects_interactions_structure(&mut scene.nodes);

    let mut now = std::time::Instant::now();
    let mut redraw_clousure = move |display: &glium::Display,
                                    egui: &mut egui_glium::EguiGlium,
                                    screen_ratio: f32,
                                    rendering_settings: &mut RenderingSettings,
                                    simulation_settings: &mut SimulationSettings| {
        
        //? simulation calculations
        {
            unsafe {
                static mut LAST_ITERATION_USE_GRID: bool = USE_GRID;
                if simulation_settings.use_grid != LAST_ITERATION_USE_GRID && simulation_settings.use_grid == false {
                    collisions_structure = simulation::general::calculate_collisions_structure_simple(&scene.nodes);
                }
                LAST_ITERATION_USE_GRID = simulation_settings.use_grid;
            }

            // check connection breaks
            if simulation::general::handle_connection_break(&mut scene.nodes, &mut scene.connections) {
                // objects_interactions = simulation::general::calculate_objects_interactions_structure(&mut scene.nodes);
                connections_structure = simulation::general::calculate_connections_structure(&scene.connections, &scene.nodes);
                #[cfg(feature = "rust-gpu-tools")]
                opencl_simulation_engine.update_connection_buffer(&connections_structure);

                if !simulation_settings.use_grid {
                    collisions_structure = simulation::general::calculate_collisions_structure_simple(&scene.nodes);
                    #[cfg(feature = "rust-gpu-tools")]
                    opencl_simulation_engine.update_collision_buffer(&collisions_structure);
                }
            }

            if simulation_settings.use_grid {
                grid = simulation::general::Grid::new(&scene.nodes, simulation_settings.cell_size);
                collisions_structure = simulation::general::calculate_collisions_structure_with_grid(&scene.nodes, &grid);
                #[cfg(feature = "rust-gpu-tools")]
                opencl_simulation_engine.update_collision_buffer(&collisions_structure);
            }
    
            match simulation_settings.engine {
                SimulationEngine::Cpu => {
                    for _i in 0..simulation_settings.steps_per_frame {
                        simulation::cpu::simulate_single_thread_cpu(
                            simulation_settings.dt,
                            &mut scene.nodes,
                            &scene.connections,
                            &collisions_structure
                        );
                    }
                }
                SimulationEngine::CpuMultithread => {
                    for _i in 0..simulation_settings.steps_per_frame {
                        simulation::cpu::simulate_multi_thread_cpu(
                            simulation_settings.dt,
                            &mut scene.nodes,
                            &connections_structure,
                            &collisions_structure
                        );
                    }
                }
                SimulationEngine::CpuMultithreadSingleKernel => {
                    for _i in 0..simulation_settings.steps_per_frame {
                        simulation::cpu::simulate_multi_thread_cpu_enchanced(
                            simulation_settings.dt,
                            &mut scene.nodes,
                            &connections_structure,
                            &collisions_structure
                        );
                    }
                }
                #[cfg(feature = "rust-gpu-tools")]
                SimulationEngine::OpenCl => {
                    for _i in 0..simulation_settings.steps_per_frame {
                        opencl_simulation_engine.simulate_opencl(
                            simulation_settings.dt,
                            &mut scene.nodes,
                        );
                    }
                }
                _ => {}
            }
        };
        let last_frame_symulation_time = simulation_settings.dt * simulation_settings.steps_per_frame as f32;

        //? logging and analitics
        {
            total_symulation_time += last_frame_symulation_time;
    
            fps_counter += 1;
            {
                let update_every_ms: u32 = 500;
                if now.elapsed().as_millis() > update_every_ms as u128 {
                    now = std::time::Instant::now();
                    current_fps = fps_counter * (1000 / update_every_ms);
                    fps_counter = 0;
                }
            }
    
            current_log_dt += last_frame_symulation_time;
            // {
            //     let log_dt = 0.01;
            //     if current_log_dt > log_dt {
            //         let (kinetic, gravity, lennjon, wallrep, objrepu) =
            //             simulation::energy::calculate_total_energy(&nodes, &connections_map);
    
            //         csv_writer
            //             .write_record(&[
            //                 total_symulation_time.to_string(),
            //                 (current_fps * steps_per_frame).to_string(),
            //                 kinetic.to_string(),
            //                 gravity.to_string(),
            //                 lennjon.to_string(),
            //                 wallrep.to_string(),
            //                 objrepu.to_string(),
            //             ])
            //             .unwrap();
    
            //         current_log_dt = 0.0;
            //     }
            // }
        }

        //? drawing objects and gui
        {
            // create egui interface
            egui.begin_frame(&display);
            draw_rendering_settings(egui, rendering_settings);
            draw_simulation_settings(egui, current_fps, simulation_settings);
            let (_needs_repaint, egui_shapes) = egui.end_frame(&display);
    
            let mut target = display.draw();
            // draw things behind egui here
            target.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);
            
            // draw scene
            scene_renderer.render(&display, &mut target, &scene, &grid, &rendering_settings, &simulation_settings, screen_ratio, &connections_structure);

            // draw egui
            if rendering_settings.gui_active {
                egui.paint(&display, &mut target, egui_shapes);
            }
    
            target.finish().unwrap();
        }
    };

    let mut is_mouse_dragging = false;
    let mut screen_ratio: f32 = initial_window_width as f32 / initial_window_height as f32;
    let mut window_width: f32 = initial_window_width as f32;

    let main_loop = move |event: Event<()>,
                          _: &event_loop::EventLoopWindowTarget<()>,
                          control_flow: &mut ControlFlow| {
        let mut redraw = || {
            redraw_clousure(
                &display,
                &mut egui,
                screen_ratio,
                &mut rendering_settings,
                &mut simulation_settings,
            )
        };

        match event {
            Event::RedrawEventsCleared if cfg!(windows) => redraw(),
            Event::RedrawRequested(_) if !cfg!(windows) => redraw(),

            Event::WindowEvent { event, .. } => {
                if !egui.on_event(&event) {
                    match event {
                        WindowEvent::CloseRequested {} => {
                            *control_flow = glutin::event_loop::ControlFlow::Exit;
                        }
                        WindowEvent::KeyboardInput {
                            device_id: _,
                            input,
                            is_synthetic: _,
                        } => {
                            if input.virtual_keycode == Some(VirtualKeyCode::F1)
                                && input.state == ElementState::Pressed
                            {
                                rendering_settings.gui_active = !rendering_settings.gui_active;
                            }
                        }
                        WindowEvent::MouseWheel {
                            device_id: _,
                            delta,
                            phase,
                            modifiers: _,
                        } => {
                            match delta {
                                glutin::event::MouseScrollDelta::LineDelta(x, y) => {
                                    rendering_settings.zoom += (x + y) * 0.05;
                                }
                                glutin::event::MouseScrollDelta::PixelDelta(a) => {
                                    println!("PixelDelta {}", a.to_logical::<f32>(1.0).y);
                                    rendering_settings.zoom += a.to_logical::<f32>(1.0).y * 0.05;
                                }
                            }
                            if rendering_settings.zoom < 0.1 {
                                rendering_settings.zoom = 0.1
                            };
                        }
                        WindowEvent::MouseInput {
                            device_id: _,
                            state,
                            button,
                            modifiers: _,
                        } => match button {
                            glutin::event::MouseButton::Left => {
                                match state {
                                    ElementState::Pressed => is_mouse_dragging = true,
                                    ElementState::Released => is_mouse_dragging = false,
                                };
                            }
                            _ => {}
                        },
                        WindowEvent::Resized(size) => {
                            screen_ratio = size.width as f32 / size.height as f32;
                            window_width = size.width as f32;
                        }
                        _ => (),
                    }
                }
                display.gl_window().window().request_redraw();
            }
            Event::DeviceEvent {
                device_id: _,
                event,
            } => match event {
                glutin::event::DeviceEvent::MouseMotion { delta } => {
                    if is_mouse_dragging {
                        let drag_scale: f32 = 1.0 / (window_width * 0.5);
                        rendering_settings.camera_position.x += delta.0 as f32 * drag_scale;
                        rendering_settings.camera_position.y += -delta.1 as f32 * drag_scale * screen_ratio;
                    }
                }
                _ => {}
            },
            Event::RedrawRequested { .. } => {
                redraw();
                display.gl_window().window().request_redraw();
            }
            Event::MainEventsCleared => {
                redraw();
                display.gl_window().window().request_redraw();
            }
            _ => (),
        }
    };

    // do execute main loop clousure
    event_loop.run(main_loop);
}

fn draw_rendering_settings(egui: &mut egui_glium::EguiGlium, rendering_settings: &mut RenderingSettings) {
    egui::Window::new("Rendering settings").show(egui.ctx(), |ui| {
        ui.label("Press F1 to hide/show this menu");
        ui.separator();
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut rendering_settings.coloring_mode,
                graphics::ColoringMode::KineticEnergy,
                "Kinetic Energy",
            );
            ui.selectable_value(
                &mut rendering_settings.coloring_mode,
                graphics::ColoringMode::Boundary,
                "Boundary nodes",
            );
        });
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut rendering_settings.coloring_mode,
                graphics::ColoringMode::Temperature,
                "Temperature",
            );
            ui.selectable_value(
                &mut rendering_settings.coloring_mode,
                graphics::ColoringMode::Pressure,
                "Pressure",
            );
        });
        ui.separator();
        // checkboxes for settings.draw
        ui.horizontal(|ui| {
            ui.checkbox(&mut rendering_settings.draw_connections, "Draw connections");
            ui.checkbox(&mut rendering_settings.draw_nodes, "Draw nodes");
            ui.checkbox(&mut rendering_settings.draw_grid, "Draw grid");
        });
    });
}


fn draw_simulation_settings(egui: &mut egui_glium::EguiGlium, current_fps: u32, simulation_settings: &mut SimulationSettings) {
    egui::Window::new("Simulation settings").show(egui.ctx(), |ui| {
        ui.label("Press F1 to hide/show this menu");
        ui.separator();
        ui.label(format!("FPS: {}", current_fps));
        ui.separator();
        ui.label("dt");
        ui.add(egui::Slider::new(
            &mut simulation_settings.dt,
            RangeInclusive::new(0.0, 0.00005),
        ));
        ui.separator();
        ui.label("Grid size");
        ui.add(egui::Slider::new(
            &mut simulation_settings.cell_size,
            RangeInclusive::new(0.02, 0.3),
        ));
        ui.separator();
        ui.label("Symulation steps per frame");
        ui.add(egui::Slider::new(
            &mut simulation_settings.steps_per_frame,
            RangeInclusive::new(0, 100),
        ));
        ui.separator();
        ui.label("Simulation Engine");
        ui.selectable_value(
            &mut simulation_settings.engine,
            SimulationEngine::None,
            "Stop simulation",
        );
        ui.label("CPU");
        ui.selectable_value(
            &mut simulation_settings.engine,
            SimulationEngine::Cpu,
            "Single threaded",
        );
        ui.label("Multi threaded");
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut simulation_settings.engine,
                SimulationEngine::CpuMultithread,
                "Multiple kernels",
            );
            ui.selectable_value(
                &mut simulation_settings.engine,
                SimulationEngine::CpuMultithreadSingleKernel,
                "Single kernel",
            );
        });
        #[cfg(feature = "rust-gpu-tools")]
        {
            ui.separator();
            ui.label("GPU");
            ui.selectable_value(&mut simulation_settings.engine, SimulationEngine::OpenCl, "OpenCL");
        }
        ui.separator();
        ui.horizontal(|ui| {
            ui.checkbox(&mut simulation_settings.use_grid, "Use grid");
        });
    });
}
