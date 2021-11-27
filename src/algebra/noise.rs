use super::{approx_equal, Vector3d};
use itertools::{assert_equal, Itertools, MultiProduct};
use num::Float;
use rand::{
    prelude::{IteratorRandom, SliceRandom},
    thread_rng, Rng,
};

#[derive(Debug, Clone)]
pub struct Perlin {
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    perm_z: Vec<usize>,
    ranfloat: Vec<f64>,
    ranvec: Vec<Vector3d>,

    cartesian: Vec<Vec<i32>>,
}

impl Default for Perlin {
    fn default() -> Self {
        Self::new()
    }
}

impl Perlin {
    pub fn new() -> Self {
        let mut rng = thread_rng();
        let mut perm_x = (0..256).collect_vec();
        let mut perm_y = (0..256).collect_vec();
        let mut perm_z = (0..256).collect_vec();
        perm_x.shuffle(&mut rng);
        perm_y.shuffle(&mut rng);
        perm_z.shuffle(&mut rng);

        Self {
            perm_x,
            perm_y,
            perm_z,
            ranfloat: (0..256).map(|_| rng.gen()).collect_vec(),
            ranvec: (0..256).map(|_| Vector3d::random(-1.0, 1.0)).collect_vec(),
            cartesian: (0..3).map(|_| 0..2).multi_cartesian_product().collect_vec(),
        }
    }

    pub fn noise(&self, p: &Vector3d) -> f64 {
        let x = p.x.floor() as i32;
        let y = p.y.floor() as i32;
        let z = p.z.floor() as i32;

        let u = p.x - p.x.floor();
        let v = p.y - p.y.floor();
        let w = p.z - p.z.floor();

        let u2 = u * u * (3.0 - 2.0 * u);
        let v2 = v * v * (3.0 - 2.0 * v);
        let w2 = w * w * (3.0 - 2.0 * w);

        self.cartesian
            .iter()
            .map(|d| {
                let c = self.ranvec[self.perm_x[((d[0] as i32 + x) & 255) as usize]
                    ^ self.perm_y[((d[1] as i32 + y) & 255) as usize]
                    ^ self.perm_z[((d[2] as i32 + z) & 255) as usize]];

                let fi = d[0] as f64;
                let fj = d[1] as f64;
                let fk = d[2] as f64;
                (fi * u2 + (1 - d[0]) as f64 * (1.0 - u2))
                    * (fj * v2 + (1 - d[1]) as f64 * (1.0 - v2))
                    * (fk * w2 + (1 - d[2]) as f64 * (1.0 - w2))
                    * (c * Vector3d::new(u - fi, v - fj, w - fk))
            })
            .sum()
    }

    pub fn turb(&self, p: &Vector3d, depth: i32) -> f64 {
        (0..depth)
            .scan((1.0, p.clone()), |(weight, temp_p), _| {
                let ret = *weight * self.noise(&p);
                *weight *= 0.5;
                *temp_p *= 2.0;

                Some(ret)
            })
            .sum::<f64>()
            .abs()
    }

    pub fn perlin_interp(&self, c: &[[[Vector3d; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let u2 = u * u * (3.0 - 2.0 * u);
        let v2 = v * v * (3.0 - 2.0 * v);
        let w2 = w * w * (3.0 - 2.0 * w);
        let mut accum = 0.0;
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let c = c[i][j][k];
                    let fi = i as f64;
                    let fj = j as f64;
                    let fk = k as f64;
                    let weight_v = Vector3d::new(u - fi, v - fj, w - fk);
                    accum += (fi * u2 + (1 - i) as f64 * (1.0 - u2))
                        * (fj * v2 + (1 - j) as f64 * (1.0 - v2))
                        * (fk * w2 + (1 - k) as f64 * (1.0 - w2))
                        * (c * weight_v)
                }
            }
        }
        accum
    }

    pub fn trilinear_interp(&self, c: &[[[f64; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let mut accum = 0.0;
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let c = c[i][j][k];
                    let fi = i as f64;
                    let fj = j as f64;
                    let fk = k as f64;
                    accum += (fi * u + (1 - i) as f64 * (1.0 - u))
                        * (fj * v + (1 - j) as f64 * (1.0 - v))
                        * (fk * w + (1 - k) as f64 * (1.0 - w))
                        * c
                }
            }
        }
        accum
    }
}
