use rand::Rng;
use ray_tracing::algebra::Vector3d;


fn main() {
    let mut rng = rand::thread_rng();

    for _ in 0..100 {
        let random_vector = Vector3d {
            x: rng.gen_range(-1.0..1.0),
            y: rng.gen_range(-1.0..1.0),
            z: rng.gen_range(-1.0..1.0),
        };
        println!("{}", &random_vector * &random_vector);
    }
}
