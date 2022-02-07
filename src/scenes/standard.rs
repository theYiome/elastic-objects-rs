use std::{ops::RangeInclusive};

use crate::{build_scene, graphics, simulation_cpu, energy};

use glium::glutin::event_loop;
use glium::{glutin, Surface};
use glutin::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

pub fn run_with_animation() {
    // scene objects
    let mut vert: Vec<graphics::Vertex> = Vec::new();
    let mut ind: Vec<u16> = Vec::new();

    let mut objects: Vec<Vec<usize>> = Vec::new();

    let object1_sx = 10;
    let object1_sy = 10;
    let object1_st = object1_sx * object1_sy;

    let object2_sx = 2;
    let object2_sy = 3;
    let object2_st = object2_sx * object2_sy;
    let mut nodes = build_scene::build_nodes(object1_sx, object1_sy, 0.1, -0.5, -0.925);

    {
        let mut obj: Vec<usize> = Vec::new();
        for i in 0..object1_st {
            obj.push(i);
        }
        objects.push(obj);
    }

    {
        let mut nodes2 = build_scene::build_nodes(object2_sx, object2_sy, 0.1, -0.1, 0.8);
        nodes.append(&mut nodes2);
        {
            let mut obj: Vec<usize> = Vec::new();
            for i in object1_st..object1_st+object2_st {
                obj.push(i);
            }
            objects.push(obj);
        }
        // let mut nodes3 = build_scene::build_object(3, 3, 0.1, -0.3, 0.7);
        // nodes.append(&mut nodes3);
    }

    let connections_map = build_scene::build_connections_2(&nodes, 0.15, 1.0);
    let mut connections_keys: Vec<(u32, u32)> = Vec::new();
    let mut connections_vals: Vec<(f32, f32)> = Vec::new();
    for (k1, k2) in &connections_map {
        connections_keys.push((k1.0 as u32, k1.1 as u32));
        connections_vals.push(*k2);
    }

    // graphics and window creation
    for n in &nodes {
        graphics::add_circle(
            &mut vert,
            &mut ind,
            n.position.x,
            n.position.y,
            0.1,
            16,
            [0.0, 0.0, 0.0],
        );
    }

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(glutin::dpi::LogicalSize::new(800, 800))
        .with_title("elastic-objects-rs");
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let vertex_buffer = glium::VertexBuffer::dynamic(&display, &vert).unwrap();
    let index_buffer =
        glium::IndexBuffer::dynamic(&display, glium::index::PrimitiveType::TrianglesList, &ind)
            .unwrap();

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

    let mut steps_per_frame: u32 = 100;
    let mut current_fps: u32 = 0;
    let mut fps_counter: u32 = 0;

    // Then we run it on OpenCL.
    // let device = *Device::all().first().unwrap();
    // let opencl_program = opencl(&device);

    let mut redraw_clousure = move |display: &glium::Display, egui: &mut egui_glium::EguiGlium| {
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
            simulation_cpu::simulate_single_thread_cpu(dt, &mut nodes, &mut objects, &connections_map);
        }
        total_symulation_time += dt * steps_per_frame as f32;

        vert.clear();
        ind.clear();
        for n in &nodes {
            graphics::add_circle(
                &mut vert,
                &mut ind,
                n.position.x,
                n.position.y,
                0.04,
                16,
                [0.0, 0.0, 0.0],
            );
        }
        vertex_buffer.write(&vert);
        index_buffer.write(&ind);

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
                RangeInclusive::new(0.0, 0.0001),
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
        egui.paint(&display, &mut target, egui_shapes);

        // draw things on top of egui here
        target.finish().unwrap();
    };

    let main_loop = move |event: Event<()>,
                          _: &event_loop::EventLoopWindowTarget<()>,
                          control_flow: &mut ControlFlow| {
        match event {
            glutin::event::Event::RedrawEventsCleared if cfg!(windows) => {
                redraw_clousure(&display, &mut egui)
            }
            glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => {
                redraw_clousure(&display, &mut egui)
            }

            glutin::event::Event::WindowEvent { event, .. } => {
                egui.on_event(&event);
                match event {
                    WindowEvent::CloseRequested {} => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }
                    _ => (),
                }
                display.gl_window().window().request_redraw();
            }
            glutin::event::Event::RedrawRequested { .. } => {
                redraw_clousure(&display, &mut egui);
                display.gl_window().window().request_redraw();
            }
            Event::MainEventsCleared => {
                redraw_clousure(&display, &mut egui);
                display.gl_window().window().request_redraw();
            }
            _ => (),
        }
    };

    // do execute main loop clousure
    event_loop.run(main_loop);
}