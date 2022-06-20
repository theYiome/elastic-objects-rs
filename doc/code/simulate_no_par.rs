pub fn simulate_single_thread_cpu(
    dt: f32,
    scene: &mut Scene,
    connections_structure: &[Vec<(usize, f32, f32)>],
    collisions_structure: &[Vec<usize>]
) {
    start_integrate_velocity_verlet(dt, &mut scene.nodes);

    // obliczanie nowego przyspieszenia na podstawie
    // obecnego stanu sceny
    // --------------------------------------------------
    connections(&mut scene.nodes, connections_structure);
    repulsion(scene, collisions_structure);
    wall_repulsion_force(&mut scene.nodes);
    gravity_force(&mut scene.nodes);
    drag_force(&mut scene.nodes);
    // --------------------------------------------------

    end_integrate_velocity_verlet(dt, &mut scene.nodes);
}
