use super::Vector3d;
use crate::world::ray::Ray;
use serde::{de, de::Visitor, Deserialize, Serialize};
use std::{fmt::Display, ops::Mul};

#[derive(Serialize, Debug, Clone)]
pub struct InversableTransform {
    translate: Vector3d,
    rotate: Vector3d,
    scale: Vector3d,
    pub direct: Transform,
    pub inverse: Transform,
}

impl InversableTransform {
    pub fn new(translate: Vector3d, rotate: Vector3d, scale: Vector3d) -> Self {
        let direct =
            Transform::translate(translate) * Transform::rotate(rotate) * Transform::scale(scale);
        let inverse = Transform::scale(Vector3d::new(1.0 / scale.x, 1.0 / scale.y, 1.0 / scale.z))
            * Transform::rotate_inverse(Vector3d::new(-rotate.x, -rotate.y, -rotate.z))
            * Transform::translate(Vector3d::new(-translate.x, -translate.y, -translate.z));
        Self { translate, rotate, scale, direct, inverse }
    }

    pub fn direct_transform_ray(&self, ray: &Ray) -> Ray {
        Ray {
            origin: self.direct.transform_point(&ray.origin),
            direction: self.direct.transform_vector(&ray.direction),
        }
    }

    pub fn inverse_transform_ray(&self, ray: &Ray) -> Ray {
        Ray {
            origin: self.inverse.transform_point(&ray.origin),
            direction: self.inverse.transform_vector(&ray.direction),
        }
    }

    /// Get a reference to the inversable transform's translate.
    pub fn translate(&self) -> Vector3d {
        self.translate
    }

    /// Get a reference to the inversable transform's rotate.
    pub fn rotate(&self) -> Vector3d {
        self.rotate
    }

    /// Get a reference to the inversable transform's scale.
    pub fn scale(&self) -> Vector3d {
        self.scale
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct InversableTransformJson {
    translate: Vector3d,
    rotate: Vector3d,
    scale: Vector3d,
}

impl From<InversableTransformJson> for InversableTransform {
    fn from(transform: InversableTransformJson) -> Self {
        Self::new(transform.translate, transform.rotate, transform.scale)
    }
}

impl From<InversableTransform> for InversableTransformJson {
    fn from(transform: InversableTransform) -> Self {
        let mat = &transform.direct.0;
        let translate = Vector3d::new(mat[0][3], mat[1][3], mat[2][3]);
        let scale = Vector3d::new(
            mat[0][0] * mat[0][0] + mat[1][0] * mat[1][0] + mat[2][0] * mat[2][0],
            mat[0][1] * mat[0][1] + mat[1][1] * mat[1][1] + mat[2][1] * mat[2][1],
            mat[0][2] * mat[0][2] + mat[1][2] * mat[1][2] + mat[2][2] * mat[2][2],
        );
        let rotate_mat = Transform([
            [
                mat[0][0] / scale.x,
                mat[0][1] / scale.y,
                mat[0][2] / scale.z,
                0.0,
            ],
            [
                mat[1][0] / scale.x,
                mat[1][1] / scale.y,
                mat[1][2] / scale.z,
                0.0,
            ],
            [
                mat[2][0] / scale.x,
                mat[2][1] / scale.y,
                mat[2][2] / scale.z,
                0.0,
            ],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        let v1 = Vector3d::new(1.0, 1.0, 1.0);
        let v2 = rotate_mat.transform_vector(&v1);
        let x_rotate_cos = (v1.y * v2.y + v1.z * v2.z)
            / ((v1.y * v1.y + v1.z * v1.z) * (v2.y * v2.y + v2.z * v2.z));
        let y_rotate_cos = (v1.x * v2.x + v1.z * v2.z)
            / ((v1.x * v1.x + v1.z * v1.z) * (v2.x * v2.x + v2.z * v2.z));
        let z_rotate_cos = (v1.x * v2.x + v1.y * v2.y)
            / ((v1.x * v1.x + v1.y * v1.y) * (v2.x * v2.x + v2.y * v2.y));
        let rotate = Vector3d::new(
            x_rotate_cos.acos(),
            y_rotate_cos.acos(),
            z_rotate_cos.acos(),
        );

        Self {
            translate,
            rotate,
            scale,
        }
    }
}

impl<'de> Deserialize<'de> for InversableTransform {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        #[allow(non_camel_case_types)]
        enum Field {
            Translate,
            Rotate,
            Scale,
        }

        struct InversableTransformVisitor;

        impl<'de> Visitor<'de> for InversableTransformVisitor {
            type Value = InversableTransform;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct InversableTransform")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut translate = None;
                let mut rotate = None;
                let mut scale = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Translate => {
                            if translate.is_some() {
                                return Err(de::Error::duplicate_field("translate"));
                            }
                            translate = Some(map.next_value()?);
                        }
                        Field::Rotate => {
                            if rotate.is_some() {
                                return Err(de::Error::duplicate_field("rotate"));
                            }
                            rotate = Some(map.next_value()?);
                        }
                        Field::Scale => {
                            if scale.is_some() {
                                return Err(de::Error::duplicate_field("scale"));
                            }
                            scale = Some(map.next_value()?);
                        }
                    }
                }
                let translate: Vector3d =
                    translate.ok_or_else(|| de::Error::missing_field("translate"))?;
                let rotate: Vector3d = rotate.ok_or_else(|| de::Error::missing_field("rotate"))?;
                let scale: Vector3d = scale.ok_or_else(|| de::Error::missing_field("scale"))?;

                Ok(InversableTransform::new(translate, rotate, scale))
            }
        }

        deserializer.deserialize_struct(
            "InversableTransform",
            &["translate", "rotate", "scale"],
            InversableTransformVisitor,
        )
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Transform(pub [[f64; 4]; 4]);

impl<'de> Deserialize<'de> for Transform {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        #[allow(non_camel_case_types)]
        enum Field {
            Translate,
            Rotate,
            Scale,
            Is_Inverse,
        }

        struct TransformVisitor;

        impl<'de> Visitor<'de> for TransformVisitor {
            type Value = Transform;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Transform")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut matrix = [[0.0; 4]; 4];

                let mut i = 0;
                while let Some(row) = seq.next_element()? {
                    matrix[i] = row;
                    i += 1;
                }

                Ok(Transform(matrix))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut translate = None;
                let mut rotate = None;
                let mut scale = None;
                let mut is_inverse = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Translate => {
                            if translate.is_some() {
                                return Err(de::Error::duplicate_field("translate"));
                            }
                            translate = Some(map.next_value()?);
                        }
                        Field::Rotate => {
                            if rotate.is_some() {
                                return Err(de::Error::duplicate_field("rotate"));
                            }
                            rotate = Some(map.next_value()?);
                        }
                        Field::Scale => {
                            if scale.is_some() {
                                return Err(de::Error::duplicate_field("scale"));
                            }
                            scale = Some(map.next_value()?);
                        }
                        Field::Is_Inverse => {
                            if is_inverse.is_some() {
                                return Err(de::Error::duplicate_field("is_inverse"));
                            }
                            is_inverse = Some(map.next_value()?)
                        }
                    }
                }
                let translate: Vector3d =
                    translate.ok_or_else(|| de::Error::missing_field("translate"))?;
                let rotate: Vector3d = rotate.ok_or_else(|| de::Error::missing_field("rotate"))?;
                let scale: Vector3d = scale.ok_or_else(|| de::Error::missing_field("scale"))?;
                let is_inverse: bool = is_inverse.unwrap_or(false);

                if is_inverse {
                    let result = Ok(Transform::scale(Vector3d::new(
                        1.0 / scale.x,
                        1.0 / scale.y,
                        1.0 / scale.z,
                    )) * Transform::rotate(Vector3d::new(
                        -rotate.x, -rotate.y, -rotate.z,
                    )) * Transform::translate(Vector3d::new(
                        -translate.x,
                        -translate.y,
                        -translate.z,
                    )));
                    result
                } else {
                    Ok(Transform::scale(scale)
                        * Transform::rotate(rotate)
                        * Transform::translate(translate))
                }
            }
        }

        deserializer.deserialize_struct(
            "Transform",
            &["translate", "rotate", "scale"],
            TransformVisitor,
        )
    }
}

impl Transform {
    // pub fn new(translate: Vector3d, rotate: Vector3d, scale: Vector3d) -> Transform {
    //     Self::scale(scale) * Self::rotate(rotate) * Self::translate(translate)
    // }

    pub fn unit() -> Transform {
        Transform([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn translate(vec: Vector3d) -> Transform {
        Transform([
            [1.0, 0.0, 0.0, vec.x],
            [0.0, 1.0, 0.0, vec.y],
            [0.0, 0.0, 1.0, vec.z],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn scale(vec: Vector3d) -> Transform {
        Transform([
            [vec.x, 0.0, 0.0, 0.0],
            [0.0, vec.y, 0.0, 0.0],
            [0.0, 0.0, vec.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotate(vec: Vector3d) -> Transform {
        // let vec = Vector3d::new(vec.x.to_radians(), vec.y.to_radians(), vec.z.to_radians());
        // Transform([
        //     [
        //         vec.z.cos() * vec.y.cos(),
        //         vec.z.cos() * vec.y.sin() * vec.x.sin() - vec.z.sin() * vec.x.cos(),
        //         vec.z.cos() * vec.y.sin() * vec.x.cos() + vec.z.sin() * vec.x.sin(),
        //         0.0,
        //     ],
        //     [
        //         vec.z.sin() * vec.y.cos(),
        //         vec.z.sin() * vec.y.sin() * vec.x.sin() + vec.z.cos() * vec.x.cos(),
        //         vec.z.sin() * vec.y.sin() * vec.x.cos() - vec.z.cos() * vec.x.sin(),
        //         0.0,
        //     ],
        //     [
        //         -vec.y.sin(),
        //         vec.y.cos() * vec.x.sin(),
        //         vec.y.cos() * vec.x.cos(),
        //         0.0,
        //     ],
        //     [0.0, 0.0, 0.0, 1.0],
        // ])
        Transform::rotate_roll(vec.x) * Transform::rotate_pitch(vec.y) * Transform::rotate_yaw(vec.z)
    }

    pub fn rotate_inverse(vec: Vector3d) -> Transform {
        Transform::rotate_yaw(vec.z) * Transform::rotate_pitch(vec.y) * Transform::rotate_roll(vec.x)
    }

    pub fn rotate_roll(degrees: f64) -> Transform {
        let radians = degrees.to_radians();
        Transform([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, radians.cos(), -radians.sin(), 0.0],
            [0.0, radians.sin(), radians.cos(), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotate_pitch(degrees: f64) -> Transform {
        let radians = degrees.to_radians();
        Transform([
            [radians.cos(), 0.0, radians.sin(), 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [-radians.sin(), 0.0, radians.cos(), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotate_yaw(degrees: f64) -> Transform {
        let radians = degrees.to_radians();
        Transform([
            [radians.cos(), -radians.sin(), 0.0, 0.0],
            [radians.sin(), radians.cos(), 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn transform_point(&self, point: &Vector3d) -> Vector3d {
        Vector3d {
            x: point.x * self.0[0][0]
                + point.y * self.0[0][1]
                + point.z * self.0[0][2]
                + self.0[0][3],
            y: point.x * self.0[1][0]
                + point.y * self.0[1][1]
                + point.z * self.0[1][2]
                + self.0[1][3],
            z: point.x * self.0[2][0]
                + point.y * self.0[2][1]
                + point.z * self.0[2][2]
                + self.0[2][3],
        }
    }

    pub fn transform_vector(&self, vector: &Vector3d) -> Vector3d {
        Vector3d {
            x: vector.x * self.0[0][0] + vector.y * self.0[0][1] + vector.z * self.0[0][2],
            y: vector.x * self.0[1][0] + vector.y * self.0[1][1] + vector.z * self.0[1][2],
            z: vector.x * self.0[2][0] + vector.y * self.0[2][1] + vector.z * self.0[2][2],
        }
    }

    pub fn transform_normal(&self, normal: &Vector3d) -> Vector3d {
        Vector3d {
            x: normal.x * self.0[0][0] + normal.y * self.0[1][0] + normal.z * self.0[2][0],
            y: normal.x * self.0[0][1] + normal.y * self.0[1][1] + normal.z * self.0[2][1],
            z: normal.x * self.0[0][2] + normal.y * self.0[1][2] + normal.z * self.0[2][2],
        }
    }

    #[allow(dead_code)]
    fn decompose(&self) -> InversableTransformJson {
        let mat = &self.0;
        let translate = Vector3d::new(mat[0][3], mat[1][3], mat[2][3]);
        let scale = Vector3d::new(
            Vector3d::new(mat[0][0], mat[1][0], mat[2][0]).length(),
            Vector3d::new(mat[0][1], mat[1][1], mat[2][1]).length(),
            Vector3d::new(mat[0][2], mat[1][2], mat[2][2]).length(),
        );
        let rotate_mat = Transform([
            [
                mat[0][0] / scale.x,
                mat[0][1] / scale.y,
                mat[0][2] / scale.z,
                0.0,
            ],
            [
                mat[1][0] / scale.x,
                mat[1][1] / scale.y,
                mat[1][2] / scale.z,
                0.0,
            ],
            [
                mat[2][0] / scale.x,
                mat[2][1] / scale.y,
                mat[2][2] / scale.z,
                0.0,
            ],
            [0.0, 0.0, 0.0, 1.0],
        ]);

        let r = &rotate_mat.0;
        let y_rotate = (-r[2][0]).atan2((r[0][0] * r[0][0] + r[1][0] * r[1][0]).sqrt());
        let x_rotate = (r[2][1] / y_rotate.cos()).atan2(r[2][2] / y_rotate.cos());
        let z_rotate = (r[1][0] / y_rotate.cos()).atan2(r[0][0] / y_rotate.cos());
        let rotate = Vector3d::new(
            x_rotate.to_degrees(),
            y_rotate.to_degrees(),
            z_rotate.to_degrees(),
        );

        // let v1 = Vector3d::new(1.0, 1.0, 1.0);
        // let v2 = rotate_mat.transform_vector(&v1);
        // let x_rotate_cos = (v2.y + v2.z) / 2.0;
        // let y_rotate_cos = (v2.x + v2.z) / 2.0;
        // let z_rotate_cos = (v2.x + v2.y) / 2.0;

        // let yz = rotate_mat.transform_vector(&Vector3d::new(0.0, 1.0, 1.0));
        // let xz = rotate_mat.transform_vector(&Vector3d::new(1.0, 0.0, 1.0));
        // let xy = rotate_mat.transform_vector(&Vector3d::new(1.0, 1.0, 0.0));
        // let x_rotate_cos = (yz.y + yz.z) / 2.0;
        // let y_rotate_cos = (xz.x + xz.z) / 2.0;
        // let z_rotate_cos = (xy.x + xy.y) / 2.0;

        // let rotate = Vector3d::new(
        //     x_rotate_cos.acos().to_degrees(),
        //     y_rotate_cos.acos().to_degrees(),
        //     (z_rotate_cos.acos() + PI).to_degrees(),
        // );

        InversableTransformJson {
            translate,
            rotate,
            scale,
        }
    }
}

impl Display for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[\n[{:?}]\n[{:?}]\n[{:?}]\n[{:?}]\n]",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

impl Mul<Vector3d> for Transform {
    type Output = Vector3d;

    fn mul(self, rhs: Vector3d) -> Self::Output {
        Vector3d {
            x: rhs.x * self.0[0][0] + rhs.y * self.0[0][1] + rhs.z * self.0[0][2] + self.0[0][3],
            y: rhs.x * self.0[1][0] + rhs.y * self.0[1][1] + rhs.z * self.0[1][2] + self.0[1][3],
            z: rhs.x * self.0[2][0] + rhs.y * self.0[2][1] + rhs.z * self.0[2][2] + self.0[2][3],
        }
    }
}

impl Mul<&Vector3d> for Transform {
    type Output = Vector3d;

    fn mul(self, rhs: &Vector3d) -> Self::Output {
        Vector3d {
            x: rhs.x * self.0[0][0] + rhs.y * self.0[0][1] + rhs.z * self.0[0][2] + self.0[0][3],
            y: rhs.x * self.0[1][0] + rhs.y * self.0[1][1] + rhs.z * self.0[1][2] + self.0[1][3],
            z: rhs.x * self.0[2][0] + rhs.y * self.0[2][1] + rhs.z * self.0[2][2] + self.0[2][3],
        }
    }
}

impl Mul<Vector3d> for &Transform {
    type Output = Vector3d;

    fn mul(self, rhs: Vector3d) -> Self::Output {
        Vector3d {
            x: rhs.x * self.0[0][0] + rhs.y * self.0[0][1] + rhs.z * self.0[0][2] + self.0[0][3],
            y: rhs.x * self.0[1][0] + rhs.y * self.0[1][1] + rhs.z * self.0[1][2] + self.0[1][3],
            z: rhs.x * self.0[2][0] + rhs.y * self.0[2][1] + rhs.z * self.0[2][2] + self.0[2][3],
        }
    }
}

impl Mul<&Vector3d> for &Transform {
    type Output = Vector3d;

    fn mul(self, rhs: &Vector3d) -> Self::Output {
        Vector3d {
            x: rhs.x * self.0[0][0] + rhs.y * self.0[0][1] + rhs.z * self.0[0][2] + self.0[0][3],
            y: rhs.x * self.0[1][0] + rhs.y * self.0[1][1] + rhs.z * self.0[1][2] + self.0[1][3],
            z: rhs.x * self.0[2][0] + rhs.y * self.0[2][1] + rhs.z * self.0[2][2] + self.0[2][3],
        }
    }
}

impl Mul<Transform> for Transform {
    type Output = Transform;

    fn mul(self, rhs: Transform) -> Self::Output {
        let mut result = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self.0[i][0] * rhs.0[0][j]
                    + self.0[i][1] * rhs.0[1][j]
                    + self.0[i][2] * rhs.0[2][j]
                    + self.0[i][3] * rhs.0[3][j];
            }
        }

        Transform(result)
    }
}

impl Mul<&Transform> for Transform {
    type Output = Transform;

    fn mul(self, rhs: &Transform) -> Self::Output {
        let mut result = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self.0[i][0] * rhs.0[0][j]
                    + self.0[i][1] * rhs.0[1][j]
                    + self.0[i][2] * rhs.0[2][j]
                    + self.0[i][3] * rhs.0[3][j];
            }
        }

        Transform(result)
    }
}

impl Mul<Transform> for &Transform {
    type Output = Transform;

    fn mul(self, rhs: Transform) -> Self::Output {
        let mut result = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self.0[i][0] * rhs.0[0][j]
                    + self.0[i][1] * rhs.0[1][j]
                    + self.0[i][2] * rhs.0[2][j]
                    + self.0[i][3] * rhs.0[3][j];
            }
        }

        Transform(result)
    }
}

impl Mul<&Transform> for &Transform {
    type Output = Transform;

    fn mul(self, rhs: &Transform) -> Self::Output {
        let mut result = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self.0[i][0] * rhs.0[0][j]
                    + self.0[i][1] * rhs.0[1][j]
                    + self.0[i][2] * rhs.0[2][j]
                    + self.0[i][3] * rhs.0[3][j];
            }
        }

        Transform(result)
    }
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng};

    use crate::algebra::{approx_equal, approx_equal_scaled};

    use super::{Transform, Vector3d};

    #[test]
    fn test_rotate_matrix() {
        let mat = Transform::rotate(Vector3d::new(0.0, -90.0, 0.0));
        let v = Vector3d::new(0.0, 0.0, -1.0);

        let v1 = mat * &v;

        assert!(approx_equal(v1.x, 1.0));
        assert!(approx_equal(v1.y, 0.0));
        assert!(approx_equal(v1.z, 0.0));

        let v = Vector3d::new(1.0, 1.0, 1.0);
        println!("{:?}; len = {}", v, &v * &v);
        let mat = Transform::rotate(Vector3d::new(0.0, 90.0, 0.0));
        let v1 = mat * &v;
        println!("{:?}; len = {}", v1, &v1 * &v1);
        let mat = Transform::rotate(Vector3d::new(0.0, -90.0, 0.0));
        let v1 = mat * &v;
        println!("{:?}; len = {}", v1, &v1 * &v1);

        let mat = Transform::rotate(Vector3d::new(-90.0, 0.0, 90.0));
        let mat1 = Transform::rotate_roll(-90.0) * Transform::rotate_pitch(0.0) * Transform::rotate_yaw(90.0);
        let v1 = &mat * &v;
        println!("mat = {:?}", mat);
        println!("mat1 = {:?}", mat1);
        println!("{:?}; len = {}", v1, &v1 * &v1);
    }

    #[test]
    fn test_matrix_multiplication() {
        let m1 = Transform([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0],
        ]);
        let m2 = Transform([
            [17.0, 18.0, 19.0, 20.0],
            [21.0, 22.0, 23.0, 24.0],
            [25.0, 26.0, 27.0, 28.0],
            [29.0, 30.0, 31.0, 32.0],
        ]);

        let m3 = &m1 * &m2;

        assert_eq!(m3.0[0][0], 250.0);
        assert_eq!(m3.0[1][0], 618.0);
        assert_eq!(m3.0[2][3], 1112.0);

        let m3 = m2 * m1;

        assert_eq!(m3.0[0][0], 538.0);
        assert_eq!(m3.0[1][0], 650.0);
        assert_eq!(m3.0[2][3], 1080.0);
    }

    #[test]
    fn test_matrix_decomposition() {
        let mut rng = thread_rng();
        let translate = Vector3d::new(23.0, 54.0, 39.0);
        let rotate = Vector3d::new(
            rng.gen_range(-90.0..90.0),
            rng.gen_range(-90.0..90.0),
            rng.gen_range(-90.0..90.0),
        );
        let scale = Vector3d::new(1.2, 3.4, 5.6);
        let mat =
            Transform::translate(translate) * Transform::rotate(rotate) * Transform::scale(scale);

        let decomposed = mat.decompose();

        assert!(approx_equal_scaled(decomposed.rotate.x, rotate.x, 1e-10));
        assert!(approx_equal_scaled(decomposed.rotate.y, rotate.y, 1e-10));
        assert!(approx_equal_scaled(decomposed.rotate.z, rotate.z, 1e-10));
    }
}
