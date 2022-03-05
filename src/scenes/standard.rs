use std::collections::HashMap;
use std::ops::RangeInclusive;

use crate::{build_scene, energy, graphics, simulation_cpu};

use glium::glutin::event_loop;
use glium::{glutin, Surface};
use glutin::event::{KeyboardInput, ElementState};
use glutin::{
    event::{Event, WindowEvent, VirtualKeyCode},
    event_loop::ControlFlow,
};

pub fn run_with_animation() {
    // scene objects

    let mut objects: Vec<Vec<usize>> = Vec::new();

    let object1_sx = 24;
    let object1_sy = 12;
    let object1_st = object1_sx * object1_sy;
    let spacing1 = 0.08;

    let object2_sx = 4;
    let object2_sy = 4;
    let object2_st = object2_sx * object2_sy;
    let object2_m = 15.0;
    let spacing2 = 0.075;

    let mut nodes1 = build_scene::build_rectangle(object1_sx, object1_sy, spacing1, -0.92, -0.925, 1.0, 5.0);
    let mut connections_map_1 = build_scene::build_connections_map(&nodes1, spacing1 * 1.5, 70.0, 0);
    {
        let mut obj: Vec<usize> = Vec::new();
        for i in 0..object1_st {
            obj.push(i);
        }
        objects.push(obj);
    }

    let mut nodes2 = build_scene::build_circle(4, spacing2, -0.12, 0.8, 5.0, 0.0);
        // build_scene::build_rectangle(object2_sx, object2_sy, spacing2, -0.12, 0.8, object2_m, 1.0);
    
    let connections_map_2 = build_scene::build_connections_map(&nodes2, spacing2 * 1.5, 500.0, object1_st);
    {
        let mut obj: Vec<usize> = Vec::new();
        for i in object1_st..object1_st + nodes2.len() {
            obj.push(i);
        }
        objects.push(obj);
    }

    let mut full_connections_map: HashMap<(usize, usize), (f32, f32)> = HashMap::new();
    full_connections_map.extend(connections_map_1);
    full_connections_map.extend(connections_map_2);

    let mut connections_keys: Vec<(u32, u32)> = Vec::new();
    let mut connections_vals: Vec<(f32, f32)> = Vec::new();
    for (k1, k2) in &full_connections_map {
        connections_keys.push((k1.0 as u32, k1.1 as u32));
        connections_vals.push(*k2);
    }

    let mut nodes = Vec::new();
    nodes.append(&mut nodes1);
    nodes.append(&mut nodes2);

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(glutin::dpi::LogicalSize::new(800, 800))
        .with_title("elastic-objects-rs");
    let cb = glutin::ContextBuilder::new();
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

    // Then we run it on OpenCL.
    // let device = *Device::all().first().unwrap();
    // let opencl_program = opencl(&device);

    let mut redraw_clousure = move |display: &glium::Display, egui: &mut egui_glium::EguiGlium, egui_active: bool| {
        // let measured_dt = last_frame_time.elapsed().as_secs_f32();
        // last_frame_time = std::time::Instant::now();
        // let dt = if measured_dt > 0.01 {0.01} else {measured_dt};
        // let dt = 0.005;
        // nodes = simulate_opencl(
        //     &nodes,
        //     &opencl_program,
        //     &connections_keys,
        //     &connections_vals,
        //     steps_per_frame,
        //     dt,
        // );
        // println!("CPU: {}\n", mem::size_of::<Node>());

        for _i in 0..steps_per_frame {
            simulation_cpu::simulate_single_thread_cpu(
                dt,
                &mut nodes,
                &mut objects,
                &mut full_connections_map,
            );
        }
        total_symulation_time += dt * steps_per_frame as f32;

        let (vert, ind) = graphics::draw_scene(&nodes, &full_connections_map);
        let vertex_buffer = glium::VertexBuffer::dynamic(display, &vert).unwrap();
        let index_buffer = glium::IndexBuffer::dynamic(display, glium::index::PrimitiveType::TrianglesList, &ind).unwrap();

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
                    energy::calculate_total_energy(&nodes, &full_connections_map, &objects);

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
        });
        let (_needs_repaint, egui_shapes) = egui.end_frame(&display);

        let mut target = display.draw();
        // draw things behind egui here
        target.clear_color(1.0, 1.0, 1.0, 1.0);
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
