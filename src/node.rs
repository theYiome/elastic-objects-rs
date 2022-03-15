use glam::Vec2;
use std::fmt;

#[derive(Default, Clone, Copy)]
pub struct Node {
    pub position: Vec2,
    pub velocity: Vec2,
    pub last_acceleration: Vec2,
    pub current_acceleration: Vec2,
    pub mass: f32,
    pub drag: f32,
    pub object_id: u32,
    pub is_boundary: bool
}

impl Node {
    pub fn distance(a: &Node, b: &Node) -> f32 {
        return (b.position - a.position).length();
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "({}, {}, {})", self.position, self.velocity, self.mass);
    }
}