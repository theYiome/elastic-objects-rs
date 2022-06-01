pub fn simulate_multi_thread_cpu_single(
    dt: f32,
    scene: &mut Scene,
    connections_structure: &[Vec<(usize, f32, f32)>],
    collisions_structure: &[Vec<usize>]
) {
    
    start_integrate_velocity_verlet(dt, nodes);

    // Obliczenie przyspieszenie/masa 
    // dla kazdego wezla jako tablica dwuwymiarowych wektorow
    let acceleration_diff: Vec<Vec2> = nodes.par_iter().map(|n| {

        let connections: Vec2 = connection_force(connections_structure, n, scene);
        let repulsion  : Vec2 = collision_force(collisions_structure, n, scene);
        let wall       : Vec2 = wall_force(n);
        let drag       : Vec2 = drag_force(n);
        let gravity    : Vec2 = gravity_force(n);

        let mut result = (connections - repulsion - wall_repulsion) / n.mass;

        result += drag;
        result += gravity;
        
        return result;
    }).collect();

    // Dodanie obliczonych przyspieszen
    nodes.iter_mut().enumerate().for_each(|(i, n)| {
        n.current_acceleration += acceleration_diff[i];
    });

    end_integrate_velocity_verlet(dt, nodes);
}