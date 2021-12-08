use rand::Rng;
use ray_tracing::algebra::Vector3d;
use itertools::{self, Itertools};


fn main() {
    let a = (0..2).combinations_with_replacement(3).collect_vec();
    println!("{:?}", a);
}
