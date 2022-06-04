use glium::{VertexBuffer, IndexBuffer, PolygonMode, Surface};
use crate::graphics::{self, Vertex};
use crate::simulation::manager::SimulationManager;
use crate::window::{RenderingSettings};

pub struct SceneRenderer {
    node_vertex_buffer: VertexBuffer<graphics::Vertex>,
    node_index_buffer: IndexBuffer<u16>,
    node_program: glium::Program,
    connection_program: glium::Program,
    grid_program: glium::Program,
}

impl SceneRenderer {
    
    pub fn new(display: &glium::Display) -> SceneRenderer {
        let (node_vertex_buffer, node_index_buffer) = SceneRenderer::create_node_buffers(display);

        SceneRenderer { 
            node_vertex_buffer, 
            node_index_buffer,
            node_program: SceneRenderer::create_node_program(display),
            connection_program: SceneRenderer::create_connections_program(display),
            grid_program: SceneRenderer::create_grid_program(display),
        }
    }

    pub fn render(&self, display: &glium::Display, target: &mut glium::Frame, settings: &RenderingSettings, screen_ratio: f32, simulation_manager: &SimulationManager) {
        
        // draw floor
        {
            let floor_params = glium::DrawParameters {
                depth: glium::Depth {
                    test: glium::DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                polygon_mode: PolygonMode::Line,
                line_width: Some(10.0),
                ..Default::default()
            };
    
            const FLOOR_HEIGHT: f32 = -0.96;
            let floor_verticies: Vec<Vertex> = vec![Vertex {local_position: [-1.5, FLOOR_HEIGHT]}, Vertex {local_position: [1.5, FLOOR_HEIGHT]}];
            let floor_vertex_buffer = glium::VertexBuffer::immutable(display, &floor_verticies).unwrap();
            target.draw(
                &floor_vertex_buffer, 
                &glium::index::NoIndices(glium::index::PrimitiveType::LinesList), 
                &self.connection_program, 
                &glium::uniform! {
                    screen_ratio: screen_ratio,
                    zoom: settings.zoom,
                    camera_position: settings.camera_position.to_array()
                },
                &floor_params
            ).unwrap();
        }

        // draw grid
        if settings.draw_grid {
            let grid_params = glium::DrawParameters {
                depth: glium::Depth {
                    test: glium::DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                polygon_mode: PolygonMode::Line,
                line_width: Some(1.0),
                ..Default::default()
            };

            let grid_verticies = graphics::draw_grid(&simulation_manager.grid);
            let grid_vertex_buffer = glium::VertexBuffer::immutable(display, &grid_verticies).unwrap();
            target.draw(
                &grid_vertex_buffer, 
                &glium::index::NoIndices(glium::index::PrimitiveType::LinesList), 
                &self.grid_program, 
                &glium::uniform! {
                    screen_ratio: screen_ratio,
                    zoom: settings.zoom,
                    camera_position: settings.camera_position.to_array()
                },
                &grid_params
            ).unwrap();
        }


        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        if settings.draw_connections {
            let connections_params = glium::DrawParameters {
                depth: glium::Depth {
                    test: glium::DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                polygon_mode: PolygonMode::Line,
                line_width: Some(2.5),
                ..Default::default()
            };

            let connections_vertex_buffer = glium::VertexBuffer::dynamic(
                display,
                &graphics::draw_connections_2(
                    &simulation_manager.scene.connections,
                    &simulation_manager.scene.nodes,
                ),
            )
            .unwrap();

            target
                .draw(
                    &connections_vertex_buffer,
                    &glium::index::NoIndices(glium::index::PrimitiveType::LinesList),
                    &self.connection_program,
                    &glium::uniform! {
                        screen_ratio: screen_ratio,
                        zoom: settings.zoom,
                        camera_position: settings.camera_position.to_array()
                    },
                    &connections_params,
                )
                .unwrap();
        }

        if settings.draw_nodes {
            let instance_buffer = glium::VertexBuffer::dynamic(
                display,
                &graphics::draw_disks(
                    &simulation_manager.scene,
                    &simulation_manager.connections_structure,
                    &settings.coloring_mode,
                    simulation_manager.last_step_dt(),
                ),
            )
            .unwrap();

            target
                .draw(
                    (&self.node_vertex_buffer, instance_buffer.per_instance().unwrap()),
                    &self.node_index_buffer,
                    &self.node_program,
                    &glium::uniform! {
                        screen_ratio: screen_ratio,
                        zoom: settings.zoom,
                        camera_position: settings.camera_position.to_array()
                    },
                    &params,
                )
                .unwrap();
        }
    }


    fn create_node_buffers(
        display: &glium::Display,
    ) -> (
        glium::VertexBuffer<graphics::Vertex>,
        glium::IndexBuffer<u16>,
    ) {
        let (disk_verticies, disk_indices) = graphics::disk_mesh(12);
        // let (disk_verticies, disk_indices) = graphics::square_mesh();
        let vertex_buffer = glium::VertexBuffer::immutable(display, &disk_verticies).unwrap();
        let index_buffer = glium::IndexBuffer::immutable(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &disk_indices,
        )
        .unwrap();
    
        (vertex_buffer, index_buffer)
    }
    
    fn create_node_program(display: &glium::Display) -> glium::Program {
        let vertex_shader_src = std::fs::read_to_string("glsl/nodes.vert").unwrap();
        let fragment_shader_src = std::fs::read_to_string("glsl/basic_coloring.frag").unwrap();
    
        glium::Program::from_source(display, &vertex_shader_src, &fragment_shader_src, None).unwrap()
    }
    
    fn create_grid_program(display: &glium::Display) -> glium::Program {
        let vertex_shader_src = std::fs::read_to_string("glsl/grid.vert").unwrap();
        let fragment_shader_src = std::fs::read_to_string("glsl/basic_coloring.frag").unwrap();
    
        glium::Program::from_source(display, &vertex_shader_src, &fragment_shader_src, None).unwrap()
    }

    fn create_connections_program(display: &glium::Display) -> glium::Program {
        let vertex_shader_src = std::fs::read_to_string("glsl/connections.vert").unwrap();
        let fragment_shader_src = std::fs::read_to_string("glsl/basic_coloring.frag").unwrap();
    
        glium::Program::from_source(display, &vertex_shader_src, &fragment_shader_src, None).unwrap()
    }
}