use std::collections::HashMap;
use std::ops::RangeInclusive;

use glam::Vec2;
use glium::glutin::event_loop;
use glium::{glutin, Surface};
use glutin::event::ElementState;
use glutin::{
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

#[cfg(feature = "rust-gpu-tools")]
use crate::simulation_gpu;

// use crate::energy;
use crate::graphics;
use crate::scene::Scene;
use crate::simulation;

#[derive(PartialEq)]
enum SimulationEngine {
    Cpu,
    CpuMultithread,
    OpenCl,
    Cuda,
    None,
}

fn create_node_buffers(
    display: &glium::Display,
) -> (
    glium::VertexBuffer<graphics::Vertex>,
    glium::IndexBuffer<u16>,
) {
    let (disk_verticies, disk_indices) = graphics::disk_mesh(12);
    // let (disk_verticies, disk_indices) = graphics::square_mesh();
    let vertex_buffer = glium::VertexBuffer::immutable(display, &disk_verticies).unwrap();
    let index_buffer = glium::IndexBuffer::immutable(
        display,
        glium::index::PrimitiveType::TrianglesList,
        &disk_indices,
    )
    .unwrap();

    (vertex_buffer, index_buffer)
}

fn create_node_program(display: &glium::Display) -> glium::Program {
    let vertex_shader_src = std::fs::read_to_string("glsl/vertex.vert").unwrap();
    let fragment_shader_src = std::fs::read_to_string("glsl/fragment.frag").unwrap();

    glium::Program::from_source(display, &vertex_shader_src, &fragment_shader_src, None).unwrap()
}

struct Settings {
    dt: f32,
    steps_per_frame: u32,
    engine: SimulationEngine,
    coloring_mode: graphics::ColoringMode,
    gui_active: bool,
    zoom: f32,
    camera_position: Vec2,
}

pub fn run_with_gui(scene: Scene) {
    let mut settings = Settings {
        dt: 0.0,
        steps_per_frame: 5,
        engine: SimulationEngine::None,
        coloring_mode: graphics::ColoringMode::KineticEnergy,
        gui_active: true,
        zoom: 0.55,
        camera_position: Vec2::new(0.0, 0.0),
    };

    let (mut nodes, mut connections_map) = (scene.nodes, scene.connections);
    let mut connections_structure =
        simulation::general::calculate_connections_structure(&connections_map, &nodes);

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
    let (disk_vertex_buffer, disk_index_buffer) = create_node_buffers(&display);
    let program = create_node_program(&display);

    // loging to csv file
    // let log_path = "data/log.csv";
    // let mut csv_writer = csv::Writer::from_path(log_path).unwrap();

    let mut total_symulation_time: f32 = 0.0;
    let mut current_log_dt = 0.0;
    let mut current_fps: u32 = 0;
    let mut fps_counter: u32 = 0;

    let mut objects_interactions: HashMap<u32, Vec<usize>> =
        simulation::general::calculate_objects_interactions_structure(&mut nodes);

    let mut now = std::time::Instant::now();
    let mut redraw_clousure = move |display: &glium::Display,
                                    egui: &mut egui_glium::EguiGlium,
                                    screen_ratio: f32,
                                    settings: &mut Settings| {
        // check connection breaks
        match simulation::general::handle_connection_break(&mut nodes, &mut connections_map) {
            Some(x) => {
                objects_interactions = x;
                connections_structure =
                    simulation::general::calculate_connections_structure(&connections_map, &nodes);
            }
            None => {}
        }

        match settings.engine {
            SimulationEngine::Cpu => {
                for _i in 0..settings.steps_per_frame {
                    simulation::cpu::simulate_single_thread_cpu(
                        settings.dt,
                        &mut nodes,
                        &connections_map,
                        &objects_interactions,
                    );
                }
            }
            SimulationEngine::CpuMultithread => {
                for _i in 0..settings.steps_per_frame {
                    simulation::cpu::simulate_multi_thread_cpu(
                        settings.dt,
                        &mut nodes,
                        &connections_structure,
                        &objects_interactions,
                    );
                }
            }
            #[cfg(feature = "rust-gpu-tools")]
            SimulationEngine::OpenCl => {
                nodes = simulation::gpu::simulate_opencl(
                    &nodes,
                    &opencl_program,
                    &connections_map,
                    steps_per_frame,
                    dt,
                );
            }
            _ => {}
        }

        let last_frame_symulation_time = settings.dt * settings.steps_per_frame as f32;
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

        // create egui interface
        egui.begin_frame(&display);
        draw_settings(egui, current_fps, settings);
        let (_needs_repaint, egui_shapes) = egui.end_frame(&display);

        let mut target = display.draw();
        // draw things behind egui here
        target.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

        let instance_buffer = glium::VertexBuffer::dynamic(
            display,
            &graphics::draw_disks(
                &nodes,
                &connections_structure,
                &settings.coloring_mode,
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
                    zoom: settings.zoom,
                    camera_position: settings.camera_position.to_array()
                },
                &params,
            )
            .unwrap();

        // draw egui
        if settings.gui_active {
            egui.paint(&display, &mut target, egui_shapes);
        }

        // draw things on top of egui here
        target.finish().unwrap();
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
                &mut settings
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
                                settings.gui_active = !settings.gui_active;
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
                                    settings.zoom += (x + y) * 0.05;
                                }
                                glutin::event::MouseScrollDelta::PixelDelta(a) => {
                                    println!("PixelDelta {}", a.to_logical::<f32>(1.0).y);
                                    settings.zoom += a.to_logical::<f32>(1.0).y * 0.05;
                                }
                            }
                            if settings.zoom < 0.1 {
                                settings.zoom = 0.1
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
                        settings.camera_position.x += delta.0 as f32 * drag_scale;
                        settings.camera_position.y += -delta.1 as f32 * drag_scale * screen_ratio;
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

fn draw_settings(egui: &mut egui_glium::EguiGlium, current_fps: u32, settings: &mut Settings) {
    egui::Window::new("General settings").show(egui.ctx(), |ui| {
        ui.label("Press F1 to hide/show this menu");
        ui.label(format!("FPS: {}", current_fps));
        ui.label("Zoom");
        ui.add(egui::Slider::new(
            &mut settings.zoom,
            RangeInclusive::new(0.1, 2.0),
        ));
        ui.label("dt");
        ui.add(egui::Slider::new(
            &mut settings.dt,
            RangeInclusive::new(0.0, 0.00005),
        ));
        ui.label("Symulation steps per frame");
        ui.add(egui::Slider::new(
            &mut settings.steps_per_frame,
            RangeInclusive::new(0, 100),
        ));
        ui.separator();
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut settings.engine,
                SimulationEngine::Cpu,
                "CPU single threaded",
            );
            ui.selectable_value(
                &mut settings.engine,
                SimulationEngine::CpuMultithread,
                "CPU multi threaded",
            );
            #[cfg(feature = "rust-gpu-tools")]
            ui.selectable_value(&mut settings.engine, SimulationEngine::OpenCl, "GPU OpenCL");
        });
        ui.selectable_value(
            &mut settings.engine,
            SimulationEngine::None,
            "Stop simulation",
        );
        ui.separator();
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut settings.coloring_mode,
                graphics::ColoringMode::KineticEnergy,
                "Kinetic Energy",
            );
            ui.selectable_value(
                &mut settings.coloring_mode,
                graphics::ColoringMode::Boundary,
                "Boundary nodes",
            );
        });
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut settings.coloring_mode,
                graphics::ColoringMode::Temperature,
                "Temperature",
            );
            ui.selectable_value(
                &mut settings.coloring_mode,
                graphics::ColoringMode::Pressure,
                "Pressure",
            );
        });
    });
}
