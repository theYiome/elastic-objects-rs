mod elastic_node;
mod graphics;
mod build_scene;

use glium::glutin::event_loop;
use glutin::{event::{Event, WindowEvent}, event_loop::ControlFlow};
use glium::{glutin, Surface};

fn main2() {
    let mut vert: Vec<graphics::Vertex> = Vec::new();
    let mut ind: Vec<u16> = Vec::new();

    let mut nodes = build_scene::build_object(8, 8, 0.1, -0.5, -0.5);

    let mut nodes2 = build_scene::build_object(6, 4, 0.1, 0.3, 0.7);

    let mut nodes3 = build_scene::build_object(3, 3, 0.1, -0.3, 0.7);

    let v = 0.0;
    for n in &mut nodes2 {
        n.velocity.x += v;
        n.velocity.y += v;
    }

    nodes.append(&mut nodes2);
    nodes.append(&mut nodes3);


    let connections = build_scene::build_connections(&nodes, 0.11);


    for n in &nodes {
        graphics::add_circle(&mut vert, &mut ind, n.position.x, n.position.y, 0.1, 16, [0.0, 0.0, 0.0]);
    }
    
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_inner_size(glutin::dpi::LogicalSize::new(800, 800)).with_title("elastic-objects-rs");
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    let vertex_buffer = glium::VertexBuffer::dynamic(&display, &vert).unwrap();
    let index_buffer = glium::IndexBuffer::dynamic(&display, glium::index::PrimitiveType::TrianglesList, &ind).unwrap();
    
    let vertex_shader_src = std::fs::read_to_string("glsl/vertex.vert").unwrap();
    let fragment_shader_src = std::fs::read_to_string("glsl/fragment.frag").unwrap();
    
    let program = glium::Program::from_source(&display, &vertex_shader_src, &fragment_shader_src, None).unwrap();
    
    let mut dt: f32 = 0.0;
    let mut steps_per_frame: i32 = 100;
    let mut current_fps: i32 = 0;

    let mut egui = egui_glium::EguiGlium::new(&display);

    let mut now = std::time::Instant::now();
    // let mut last_frame_time = std::time::Instant::now();

    let mut fps_counter = 0;

    let mut redraw_clousure = move |display: &glium::Display, egui: &mut egui_glium::EguiGlium| {
        // let measured_dt = last_frame_time.elapsed().as_secs_f32();
        // last_frame_time = std::time::Instant::now();
        // let dt = if measured_dt > 0.01 {0.01} else {measured_dt};
        // let dt = 0.005;
        for _i in 0..steps_per_frame {
            // println!("{}", dt);
            build_scene::simulate(dt, &mut nodes, &connections);
        }

        vert.clear();
        ind.clear();
        for n in &nodes {
            graphics::add_circle(&mut vert, &mut ind, n.position.x, n.position.y, 0.04, 16, [0.0, 0.0, 0.0]);
        }
        vertex_buffer.write(&vert);
        index_buffer.write(&ind);

        
        fps_counter += 1;
        if now.elapsed().as_millis() > 1000 {
            // println!("FPS: {}", fps_counter);
            now = std::time::Instant::now();
            current_fps = fps_counter;
            fps_counter = 0;
        }
        
        // create egui interface
        egui.begin_frame(&display);
        egui::Window::new("Parametry symulacji").show(egui.ctx(), |ui| {
            ui.label(format!("FPS: {}", current_fps));
            ui.label("dt");
            ui.add(egui::Slider::new(&mut dt, RangeInclusive::new(0.0, 0.00001)));
            ui.label("Kroki symulacji na klatkę");
            ui.add(egui::Slider::new(&mut steps_per_frame, RangeInclusive::new(0, 1000)));
        });
        let (_needs_repaint, egui_shapes) = egui.end_frame(&display);
        
        let mut target = display.draw();
        // draw things behind egui here
        target.clear_color(1.0, 1.0, 1.0, 1.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &glium::uniform! {tim: dt}, &Default::default()).unwrap();

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


use core::sync::atomic::Ordering::Relaxed;
use std::{ops::RangeInclusive, sync::atomic::AtomicI32};
use rayon::prelude::*;

#[derive(Debug)]
pub struct Node {
    value: i32,
    other_value: AtomicI32,
}

#[derive(Default, Clone, Copy)]
pub struct Node2 {
    value: i32,
    other_value: i32,
}

fn repulsion_force_atomic_par(object: &mut [Node]) {
    for i in 0..object.len() {
        let (left, right) = object.split_at_mut(i + 1);
        let mut node_i = &mut left[i];
        right.iter_mut().par_bridge().for_each(|node_j| {
            let mi = 2 * node_i.value;
            let mj = mi + node_j.value;
            let c = (mi * mj + 1).pow(7);
            node_i.other_value.fetch_sub(mi / c, Relaxed);
            node_j.other_value.fetch_add(mj / c, Relaxed);
        });
    }
}


fn repulsion_force_loops(object: &mut Vec<Node2>) {
    for i in 0..object.len() {
        let mut node_i = object[i];

        for j in i+1..object.len() {
            let mut node_j = object[j];

            let mi = 2 * node_i.value;
            let mj = mi + node_j.value;
            let c = (mi * mj + 1).pow(7);

            node_i.other_value -= mi / c;
            node_j.other_value += mj / c;
        }
    }
}

fn repulsion_force_foreach(object: &mut Vec<Node2>) {
    for i in 0..object.len() {
        let (left, right) = object.split_at_mut(i + 1);
        let mut node_i = &mut left[i];

        right.iter_mut().for_each(|node_j| {
            let mi = 2 * node_i.value;
            let mj = mi + node_j.value;
            let c = (mi * mj + 1).pow(7);
            node_i.other_value -= mi / c;
            node_j.other_value += mj / c;
        });
    }
}

fn repulsion_force_naive_par(object: &mut Vec<Node2>) {

    let length = object.len();
    let obj2 = object.clone();

    object.par_iter_mut()
        .enumerate()
        .for_each(|(i, node_i)| {
            (0..length).for_each(|j| {
                if j != i {
                    let node_j = obj2[j];

                    let mi = 2 * node_i.value;
                    let mj = mi + node_j.value;
                    let c = (mi * mj + 1).pow(7);
                    node_i.other_value -= mi / c;
                }
            });
        });
}


fn main() {
    use std::time::Instant;
    
    let mut object: Vec<Node> = (0..10000).map(|k| Node { value: k, other_value: AtomicI32::new(k) }).collect();
    let mut object2: Vec<Node2> = (0..10000).map(|k| Node2 { value: k, other_value: k }).collect();
    
    let now = Instant::now();
    {
        repulsion_force_atomic_par(&mut object);
    }
    let elapsed = now.elapsed();
    println!("repulsion_force_atomic_par: {:.2?}", elapsed);


    let now = Instant::now();
    {
        repulsion_force_loops(&mut object2);
    }
    let elapsed = now.elapsed();
    println!("repulsion_force_loops: {:.2?}", elapsed);

    let now = Instant::now();
    {
        repulsion_force_foreach(&mut object2);
    }
    let elapsed = now.elapsed();
    println!("repulsion_force_foreach: {:.2?}", elapsed);


    let now = Instant::now();
    {
        repulsion_force_naive_par(&mut object2);
    }
    let elapsed = now.elapsed();
    println!("repulsion_force_naive_par: {:.2?}", elapsed);
}