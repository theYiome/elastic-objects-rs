mod elastic_node;
mod graphics;

use std::ops::RangeInclusive;

use glium::glutin::event_loop;
use glutin::{event::{Event, WindowEvent}, event_loop::ControlFlow};
use glium::{glutin, Surface};

fn main() {
    let mut vert: Vec<graphics::Vertex> = Vec::new();
    let mut ind: Vec<u16> = Vec::new();
    graphics::add_circle(&mut vert, &mut ind, 0.4, -0.1, 0.2, 3);
    graphics::add_circle(&mut vert, &mut ind, -0.4, 0.1, 0.4, 6);
    graphics::add_circle(&mut vert, &mut ind, 0.0, 0.65, 0.3, 32);
    
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_inner_size(glutin::dpi::LogicalSize::new(800, 800));
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    let vertex_buffer = glium::VertexBuffer::new(&display, &vert).unwrap();
    let index_buffer = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &ind).unwrap();
    
    let vertex_shader_src = std::fs::read_to_string("glsl/vertex.vert").unwrap();
    let fragment_shader_src = std::fs::read_to_string("glsl/fragment.frag").unwrap();
    
    let program = glium::Program::from_source(&display, &vertex_shader_src, &fragment_shader_src, None).unwrap();
    
    let mut t: f32 = 0.0;


    let mut egui = egui_glium::EguiGlium::new(&display);

    let mut now = std::time::Instant::now();
    let mut fps_counter = 0;

    let main_loop = move |event: Event<()>, _: &event_loop::EventLoopWindowTarget<()>, control_flow: &mut ControlFlow| {
        
        fps_counter += 1;
        if now.elapsed().as_millis() > 1000 {
            println!("{}", fps_counter);
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
        target.draw(&vertex_buffer, &index_buffer, &program, &glium::uniform! {tim: (t.sin() + 1.0) * 0.5 }, &Default::default()).unwrap();

        // draw egui
        egui.paint(&display, &mut target, egui_shapes);

        // draw things on top of egui here
        target.finish().unwrap();

        match event {

            glutin::event::Event::WindowEvent { event, .. } => {
                egui.on_event(&event);

                match event {
                    WindowEvent::CursorEntered {..} => (),
                    WindowEvent::CloseRequested { } => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }
                    _ => ()
                }
            },
            glutin::event::Event::RedrawRequested {..} => (),
            _ => ()
        }
    };


    // do execute main loop clousure
    event_loop.run(main_loop);
    
}
