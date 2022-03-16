use std::collections::HashMap;
use std::ops::RangeInclusive;

use crate::{energy, graphics, simulation_cpu, simulation_general, simulation_gpu};

use glium::glutin::event_loop;
use glium::{glutin, PolygonMode, Surface};
use glutin::event::ElementState;
use glutin::{
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

use super::generate_scene::standard_scene;

#[derive(PartialEq)]
enum SimulationEngine {
    CPU,
    OPENCL,
    CUDA,
}

pub fn run_with_animation() {
    // scene objects

    let (mut nodes, mut connections_map) = standard_scene();

    let event_loop = glutin::event_loop::EventLoop::new();
    let display = {
        let wb = glutin::window::WindowBuilder::new()
            .with_inner_size(glutin::dpi::LogicalSize {
                width: 1280 as u32,
                height: 720 as u32,
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
    let mut egui_active = true;

    let mut now = std::time::Instant::now();
    // let mut last_frame_time = std::time::Instant::now();

    let mut steps_per_frame: u32 = 5;
    let mut current_fps: u32 = 0;
    let mut fps_counter: u32 = 0;

    let mut current_simulation_engine = SimulationEngine::CUDA;
    let mut current_coloring_mode = graphics::ColoringMode::KineticEnergy;

    // prepare opencl and cuda programs
    let device = *rust_gpu_tools::Device::all().first().unwrap();
    let opencl_program = simulation_gpu::create_opencl_program(&device);

    let mut zoom: f32 = 0.55;

    let (disk_verticies, disk_indices) = graphics::disk_mesh(12);
    // let (disk_verticies, disk_indices) = graphics::square_mesh();
    let disk_vertex_buffer = glium::VertexBuffer::immutable(&display, &disk_verticies).unwrap();
    let disk_index_buffer = glium::IndexBuffer::immutable(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &disk_indices,
    )
    .unwrap();
    
    let mut objects_interactions: HashMap<u32, Vec<usize>> = simulation_general::calculate_objects_interactions_structure(&mut nodes);

    let mut redraw_clousure =
        move |display: &glium::Display, egui: &mut egui_glium::EguiGlium, egui_active: bool| {
            // real time dependent dt
            {
                // let measured_dt = last_frame_time.elapsed().as_secs_f32();
                // last_frame_time = std::time::Instant::now();
                // dt = if measured_dt > 0.01 {0.01} else {measured_dt};
                // dt = 0.005;
            }

            match simulation_general::handle_connection_break(&mut nodes, &mut connections_map) {
                Some(x) => {
                    objects_interactions = x;
                },
                None => {}
            }

            match current_simulation_engine {
                SimulationEngine::CPU => {
                    for _i in 0..steps_per_frame {
                        simulation_cpu::simulate_single_thread_cpu(
                            dt,
                            &mut nodes,
                            &connections_map,
                            &objects_interactions
                        );
                    }
                }
                SimulationEngine::OPENCL => {
                    // nodes = simulation_gpu::simulate_opencl(
                    //     &nodes,
                    //     &opencl_program,
                    //     &connections_map,
                    //     steps_per_frame,
                    //     dt,
                    // );
                }
                SimulationEngine::CUDA => {}
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
                ui.label(format!("FPS: {}", current_fps));
                ui.label("Zoom");
                ui.add(egui::Slider::new(
                    &mut zoom,
                    RangeInclusive::new(0.1, 1.5),
                ));
                ui.label("dt");
                ui.add(egui::Slider::new(&mut dt, RangeInclusive::new(0.0, 0.0001)));
                ui.label("Symulation steps per frame");
                ui.add(egui::Slider::new(
                    &mut steps_per_frame,
                    RangeInclusive::new(0, 100),
                ));
                ui.separator();
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut current_simulation_engine,
                        SimulationEngine::CPU,
                        "Use CPU",
                    );
                    ui.selectable_value(
                        &mut current_simulation_engine,
                        SimulationEngine::OPENCL,
                        "Use OpenCL",
                    );
                    ui.selectable_value(
                        &mut current_simulation_engine,
                        SimulationEngine::CUDA,
                        "Use CUDA",
                    );
                });
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
                    ui.selectable_value(
                        &mut current_coloring_mode,
                        graphics::ColoringMode::Temperature,
                        "Temperature",
                    );
                });

            });
            let (_needs_repaint, egui_shapes) = egui.end_frame(&display);

            let mut target = display.draw();
            // draw things behind egui here
            target.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

            let instance_buffer = glium::VertexBuffer::dynamic(
                display,
                &graphics::draw_disks(&nodes, &connections_map, &objects_interactions, &current_coloring_mode, last_frame_symulation_time),
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

            let window_size = display.gl_window().window().inner_size();

            target
                .draw(
                    (&disk_vertex_buffer, instance_buffer.per_instance().unwrap()),
                    &disk_index_buffer,
                    &program,
                    &glium::uniform! {
                        screen_ratio: window_size.width as f32 / window_size.height as f32,
                        zoom: zoom
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

    let main_loop = move |event: Event<()>,
                          _: &event_loop::EventLoopWindowTarget<()>,
                          control_flow: &mut ControlFlow| {
        match event {
            glutin::event::Event::RedrawEventsCleared if cfg!(windows) => {
                redraw_clousure(&display, &mut egui, egui_active)
            }
            glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => {
                redraw_clousure(&display, &mut egui, egui_active)
            }

            glutin::event::Event::WindowEvent { event, .. } => {
                egui.on_event(&event);
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
                    _ => (),
                }
                display.gl_window().window().request_redraw();
            }
            glutin::event::Event::RedrawRequested { .. } => {
                redraw_clousure(&display, &mut egui, egui_active);
                display.gl_window().window().request_redraw();
            }
            Event::MainEventsCleared => {
                redraw_clousure(&display, &mut egui, egui_active);
                display.gl_window().window().request_redraw();
            }
            _ => (),
        }
    };

    // do execute main loop clousure
    event_loop.run(main_loop);
}
