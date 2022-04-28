use std::collections::HashMap;
use std::ops::RangeInclusive;

// use crate::energy;
use crate::simulation;
use crate::graphics;

#[cfg(feature = "rust-gpu-tools")]
use crate::simulation_gpu;

use glam::Vec2;
use glium::glutin::event_loop;
use glium::{glutin, Surface};
use glutin::event::ElementState;
use glutin::{
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

use super::generate_scene::standard_scene;

#[derive(PartialEq)]
enum SimulationEngine {
    Cpu,
    CpuMultithread,
    OpenCl,
    Cuda,
    None
}

pub fn run_with_animation() {
    // scene objects

    let (mut nodes, mut connections_map) = standard_scene();
    let mut connections_structure = simulation::general::calculate_connections_structure(&connections_map, &nodes);

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

    let vertex_shader_src = std::fs::read_to_string("glsl/vertex.vert").unwrap();
    let fragment_shader_src = std::fs::read_to_string("glsl/fragment.frag").unwrap();

    let program =
        glium::Program::from_source(&display, &vertex_shader_src, &fragment_shader_src, None)
            .unwrap();

    // loging to csv file
    let log_path = "data/log.csv";
    // let mut log_file = File::create(log_path).unwrap();
    // let mut reader = csv::Reader::from_path(log_path).unwrap();
    let mut csv_writer = csv::Writer::from_path(log_path).unwrap();

    let mut dt: f32 = 0.0;
    let mut total_symulation_time: f32 = 0.0;
    let mut current_log_dt = 0.0;

    let mut egui = egui_glium::EguiGlium::new(&display);

    let mut now = std::time::Instant::now();
    // let mut last_frame_time = std::time::Instant::now();

    let mut steps_per_frame: u32 = 5;
    let mut current_fps: u32 = 0;
    let mut fps_counter: u32 = 0;

    let mut current_simulation_engine = SimulationEngine::None;
    let mut current_coloring_mode = graphics::ColoringMode::KineticEnergy;

    // prepare opencl and cuda programs
    #[cfg(feature = "rust-gpu-tools")]
    let opencl_program = simulation_gpu::gpu::create_opencl_program();

    let (disk_verticies, disk_indices) = graphics::disk_mesh(12);
    // let (disk_verticies, disk_indices) = graphics::square_mesh();
    let disk_vertex_buffer = glium::VertexBuffer::immutable(&display, &disk_verticies).unwrap();
    let disk_index_buffer = glium::IndexBuffer::immutable(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &disk_indices,
    )
    .unwrap();

    let mut objects_interactions: HashMap<u32, Vec<usize>> =
        simulation::general::calculate_objects_interactions_structure(&mut nodes);

    let mut redraw_clousure = move |display: &glium::Display,
                                    egui: &mut egui_glium::EguiGlium,
                                    egui_active: bool,
                                    zoom: &mut f32,
                                    camera_position: Vec2,
                                    screen_ratio: f32| {
        // real time dependent dt
        {
            // let measured_dt = last_frame_time.elapsed().as_secs_f32();
            // last_frame_time = std::time::Instant::now();
            // dt = if measured_dt > 0.01 {0.01} else {measured_dt};
            // dt = 0.005;
        }

        match simulation::general::handle_connection_break(&mut nodes, &mut connections_map) {
            Some(x) => {
                objects_interactions = x;
                connections_structure = simulation::general::calculate_connections_structure(&connections_map, &nodes);
            }
            None => {}
        }

        match current_simulation_engine {
            SimulationEngine::Cpu => {
                for _i in 0..steps_per_frame {
                    simulation::cpu::simulate_single_thread_cpu(
                        dt,
                        &mut nodes,
                        &connections_map,
                        &objects_interactions,
                    );
                }
            }
            SimulationEngine::CpuMultithread => {
                for _i in 0..steps_per_frame {
                    simulation::cpu::simulate_multi_thread_cpu(
                        dt,
                        &mut nodes,
                        &connections_structure,
                        &objects_interactions,
                    );
                }
            },
            #[cfg(feature = "rust-gpu-tools")]
            SimulationEngine::OpenCl => {
                nodes = simulation::gpu::simulate_opencl(
                    &nodes,
                    &opencl_program,
                    &connections_map,
                    steps_per_frame,
                    dt,
                );
            },
            _ => {}
        }

        let last_frame_symulation_time = dt * steps_per_frame as f32;
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
        //             energy::calculate_total_energy(&nodes, &connections_map);

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

        // create egui interface
        egui.begin_frame(&display);
        egui::Window::new("General settings").show(egui.ctx(), |ui| {
            ui.label("Press F1 to hide/show this menu");
            ui.label(format!("FPS: {}", current_fps));
            ui.label("Zoom");
            ui.add(egui::Slider::new(zoom, RangeInclusive::new(0.1, 2.0)));
            ui.label("dt");
            ui.add(egui::Slider::new(&mut dt, RangeInclusive::new(0.0, 0.00005)));
            ui.label("Symulation steps per frame");
            ui.add(egui::Slider::new(
                &mut steps_per_frame,
                RangeInclusive::new(0, 100),
            ));
            ui.separator();
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut current_simulation_engine,
                    SimulationEngine::Cpu,
                    "CPU single threaded",
                );
                ui.selectable_value(
                    &mut current_simulation_engine,
                    SimulationEngine::CpuMultithread,
                    "CPU multi threaded",
                );
            });
            ui.selectable_value(
                &mut current_simulation_engine,
                SimulationEngine::None,
                "Stop simulation",
            );
            ui.separator();
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut current_coloring_mode,
                    graphics::ColoringMode::KineticEnergy,
                    "Kinetic Energy",
                );
                ui.selectable_value(
                    &mut current_coloring_mode,
                    graphics::ColoringMode::Boundary,
                    "Boundary nodes",
                );
            });
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut current_coloring_mode,
                    graphics::ColoringMode::Temperature,
                    "Temperature",
                );
                ui.selectable_value(
                    &mut current_coloring_mode,
                    graphics::ColoringMode::Pressure,
                    "Pressure",
                );
            });
        });
        let (_needs_repaint, egui_shapes) = egui.end_frame(&display);

        let mut target = display.draw();
        // draw things behind egui here
        target.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

        let instance_buffer = glium::VertexBuffer::dynamic(
            display,
            &graphics::draw_disks(
                &nodes,
                &connections_structure,
                &current_coloring_mode,
                last_frame_symulation_time,
            ),
        )
        .unwrap();

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        target
            .draw(
                (&disk_vertex_buffer, instance_buffer.per_instance().unwrap()),
                &disk_index_buffer,
                &program,
                &glium::uniform! {
                    screen_ratio: screen_ratio,
                    zoom: *zoom,
                    camera_position: camera_position.to_array()
                },
                &params,
            )
            .unwrap();

        // draw egui
        if egui_active {
            egui.paint(&display, &mut target, egui_shapes);
        }

        // draw things on top of egui here
        target.finish().unwrap();
    };

    let mut zoom: f32 = 0.55;
    let mut egui_active = true;
    let mut is_mouse_dragging = false;
    let mut camera_position: Vec2 = Vec2::new(0.0, 0.0);
    let mut screen_ratio: f32 = initial_window_width as f32 / initial_window_height as f32;
    let mut window_width: f32 = initial_window_width as f32;

    let main_loop = move |event: Event<()>,
                          _: &event_loop::EventLoopWindowTarget<()>,
                          control_flow: &mut ControlFlow| {
        let mut redraw = || {
            redraw_clousure(
                &display,
                &mut egui,
                egui_active,
                &mut zoom,
                camera_position,
                screen_ratio,
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
                                egui_active = !egui_active;
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
                                    zoom += (x + y) * 0.05;
                                }
                                glutin::event::MouseScrollDelta::PixelDelta(a) => {
                                    println!("PixelDelta {}", a.to_logical::<f32>(1.0).y);
                                    zoom += a.to_logical::<f32>(1.0).y * 0.05;
                                }
                            }
                            if zoom < 0.1 {
                                zoom = 0.1
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
                        camera_position.x += delta.0 as f32 * drag_scale;
                        camera_position.y += -delta.1 as f32 * drag_scale * screen_ratio;
                    }
                },
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
