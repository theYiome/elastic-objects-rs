use mylib::scene::Scene;

fn main() {

    let args: Vec<String> = std::env::args().collect();
    let scene_path = {
        if args.len() < 2 {
            format!["scenes/default.bincode"]
        }
        else {
            format!("scenes/{}.bincode", args[1])
        }
    };

    println!("Trying to read scene from file: {}", scene_path);
    
    let f2 = std::fs::File::open(scene_path).unwrap();
    let decoded_scene: Scene = bincode::deserialize_from(f2).unwrap();

    mylib::window::run_with_gui(decoded_scene);
}