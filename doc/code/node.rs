#[repr(C)]
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