
#[derive(Serialize, Deserialize, Debug)]
pub struct Plane {
    pub normal: Vector3d,
    pub p0: Vector3d,
    material: Box<dyn Material>,
}

impl Plane {
    fn ray_intersect<'a>(&'a self, ray: &'a Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        let ln = &ray.direction * &self.normal;
        if approx_equal(ln, 0.0) {
            None
        } else {
            let x = ((&self.p0 - &ray.origin) * &self.normal) / ln;
            if x < min_t || x > max_t {
                None
            } else {
                Some(RayHit::new(
                    &ray.origin + x * &ray.direction,
                    self.normal.clone(),
                    x,
                    // &self.normal * &ray.direction < 0.0,
                    &self.material,
                    ray.clone(),
                    // TODO: сделать текстурные координаты
                    0.0,
                    0.0,
                ))
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SphereNoTransform {
    pub center: Vector3d,
    pub radius: f64,
    pub material: Box<dyn Material>,
}

impl SphereNoTransform {
    fn ray_intersect<'a>(&'a self, ray: &'a Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        let oc = &ray.origin - &self.center;
        let a = 1.0;
        let half_b = &ray.direction * &oc;
        let c = &oc * &oc - self.radius * self.radius;

        //  решение квадратного уравнения для x - растояние от ray.origin до точек пересечения
        //  ax^2 + bx + c = 0
        let d = half_b * half_b - a * c;
        let x = if d < 0.0 {
            return None;
        } else if d == 0.0 {
            -half_b * a
        } else {
            let x = (-half_b - d.sqrt()) / a;
            if x < min_t || x > max_t {
                let x = (-half_b + d.sqrt()) / a;
                if x < min_t || x > max_t {
                    return None;
                }
                x
            } else {
                x
            }
        };

        let p = &ray.origin + &ray.direction * x;
        let normal = (&p - &self.center) / self.radius;

        // let is_front_face = &normal * &ray.direction < 0.0;
        // if !is_front_face {
        //     normal = -normal;
        // }

        let theta = (-p.y).acos();
        let phi = (-p.z).atan2(p.x) + PI;
        Some(RayHit::new(
            p,
            normal,
            x,
            // is_front_face,
            &self.material,
            ray.clone(),
            phi / (2.0 * PI),
            theta / PI,
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Elipsoid {
    pub center: Vector3d,
    pub radius: Vector3d,
    pub material: Box<dyn Material>,
}

impl Elipsoid {
    fn ray_intersect<'a>(&'a self, ray: &'a Ray, min_t: f64, max_t: f64) -> Option<RayHit<'a>> {
        let oc = &ray.origin - &self.center;
        let a = ray.direction.x * ray.direction.x / self.radius.x
            + ray.direction.y * ray.direction.y / self.radius.y
            + ray.direction.z * ray.direction.z / self.radius.z;
        let half_b = oc.x * ray.direction.x / self.radius.x
            + oc.y * ray.direction.y / self.radius.y
            + oc.z * ray.direction.z / self.radius.z;
        let c =
            oc.x * oc.x / self.radius.x + oc.y * oc.y / self.radius.y + oc.z * oc.z / self.radius.z
                - 1.0;

        //  TODO: вынести решение квадратного уравнения в отдельную функцию
        let d = half_b * half_b - a * c;
        let x = if d < 0.0 {
            return None;
        } else if d == 0.0 {
            -half_b * a
        } else {
            let x = (-half_b - d.sqrt()) / a;
            if x < min_t || x > max_t {
                let x = (-half_b + d.sqrt()) / a;
                if x < min_t || x > max_t {
                    return None;
                }
                x
            } else {
                x
            }
        };

        let p = &ray.origin + &ray.direction * x;
        let normal = (&p - &self.center) / &self.radius;

        // let is_front_face = &normal * &ray.direction < 0.0;
        // if !is_front_face {
        //     normal = -normal;
        // }
        let theta = (-p.y).acos();
        let phi = (-p.z).atan2(p.x) + PI;
        Some(RayHit::new(
            p,
            normal,
            x,
            // is_front_face,
            &self.material,
            ray.clone(),
            phi / (2.0 * PI),
            theta / PI,
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
