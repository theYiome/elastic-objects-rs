mod build_scene;
mod graphics;
mod scene;
mod simulation;

// use scenes::performance_test;
// use scenes::standard;

fn main() {
    scene::standard::run_with_animation();

    // let object_sizes = [3, 5, 9, 13, 15, 19, 21, 25, 30, 35, 40, 45, 50, 55, 60];
    // for size in object_sizes {
    //     scenes::performance_test::run_performace_test(size, 0.0001, 100.0);
    //     scenes::performance_test::run_performace_test_optimized(size, 0.0001, 100.0);
    // }
}