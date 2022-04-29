use std::collections::HashMap;
use crate::simulation::node::Node;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Scene {
    nodes: Vec<Node>,
    connections: HashMap<(usize, usize), (f32, f32)>
}


// pub mod performance_test;
pub mod standard;
pub mod generate_scene;
pub mod performance_test;