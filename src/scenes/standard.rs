use std::ops::RangeInclusive;

use crate::{energy, graphics, simulation_cpu, simulation_general, simulation_gpu};

use glium::glutin::event_loop;
use glium::{glutin, Surface};
use glutin::event::{ElementState};
use glutin::{
    event::{Event, WindowEvent, VirtualKeyCode},
    event_loop::ControlFlow,
};

use super::generate_scene::standard_scene;

#[derive(PartialEq)]
enum SimulationEngine {
    CPU,
    OPENCL,
    CUDA
}

pub fn run_with_animation() {
    // scene objects

    let (mut nodes, mut connections_map, mut objects) = standard_scene();

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(glutin::dpi::LogicalSize::new(800, 800))
        .with_title("elastic-objects-rs");
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

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

    let mut steps_per_frame: u32 = 100;
    let mut current_fps: u32 = 0;
    let mut fps_counter: u32 = 0;

    let mut current_simulation_engine = SimulationEngine::CUDA;

    // prepare opencl and cuda programs
    let device = *rust_gpu_tools::Device::all().first().unwrap();
    let opencl_program = simulation_gpu::create_opencl_program(&device);

    let mut redraw_clousure = move |display: &glium::Display, egui: &mut egui_glium::EguiGlium, egui_active: bool| {
        
        // real time dependent dt
        {
            // let measured_dt = last_frame_time.elapsed().as_secs_f32();
            // last_frame_time = std::time::Instant::now();
            // dt = if measured_dt > 0.01 {0.01} else {measured_dt};
            // dt = 0.005;
        }

        match current_simulation_engine {
            SimulationEngine::CPU => {
                for _i in 0..steps_per_frame {
                    simulation_cpu::simulate_single_thread_cpu(
                        dt,
                        &mut nodes,
                        &mut objects,
                        &mut connections_map
                    );
                }
            },
            SimulationEngine::OPENCL => {
                // nodes = simulation_gpu::simulate_opencl(
                //     &nodes,
                //     &opencl_program,
                //     &connections_map,
                //     steps_per_frame,
                //     dt,
                // );
            },
            SimulationEngine::CUDA => {

            }
        }

        total_symulation_time += dt * steps_per_frame as f32;
        simulation_general::handle_connection_break(&mut nodes, &mut objects, &mut connections_map);

        fps_counter += 1;
        {
            let update_every_ms: u32 = 500;
            if now.elapsed().as_millis() > update_every_ms as u128 {
                now = std::time::Instant::now();
                current_fps = fps_counter * (1000 / update_every_ms);
                fps_counter = 0;
            }
        }

        current_log_dt += dt * steps_per_frame as f32;
        {
            let log_dt = 0.01;
            if current_log_dt > log_dt {
                let (kinetic, gravity, lennjon, wallrep, objrepu) =
                    energy::calculate_total_energy(&nodes, &connections_map, &objects);

                csv_writer
                    .write_record(&[
                        total_symulation_time.to_string(),
                        (current_fps * steps_per_frame).to_string(),
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

        // create egui interface
        egui.begin_frame(&display);
        egui::Window::new("Parametry symulacji").show(egui.ctx(), |ui| {
            ui.label(format!("FPS: {}", current_fps));
            ui.label("dt");
            ui.add(egui::Slider::new(
                &mut dt,
                RangeInclusive::new(0.0, 0.00001),
            ));
            ui.label("Kroki symulacji na klatkę");
            ui.add(egui::Slider::new(
                &mut steps_per_frame,
                RangeInclusive::new(0, 500),
            ));
            ui.separator();
            ui.horizontal(|ui| {
                ui.selectable_value(&mut current_simulation_engine, SimulationEngine::CPU, "Use CPU");
                ui.selectable_value(&mut current_simulation_engine, SimulationEngine::OPENCL, "Use OpenCL");
                ui.selectable_value(&mut current_simulation_engine, SimulationEngine::CUDA, "Use CUDA");
            });
        });
        let (_needs_repaint, egui_shapes) = egui.end_frame(&display);

        let mut target = display.draw();
        // draw things behind egui here
        target.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);
                
        let (vert, ind) = graphics::draw_scene(&nodes, &connections_map, &objects);
        let vertex_buffer = glium::VertexBuffer::dynamic(display, &vert).unwrap();
        let index_buffer = glium::IndexBuffer::dynamic(display, glium::index::PrimitiveType::TrianglesList, &ind).unwrap();
        
        target
            .draw(
                &vertex_buffer,
                &index_buffer,
                &program,
                &glium::uniform! {tim: dt},
                &Default::default(),
            )
            .unwrap();

        // draw egui
        if egui_active {
            egui.paint(&display, &mut target, egui_shapes);
        }

        // draw things on top of egui here
        target.finish().unwrap();
    };

    let main_loop = move |event: Event<()>, _: &event_loop::EventLoopWindowTarget<()>, control_flow: &mut ControlFlow| {
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
                    },
                    WindowEvent::KeyboardInput {device_id: _, input, is_synthetic: _} => {
                        if input.virtual_keycode == Some(VirtualKeyCode::F1) && input.state == ElementState::Pressed {
                            egui_active = !egui_active;
                        }
                    },
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
