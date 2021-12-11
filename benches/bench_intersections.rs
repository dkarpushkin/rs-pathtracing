use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ray_tracing::{
    algebra::{transform::InversableTransform, Vector3d},
    world::{
        material::Lambertian,
        ray::Ray,
        shapes::{
            ray_marching::{RayMarchingShape, Heart},
            Cube, Shape, Sphere,
        },
        texture::SolidColor,
    },
};
use std::sync::Arc;

fn bench_intersections(c: &mut Criterion) {
    let material = Lambertian {
        albedo: Box::new(SolidColor {
            color: Vector3d::new(0.9, 0.1, 0.1),
        }),
    };
    let sphere_pos = Vector3d::zero();
    let sphere = Sphere::new(
        "Test sphere".into(),
        InversableTransform::new(
            sphere_pos,
            Vector3d::new(0.0, 0.0, 0.0),
            Vector3d::new(1.0, 1.0, 1.0),
        ),
        Arc::new(Box::new(Lambertian {
            albedo: Box::new(SolidColor {
                color: Vector3d::new(0.9, 0.1, 0.1),
            }),
        })),
        false,
    );
    let cube = Cube::new(
        "Test sphere".into(),
        InversableTransform::new(
            sphere_pos,
            Vector3d::new(0.0, 0.0, 0.0),
            Vector3d::new(1.0, 1.0, 1.0),
        ),
        Arc::new(Box::new(Lambertian {
            albedo: Box::new(SolidColor {
                color: Vector3d::new(0.9, 0.1, 0.1),
            }),
        })),
    );
    let heart = RayMarchingShape::new(
        Box::new(Heart::new()),
        0.01,
        InversableTransform::new(
            sphere_pos,
            Vector3d::new(0.0, 0.0, 0.0),
            Vector3d::new(1.0, 1.0, 1.0),
        ),
        Arc::new(Box::new(Lambertian {
            albedo: Box::new(SolidColor {
                color: Vector3d::new(0.9, 0.1, 0.1),
            }),
        })),
    );

    let mut group = c.benchmark_group("Intersects");
    group.significance_level(0.1).sample_size(500);

    group.bench_function("Sphere ray intersect", |b| {
        let pos = sphere_pos - Vector3d::random_in_sphere(10.0);
        let ray = Ray::new(pos, sphere_pos - &pos);

        let sphr = black_box(&sphere);
        b.iter(|| {
            sphr.ray_hit(&ray, 0.001, f64::INFINITY);
        })
    });
    group.bench_function("Cube ray intersect", |b| {
        let pos = sphere_pos - Vector3d::random_in_sphere(10.0);
        let ray = Ray::new(pos, sphere_pos - &pos);

        let sphr = black_box(&cube);
        b.iter(|| {
            sphr.ray_hit(&ray, 0.001, f64::INFINITY);
        })
    });
    group.bench_function("Cube ray intersect", |b| {
        let pos = sphere_pos - Vector3d::random_in_sphere(10.0);
        let ray = Ray::new(pos, sphere_pos - &pos);

        let sphr = black_box(&heart);
        b.iter(|| {
            sphr.ray_hit(&ray, 0.001, f64::INFINITY);
        })
    });

    group.finish();
}

criterion_group!(benches, bench_intersections);
criterion_main!(benches);
