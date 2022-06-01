pub fn simulate_multi_thread_cpu(
    dt: f32,
    scene: &mut Scene,
    connections_structure: &[Vec<(usize, f32, f32)>],
    collisions_structure: &[Vec<usize>]
) {
    start_integrate_velocity_verlet(dt, &mut scene.nodes);
    
    connections_multithreaded(&mut scene.nodes, connections_structure);
    repulsion_multithreaded(scene, collisions_structure);
    wall_repulsion_force(&mut scene.nodes);
    gravity_force(&mut scene.nodes);
    drag_force(&mut scene.nodes);

    end_integrate_velocity_verlet(dt, &mut scene.nodes);
}
