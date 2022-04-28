use glam::Vec2;
use super::node::Node;
use rayon::prelude::*;

fn force_dot_position(
    nodes: &[Node],
    connections_structure: &[Vec<(usize, f32, f32)>]
) -> Vec<f32> {
    let mut forces: Vec<Vec2> = vec![Vec2::new(0.0, 0.0); nodes.len()];

    nodes.iter().enumerate().for_each(|(i, n)| {
        connections_structure[i].iter().for_each(|(j, dx, v0)| {
            let dir = nodes[*j].position - n.position;
            let l = dir.length();
    
            let c = (dx / l).powi(7) - (dx / l).powi(13);
            let v = dir.normalize() * 3.0 * (v0 / dx) * c;
    
            forces[i] += v;
        });
    });

    let v0 = super::general::WALL_REPULSION_V0;
    let dx = super::general::WALL_REPULSION_DX;

    nodes.iter().enumerate().for_each(|(index, n)| {
        let dir = glam::vec2(n.position.x, -1.0) - n.position;
        let l = dir.length();

        let c = (dx / l).powi(13);
        let v = dir.normalize() * 3.0 * (v0 / dx) * c;

        forces[index] -= v;
    });

    forces
        .iter()
        .enumerate()
        .map(|(i, f)| f.dot(nodes[i].position))
        .collect()
}


pub fn cached_avg_temperature_per_node(
    nodes: &[Node],
    connections_structure: &[Vec<(usize, f32, f32)>],
    dt: f32
) -> Vec<f32> {
    const TEMPERATURE_CACHE_SIZE: usize = 500;
    const RECORD_INTERVAL: f32 = 0.0005;
    const TOTAL_DT: f32 = TEMPERATURE_CACHE_SIZE as f32 * RECORD_INTERVAL;

    static mut TEMPERATURE_CACHE: Vec<Vec<f32>> = Vec::new();
    static mut CURRENT_RECORD: usize = 0;
    static mut CURRENT_DT: f32 = 0.0;
    unsafe {
        TEMPERATURE_CACHE.resize(nodes.len(), vec![0.0; TEMPERATURE_CACHE_SIZE]);
        TEMPERATURE_CACHE
            .iter_mut()
            .for_each(|cache| cache.resize(TEMPERATURE_CACHE_SIZE, 0.0));

        CURRENT_DT += dt;

        if CURRENT_DT > RECORD_INTERVAL {
            CURRENT_RECORD = (CURRENT_RECORD + 1) % TEMPERATURE_CACHE_SIZE;
            let current_temperature = force_dot_position(nodes, connections_structure);
            TEMPERATURE_CACHE
                .iter_mut()
                .enumerate()
                .for_each(|(node_index, cache)| {
                    cache[CURRENT_RECORD] = current_temperature[node_index];
                });
            CURRENT_DT = 0.0;
        }
    }

    let energy: Vec<f32> = nodes
        .iter()
        .enumerate()
        .map(|(i, _n)| unsafe { -0.5 * TEMPERATURE_CACHE[i].iter().copied().sum::<f32>() / TOTAL_DT })
        .collect();

    let avg_per_node: Vec<f32> = energy.par_iter().enumerate().map(|(i, _n)| {
        let mut sum = energy[i];
        let mut node_count: usize = 0;
        connections_structure[i].iter().for_each(|&(j, _dx, _v0)| {
            connections_structure[j].iter().for_each(|&(k, _dx, _v0)| {
                connections_structure[k].iter().for_each(|&(m, _dx, _v0)| {
                    connections_structure[m].iter().for_each(|&(l, _dx, _v0)| {
                        sum += energy[l];
                        node_count += 1;
                    });
                    sum += energy[m];
                    node_count += 1;
                });
                sum += energy[k];
                node_count += 1;
            });
            sum += energy[j];
            node_count += 1;
        });
        sum / (node_count as f32 + 1.0)
    }).collect();

    avg_per_node
}