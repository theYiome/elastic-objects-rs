mod elastic_node;
mod graphics;

use glium::glutin::event_loop;
use glutin::{event::Event, event_loop::ControlFlow};
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

    let main_loop = move |ev: Event<()>, _: &event_loop::EventLoopWindowTarget<()>, control_flow: &mut ControlFlow| {

        let mut target = display.draw();
        target.clear_color(1.0, 1.0, 1.0, 1.0);

        target.draw(&vertex_buffer, &index_buffer, &program, &glium::uniform! {tim: t.sin()},&Default::default()).unwrap();
        target.finish().unwrap();

        let next_frame_time = std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        t += 0.1;

        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
        match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                _ => return,
            },
            _ => (),
        }
    };

    event_loop.run(main_loop);
    
}
