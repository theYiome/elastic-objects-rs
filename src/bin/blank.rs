use mylib::scene;

fn main() {
    mylib::window::run_with_gui(scene::three_squares::generate(10));
}