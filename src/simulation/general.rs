use std::collections::HashMap;

use super::node::Node;
use glam::Vec2;
use rayon::prelude::*;

pub const OBJECT_REPULSION_V0: f32 = 10.0;
pub const OBJECT_REPULSION_DX: f32 = 0.015;

pub const WALL_REPULSION_V0: f32 = 200.0;
pub const WALL_REPULSION_DX: f32 = 0.05;

fn nodes_too_far(nodes: &mut Vec<Node>, connections: &mut HashMap<(usize, usize), (f32, f32)>) -> Vec<(usize, usize)> {

    let mut to_remove: Vec<(usize, usize)> = Vec::new();

    for (k, v) in connections.iter() {

        let (i, j) = *k;
        let (dx, _v0) = *v;

        let dir = nodes[j].position - nodes[i].position;
        let l = dir.length();
        if l > dx * 1.5 {
            to_remove.push(*k);
        }
    }

    to_remove
}

pub fn handle_connection_break(
    nodes: &mut Vec<Node>,
    connections: &mut HashMap<(usize, usize), (f32, f32)>,
) -> Option<HashMap<u32, Vec<usize>>> {

    let connections_to_break = nodes_too_far(nodes, connections);
    let recalculate_objects_interactions = connections_to_break.len() > 0;

    for k in connections_to_break {
        connections.iter().filter(|(&i, _j)| {
            i.0 == k.0 || i.0 == k.1 || i.1 == k.0 || i.1 == k.1
        }).for_each(|(i, _j)| {
            nodes[i.0].is_boundary = true;
            nodes[i.1].is_boundary = true;
        });
        connections.remove(&k);
    }

    if recalculate_objects_interactions {
        return Some(calculate_objects_interactions_structure(nodes));
    }

    return None;
}

pub fn calculate_objects_interactions_structure(nodes: &mut Vec<Node>) -> HashMap<u32, Vec<usize>> {
    let mut objects_interactions_structure: HashMap<u32, Vec<usize>> = HashMap::new();
    nodes.iter().enumerate().for_each(|(index, n)| {
        if n.is_boundary {
            let obj = objects_interactions_structure.get_mut(&n.object_id);
            match obj {
                Some(x) => {
                    x.push(index);
                }
                None => {
                    objects_interactions_structure.insert(n.object_id, vec![index]);
                }
            }
        }
    });

    objects_interactions_structure
}

pub fn calculate_connections_structure(connections_map: &HashMap<(usize, usize), (f32, f32)>, nodes: &Vec<Node>) -> Vec<Vec<(usize, f32, f32)>> {
    let mut connections_structure: Vec<Vec<(usize, f32, f32)>> = vec![Vec::new(); nodes.len()];
    connections_map.iter().for_each(|(k, v)| {
        connections_structure[k.0].push((k.1, v.0, v.1));
        connections_structure[k.1].push((k.0, v.0, v.1));
    });
    connections_structure
}

pub struct Grid {
    pub top_left: Vec2,
    pub bottom_right: Vec2,
    pub cells: Vec<Vec<Vec<usize>>>,
    pub cell_size: f32,
    pub cell_count_x: usize,
    pub cell_count_y: usize,
}

impl Grid {
    // return index of the cell that contains the point
    pub fn get_cell_index(&self, point: &Vec2) -> (usize, usize) {
        let mut x = ((point.x - self.top_left.x) / self.cell_size) as i32;
        let mut y = ((self.top_left.y - point.y) / self.cell_size) as i32;
        if x >= self.cell_count_x as i32 {
            x = self.cell_count_x as i32 - 1;
        }
        else if x < 0 {
            x = 0;
        }
        if y >= self.cell_count_y as i32 {
            y = self.cell_count_y as i32 - 1;
        }
        else if y < 0 {
            y = 0;
        }
        (x as usize, y as usize)
    }

    // return values of cells that contain the point and the cells that are adjacent to it including the cell itself and diagonals
    pub fn get_node_indexes_from_neighbours(&self, point: &Vec2) -> Vec<usize> {
        let (x, y) = self.get_cell_index(point);
        let mut neighbours: Vec<usize> = Vec::new();
        neighbours.append(&mut self.cells[x][y].clone());
        if y > 0 {
            neighbours.append(&mut self.cells[x][y - 1].clone());
        }
        if y < self.cell_count_y - 1 {
            neighbours.append(&mut self.cells[x][y + 1].clone());
        }
        if x > 0 {
            neighbours.append(&mut self.cells[x - 1][y].clone());
        }
        if x < self.cell_count_x - 1 {
            neighbours.append(&mut self.cells[x + 1][y].clone());
        }
        if x > 0 && y > 0 {
            neighbours.append(&mut self.cells[x - 1][y - 1].clone());
        }
        if x > 0 && y < self.cell_count_y - 1 {
            neighbours.append(&mut self.cells[x - 1][y + 1].clone());
        }
        if x < self.cell_count_x - 1 && y > 0 {
            neighbours.append(&mut self.cells[x + 1][y - 1].clone());
        }
        if x < self.cell_count_x - 1 && y < self.cell_count_y - 1 {
            neighbours.append(&mut self.cells[x + 1][y + 1].clone());
        }
        neighbours
    }

    pub fn new(nodes: &Vec<Node>, cell_size: f32) -> Grid {
        // get the largest and lowest position x and y from nodes
        let mut top_left = Vec2::new(std::f32::MAX, std::f32::MIN);
        let mut bottom_right = Vec2::new(std::f32::MIN, std::f32::MAX);
        nodes.iter().for_each(|n| {
            if n.position.x < top_left.x {
                top_left.x = n.position.x;
            }
            if n.position.y > top_left.y {
                top_left.y = n.position.y;
            }
            if n.position.x > bottom_right.x {
                bottom_right.x = n.position.x;
            }
            if n.position.y < bottom_right.y {
                bottom_right.y = n.position.y;
            }
        });
        
        const MAX_SIZE: f32 = 2.0;
        if top_left.x < -MAX_SIZE { top_left.x = -MAX_SIZE };
        if top_left.y > MAX_SIZE { top_left.y = MAX_SIZE };
        if bottom_right.x > MAX_SIZE { bottom_right.x = MAX_SIZE };
        if bottom_right.y < -MAX_SIZE { bottom_right.y = -MAX_SIZE };
        // let top_left = Vec2::new(-1.0, 1.0);
        // let bottom_right = Vec2::new(1.0, -1.0);
    
        let width = bottom_right.x - top_left.x;
        let height = top_left.y - bottom_right.y;
    
        let cell_count_x = (width / cell_size).ceil() as usize;
        let cell_count_y = (height / cell_size).ceil() as usize;
    
        let cells: Vec<Vec<Vec<usize>>> = vec![vec![Vec::new(); cell_count_y]; cell_count_x];
    
        let mut grid = Grid {
            top_left,
            bottom_right,
            cells,
            cell_size,
            cell_count_x,
            cell_count_y,
        };
     
        nodes.iter().enumerate().for_each(|(index, n)| {
            if n.is_boundary {
                let (x, y) = grid.get_cell_index(&n.position);
                grid.cells[x][y].push(index);
            }
        });
    
        grid
    }
}


pub fn calculate_collisions_structure_with_grid(nodes: &Vec<Node>, grid: &Grid) -> Vec<Vec<usize>> {
    nodes.par_iter().map(|n| {
        if n.is_boundary {
            grid.get_node_indexes_from_neighbours(&n.position).iter().copied().filter(|j| nodes[*j].object_id != n.object_id).collect()
        }
        else {
            vec![]
        }
    }).collect()
}

pub fn calculate_collisions_structure_simple(nodes: &Vec<Node>) -> Vec<Vec<usize>> {
    nodes.par_iter().map(|n| {
        if n.is_boundary {
            nodes.iter().enumerate().filter(|(_j, n2)| {
                n2.is_boundary && n.object_id != n2.object_id
            }).map(|(j, _n2)| j).collect()
        }
        else {
            Vec::new()
        }
    }).collect()
}