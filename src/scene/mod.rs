use std::collections::HashMap;
use crate::simulation::node::Node;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Scene {
    pub nodes: Vec<Node>,
    pub connections: HashMap<(usize, usize), (f32, f32)>,
    pub object_repulsion_dx: f32,
    pub object_repulsion_v0: f32,
}

mod objects;
pub mod default;
pub mod scene01;
pub mod scene02;
pub mod scene03;
pub mod scene04;
pub mod scene05;
pub mod two_squares;
pub mod three_squares;
pub mod blank;
pub mod pressure;
pub mod pressure02;