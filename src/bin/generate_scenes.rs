use mylib::scene::Scene;
use rayon::prelude::*;

fn main() {

    let mut scenes_to_generate: Vec<(Scene, String)> = vec![
        (mylib::scene::default::generate(), "default".to_string()),
        (mylib::scene::scene01::generate(), "scene01".to_string()),
        (mylib::scene::scene02::generate(), "scene02".to_string()),
        (mylib::scene::scene03::generate(), "scene03".to_string()),
        (mylib::scene::scene04::generate(), "scene04".to_string()),
        (mylib::scene::scene05::generate(), "scene05".to_string()),
    ];

    let object_sizes = [3, 5, 9, 13, 15, 19, 21, 25, 30, 35, 40, 45, 50, 55, 60];
    let mut multiple: Vec<(Scene, String)> = object_sizes.par_iter().map(|size| {
        (
            mylib::scene::two_squares::generate(*size),
            format!("2_{size}x{size}")
        )
    }).collect();

    scenes_to_generate.append(&mut multiple);

    scenes_to_generate.par_iter().for_each(|(scene, name)| {
        let f = std::fs::File::create(format!("scenes/{}.bincode", name)).unwrap();
        bincode::serialize_into(f, &scene).unwrap();
    });

    // let f2 = std::fs::File::open("scenes/default.bincode").unwrap();
    // let decoded: Scene = bincode::deserialize_from(f2).unwrap();
    // let encoded: Vec<u8> = bincode::serialize(&scene).unwrap();
    // let mut decoded: Scene = bincode::deserialize(&encoded[..]).unwrap();
    // assert_eq!(scene, decoded);
}