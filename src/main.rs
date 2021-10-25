use glam::Vec2;

mod elastic_node;
use elastic_node::Node;
use glium::glutin::event_loop;

fn build_object(size_x: usize, size_y: usize, spacing: f32, offset_x: f32, offset_y: f32) -> Vec<Node> {
    let mut object = Vec::with_capacity(size_x * size_y);
    for y in 0..size_y {
        for x in 0..size_x {
            object.push(Node {
                position: Vec2::new(offset_x + (x as f32) * spacing, offset_y + (y as f32) * spacing),
                velocity: Vec2::new(0.0, 0.0),
                mass: 1.0
            });
        }
    }
    return object;
}

fn build_connections(object: &Vec<Node>, search_distance: f32) -> Vec<Vec<usize>> {
    let mut connections: Vec<Vec<usize>> = Vec::new();
    for i in 0..object.len() {

        let mut row: Vec<usize> = Vec::new();

        for j in 0..object.len() {
            if i == j { continue };

            if Node::distance(&object[i], &object[j]) < search_distance {
                row.push(j);
            }
        }

        connections.push(row);
    }
    return connections;
}


#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    col: [f32; 3],
}

glium::implement_vertex!(Vertex, position, col);

fn main() {
    // let object = build_object(5, 3, 0.1, 0.0, 0.0);
    // let connections = build_connections(&object, 0.2);
    // for o in object.iter() {
    //     println!("{}", o);
    // }

    let vertex1 = Vertex { position: [-0.5, -0.5], col: [0.1, 0.2, 0.9] };
    let vertex2 = Vertex { position: [ 0.0,  0.5], col: [0.0, 0.9, 0.1] };
    let vertex3 = Vertex { position: [ 0.5, -0.25], col: [0.99, 0.1, 0.2]};
    let shape = vec![vertex1, vertex2, vertex3];

    
    use glium::{glutin, Surface};
    
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
    
    let vertex_shader_src = std::fs::read_to_string("glsl/vertex.glsl").unwrap();
    let fragment_shader_src = std::fs::read_to_string("glsl/fragment.glsl").unwrap();

    let program = glium::Program::from_source(&display, &vertex_shader_src, &fragment_shader_src, None).unwrap();

    event_loop.run(move |ev, _, control_flow| {

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        target.draw(&vertex_buffer, &indices, &program, &glium::uniforms::EmptyUniforms,&Default::default()).unwrap();
        target.finish().unwrap();

        let next_frame_time = std::time::Instant::now() +
            std::time::Duration::from_nanos(16_666_667);

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
    });
    
}
