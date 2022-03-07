mod build_scene;
mod node;
mod graphics;
mod simulation_general;
mod simulation_cpu;
mod simulation_gpu;
mod energy;

// use scenes::performance_test;
// use scenes::standard;
mod scenes;

fn main() {
    scenes::standard::run_with_animation();

    // let object_sizes = [3, 5, 9, 13, 15, 19, 21, 25, 30, 35, 40, 45, 50, 55, 60];
    // for size in object_sizes {
    //     scenes::performance_test::run_performace_test(size, 0.0001, 100.0);
    //     scenes::performance_test::run_performace_test_optimized(size, 0.0001, 100.0);
    // }
}