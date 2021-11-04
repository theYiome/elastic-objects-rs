mod elastic_node;
mod graphics;
mod build_scene;

use std::ops::RangeInclusive;

use glium::glutin::event_loop;
use glutin::{event::{Event, WindowEvent}, event_loop::ControlFlow};
use glium::{glutin, Surface};

fn main() {
    let mut vert: Vec<graphics::Vertex> = Vec::new();
    let mut ind: Vec<u16> = Vec::new();

    let mut nodes = build_scene::build_object(6, 3, 0.2, -0.5, -0.5);

    let mut nodes2 = build_scene::build_object(3, 2, 0.15, 0.5, 0.5);

    let v = -1.6;
    for n in &mut nodes2 {
        n.velocity.x += v;
        n.velocity.y += v;
    }

    nodes.append(&mut nodes2);


    let connections = build_scene::build_connections(&nodes, 0.3);


    for n in &nodes {
        graphics::add_circle(&mut vert, &mut ind, n.position.x, n.position.y, 0.05, 16);
    }
    
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_inner_size(glutin::dpi::LogicalSize::new(800, 800));
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    let vertex_buffer = glium::VertexBuffer::dynamic(&display, &vert).unwrap();
    let index_buffer = glium::IndexBuffer::dynamic(&display, glium::index::PrimitiveType::TrianglesList, &ind).unwrap();
    
    let vertex_shader_src = std::fs::read_to_string("glsl/vertex.vert").unwrap();
    let fragment_shader_src = std::fs::read_to_string("glsl/fragment.frag").unwrap();
    
    let program = glium::Program::from_source(&display, &vertex_shader_src, &fragment_shader_src, None).unwrap();
    
    let mut t: f32 = 0.0;

    let mut egui = egui_glium::EguiGlium::new(&display);

    let mut now = std::time::Instant::now();
    let mut last_frame_time = std::time::Instant::now();

    let mut fps_counter = 0;

    let mut redraw_clousure = move |display: &glium::Display, egui: &mut egui_glium::EguiGlium| {
        let measured_dt = last_frame_time.elapsed().as_secs_f32();
        let dt = if measured_dt > 0.02 {0.02} else {measured_dt};
        for i in 0..100 {
            build_scene::simulate(dt * 0.001, &mut nodes, &connections);
        }

        vert.clear();
        ind.clear();
        for n in &nodes {
            graphics::add_circle(&mut vert, &mut ind, n.position.x, n.position.y, 0.04, 16);
        }
        vertex_buffer.write(&vert);
        index_buffer.write(&ind);

        last_frame_time = std::time::Instant::now();
        
        fps_counter += 1;
        if now.elapsed().as_millis() > 1000 {
            println!("FPS: {}", fps_counter);
            now = std::time::Instant::now();
            fps_counter = 0;
        }
        
        // create egui interface
        egui.begin_frame(&display);
        egui::Window::new("My Window").show(egui.ctx(), |ui| {
            ui.label("Hello World!");
            ui.add(egui::Slider::new(&mut t, RangeInclusive::new(0.1, 5.2)));
        });
        let (_needs_repaint, egui_shapes) = egui.end_frame(&display);
        
        let mut target = display.draw();
        // draw things behind egui here
        target.clear_color(1.0, 1.0, 1.0, 1.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &glium::uniform! {tim: t}, &Default::default()).unwrap();

        // draw egui
        egui.paint(&display, &mut target, egui_shapes);

        // draw things on top of egui here
        target.finish().unwrap();
    };

    let main_loop = move |event: Event<()>, _: &event_loop::EventLoopWindowTarget<()>, control_flow: &mut ControlFlow| {
        match event {

            glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw_clousure(&display, &mut egui),
            glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw_clousure(&display, &mut egui),

            glutin::event::Event::WindowEvent { event, .. } => {
                egui.on_event(&event);
                match event {
                    WindowEvent::CursorEntered {..} => (),
                    WindowEvent::CloseRequested { } => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }
                    _ => ()
                }
                display.gl_window().window().request_redraw();
            },
            glutin::event::Event::RedrawRequested {..} => {
                redraw_clousure(&display, &mut egui);
                display.gl_window().window().request_redraw();
            },
            Event::MainEventsCleared => {
                redraw_clousure(&display, &mut egui);
                display.gl_window().window().request_redraw();
            }
            _ => ()
        }
    };

    // do execute main loop clousure
    event_loop.run(main_loop);
}
