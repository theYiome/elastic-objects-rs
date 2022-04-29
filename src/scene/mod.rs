use std::collections::HashMap;
use crate::simulation::node::Node;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Scene {
    pub nodes: Vec<Node>,
    pub connections: HashMap<(usize, usize), (f32, f32)>
}

mod objects;
pub mod default;
pub mod two_squares;