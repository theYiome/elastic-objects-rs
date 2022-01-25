mod elastic_node;
mod graphics;
mod build_scene;

use std::{ops::RangeInclusive, fs::File, collections::HashMap};

use glium::glutin::event_loop;
use glutin::{event::{Event, WindowEvent}, event_loop::ControlFlow};
use glium::{glutin, Surface};

fn run_with_animation() {
     // scene objects
     let mut vert: Vec<graphics::Vertex> = Vec::new();
     let mut ind: Vec<u16> = Vec::new();
 
     
     let mut objects: Vec<Vec<usize>> = Vec::new();
     let mut nodes = build_scene::build_nodes(8, 8, 0.1, -0.5, -0.7);
 
     {
         let mut obj: Vec<usize> = Vec::new();
         for i in 0..64 {
             obj.push(i);
         }
         objects.push(obj);
     }
     
     {
         let mut nodes2 = build_scene::build_nodes(6, 4, 0.1, -0.4, 0.2);
         nodes.append(&mut nodes2);
         {
             let mut obj: Vec<usize> = Vec::new();
             for i in 64..88 {
                 obj.push(i);
             }
             objects.push(obj);
         }
         // let mut nodes3 = build_scene::build_object(3, 3, 0.1, -0.3, 0.7);
         // nodes.append(&mut nodes3);
     }
 
     let connections = build_scene::build_connections(&nodes, 0.11);
     let connections_map = build_scene::build_connections_2(&nodes, 0.15);
 
 
     // graphics and window creation
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
 
     let mut redraw_clousure = move |display: &glium::Display, egui: &mut egui_glium::EguiGlium| {
         // let measured_dt = last_frame_time.elapsed().as_secs_f32();
         // last_frame_time = std::time::Instant::now();
         // let dt = if measured_dt > 0.01 {0.01} else {measured_dt};
         // let dt = 0.005;
         for _i in 0..steps_per_frame {
             build_scene::simulate_2(dt, &mut nodes, &mut objects, &connections_map);
         }
         total_symulation_time += dt * steps_per_frame as f32;
         
         
         vert.clear();
         ind.clear();
         for n in &nodes {
             graphics::add_circle(&mut vert, &mut ind, n.position.x, n.position.y, 0.04, 16, [0.0, 0.0, 0.0]);
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
                 let (kinetic, gravity, lennjon, wallrep, objrepu) = calculate_energy_2(&nodes, &connections_map, &objects);
                 
                 csv_writer.write_record(&[
                     total_symulation_time.to_string(), 
                     (current_fps * steps_per_frame).to_string(), 
                     kinetic.to_string(), 
                     gravity.to_string(), 
                     lennjon.to_string(),
                     wallrep.to_string(),
                     objrepu.to_string()
                 ]).unwrap();
 
                 current_log_dt = 0.0;
             }
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


fn run_performace_test(object_size: usize, dt: f32, simulation_time: f32) {

    let spacing = 0.6 / object_size as f32;

    let mut objects: Vec<Vec<usize>> = Vec::new();
    let mut nodes = build_scene::build_nodes(object_size, object_size, spacing, -0.5, -0.7);

    let end_of_first = object_size * object_size;
    let end_of_second = object_size * object_size * 2;

    {
        let mut obj: Vec<usize> = Vec::new();
        for i in 0..end_of_first {
            obj.push(i);
        }
        objects.push(obj);
    }
    
    {
        let mut nodes2 = build_scene::build_nodes(object_size, object_size, spacing, -0.4, 0.2);
        nodes.append(&mut nodes2);
        {
            let mut obj: Vec<usize> = Vec::new();
            for i in end_of_first..end_of_second {
                obj.push(i);
            }
            objects.push(obj);
        }
    }

    let connections_map = build_scene::build_connections_2(&nodes, spacing * 1.1);

    // loging to csv file
    let log_path_energy = format!("data/{}x{}_energy.csv", object_size, object_size);
    let mut csv_energy_writer = csv::Writer::from_path(log_path_energy).unwrap();

    let mut performance_test_file = std::fs::OpenOptions::new().write(true).append(true).open("data/performance_test.csv").unwrap();
    let mut csv_performance_writer = csv::Writer::from_writer(performance_test_file);


    let mut total_symulation_time: f32 = 0.0;
    let mut current_log_dt = 0.0;
    
    let mut now = std::time::Instant::now();
    
    let mut steps_per_frame: u32 = 100;
    
    while total_symulation_time < simulation_time / (object_size * object_size) as f32  {
        
        for _i in 0..steps_per_frame {
            build_scene::simulate_2(dt, &mut nodes, &mut objects, &connections_map);
        }

        total_symulation_time += dt * steps_per_frame as f32;

        current_log_dt += dt * steps_per_frame as f32;
        {
            let log_dt = 0.01;
            if current_log_dt > log_dt {
                let (kinetic, gravity, lennjon, wallrep, objrepu) = calculate_energy_2(&nodes, &connections_map, &objects);
                
                csv_energy_writer.write_record(&[
                    total_symulation_time.to_string(),
                    kinetic.to_string(), 
                    gravity.to_string(), 
                    lennjon.to_string(),
                    wallrep.to_string(),
                    objrepu.to_string()
                ]).unwrap();

                current_log_dt = 0.0;
            }
        }

    }

    let iterations_per_second =  (total_symulation_time / dt) / (now.elapsed().as_millis() as f32 / 1000.0);

    csv_performance_writer.write_record(&[
        object_size.to_string(),
        iterations_per_second.to_string()
    ]).unwrap();
}


fn count_neighbours(connections_map: HashMap<(usize, usize), f32>, node_count: usize) -> Vec<usize> {
    let mut counts: Vec<usize> = Vec::new();
    counts.resize_with(node_count, || {0});
    connections_map.keys().for_each(|(i, j)| {
        counts[*i] += 1;
        counts[*j] += 1;
    });

    counts
}

fn get_boundary_nodes(nodes: &Vec<elastic_node::Node>, search_distance: f32, offset: usize) -> Vec<usize> {
    let mut counts: Vec<usize> = Vec::new();
    counts.resize_with(nodes.len(), || {0});

    for i in 0..nodes.len() {

        for j in 0..nodes.len() {
            if i == j {
                continue;
            };

            if elastic_node::Node::distance(&nodes[i], &nodes[j]) < search_distance {
                counts[i] += 1;
            }
        }
    }

    let mut bonudary_nodes: Vec<usize> = Vec::new();

    for i in 0..nodes.len() {
        if counts[i] < 4 {
            bonudary_nodes.push(i + offset)
        }
    }

    bonudary_nodes
}


fn run_performace_test_optimized(object_size: usize, dt: f32, simulation_time: f32) {

    let spacing = 0.6 / object_size as f32;

    let mut objects: Vec<Vec<usize>> = Vec::new();

    let mut nodes = build_scene::build_nodes(object_size, object_size, spacing, -0.5, -0.7);
    objects.push(get_boundary_nodes(&nodes, spacing * 1.1, 0));
    
    
    let end_of_first = object_size * object_size;
    let end_of_second = object_size * object_size * 2;
    println!("{} -> {}", object_size, get_boundary_nodes(&nodes, spacing * 1.1, 0).len());
    
    {
        let mut nodes2 = build_scene::build_nodes(object_size, object_size, spacing, -0.4, 0.2);
        objects.push(get_boundary_nodes(&nodes2, spacing * 1.1, end_of_first));
        nodes.append(&mut nodes2);
    }

    let connections_map = build_scene::build_connections_2(&nodes, spacing * 1.1);

    let mut performance_test_file = std::fs::OpenOptions::new().write(true).append(true).open("data/performance_test_optimized.csv").unwrap();
    let mut csv_performance_writer = csv::Writer::from_writer(performance_test_file);


    let mut total_symulation_time: f32 = 0.0;
    let mut current_log_dt = 0.0;
    
    let mut now = std::time::Instant::now();
    
    let mut steps_per_frame: u32 = 100;
    
    while total_symulation_time < simulation_time / (object_size * object_size) as f32  {
        
        for _i in 0..steps_per_frame {
            build_scene::simulate_2(dt, &mut nodes, &mut objects, &connections_map);
        }

        total_symulation_time += dt * steps_per_frame as f32;

        current_log_dt += dt * steps_per_frame as f32;
        {
            let log_dt = 0.01;
            if current_log_dt > log_dt {
                let (kinetic, gravity, lennjon, wallrep, objrepu) = calculate_energy_2(&nodes, &connections_map, &objects);

                current_log_dt = 0.0;
            }
        }

    }

    let iterations_per_second =  (total_symulation_time / dt) / (now.elapsed().as_millis() as f32 / 1000.0);

    csv_performance_writer.write_record(&[
        object_size.to_string(),
        iterations_per_second.to_string()
    ]).unwrap();
}



fn main() {
    // run_with_animation();

    let object_sizes = [3, 5, 9, 13, 15, 19, 21, 25, 30, 35, 40, 45, 50, 55, 60];
    for size in object_sizes {
        run_performace_test(size, 0.0001, 100.0);
        run_performace_test_optimized(size, 0.0001, 100.0);
    }
}


fn calculate_energy_2(nodes: &Vec<elastic_node::Node>, connections: &HashMap<(usize, usize), f32>, objects: &Vec<Vec<usize>>) -> (f32, f32, f32, f32, f32) {
    let mut total_kinetic: f32 = 0.0;
    let mut total_gravity: f32 = 0.0;
    let mut total_lennjon: f32 = 0.0;
    let mut total_wallrep: f32 = 0.0;
    let mut total_objrepu: f32 = 0.0;

    nodes.iter().enumerate().for_each(|(i, n1)| {
        total_kinetic += n1.velocity.length_squared() * n1.mass * 0.5;
        total_gravity += n1.mass * 9.81 * (n1.position.y + 0.5);
    });

    let v0 = 100.0;
    connections.keys().for_each(|(a, b)| {
        let n1 = &nodes[*a];
        let n2 = &nodes[*b];
        let dist = (n2.position - n1.position).length();
        let dx = *connections.get(&(*a, *b)).unwrap();

        let m01 = 1.12246204831;
        let sigma = (1.0 / m01) * dx;
        let inner = sigma / dist;

        total_lennjon += v0 * (inner.powf(12.0) - inner.powf(6.0));
    });

    nodes.iter().for_each(|n| {
        let v0 = 200.0;
        let dx = 0.05;

        let dist = (-1.05 - n.position.y).abs();
        let m01 = 1.12246204831;
        let sigma = (1.0 / m01) * dx;
        let inner = sigma / dist;

        total_wallrep += v0 * inner.powf(12.0);
    });


    let length = objects.len();
    for i in 0..length {
        for j in i + 1..length {
            // calculate energy between each node in object "i" and object "j"
            for n_i in &objects[i] {
                for n_j in &objects[j] {
                    let a = *n_i;
                    let b = *n_j;

                    let v0 = 20.0;
                    let dx = 0.1;
            
                    let dist = (nodes[b].position - nodes[a].position).length();
                    let m01 = 1.12246204831;
            
                    let sigma = (1.0 / m01) * dx;
                    let inner = sigma / dist;
            
                    total_objrepu += v0 * inner.powf(12.0);
                }
            }
        }
    }

    (total_kinetic, total_gravity, total_lennjon, total_wallrep, total_objrepu)
}