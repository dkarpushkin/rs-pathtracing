use std::{fmt::Display, ops::Mul};

use serde::{de, de::Visitor, Deserialize, Serialize};

use super::Vector3d;

#[derive(Serialize, Debug)]
pub struct InversableTransform {
    pub direct: Transform,
    pub inverse: Transform,
}

impl InversableTransform {
    pub fn new(translate: Vector3d, rotate: Vector3d, scale: Vector3d) -> Self {
        let direct =
            Transform::translate(translate) * Transform::rotate(rotate) * Transform::scale(scale);
        let inverse = Transform::scale(Vector3d::new(1.0 / scale.x, 1.0 / scale.y, 1.0 / scale.z))
            * Transform::rotate(Vector3d::new(-rotate.x, -rotate.y, -rotate.z))
            * Transform::translate(Vector3d::new(-translate.x, -translate.y, -translate.z));
        Self { direct, inverse }
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

#[derive(Serialize, Debug)]
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
        let vec = Vector3d::new(vec.x.to_radians(), vec.y.to_radians(), vec.z.to_radians());
        Transform([
            [
                vec.z.cos() * vec.y.cos(),
                vec.z.cos() * vec.y.sin() * vec.x.sin() - vec.z.sin() * vec.x.cos(),
                vec.z.cos() * vec.y.sin() * vec.x.cos() + vec.z.sin() * vec.x.sin(),
                0.0,
            ],
            [
                vec.z.sin() * vec.y.cos(),
                vec.z.sin() * vec.y.sin() * vec.x.sin() + vec.z.cos() * vec.x.cos(),
                vec.z.sin() * vec.y.sin() * vec.x.cos() - vec.z.cos() * vec.x.sin(),
                0.0,
            ],
            [
                -vec.y.sin(),
                vec.y.cos() * vec.x.sin(),
                vec.y.cos() * vec.x.cos(),
                0.0,
            ],
            [0.0, 0.0, 0.0, 1.0],
        ])
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
    use crate::algebra::approx_equal;

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

    // #[test]
    // fn test_matrix_vector_multiplication() {
    //     let m1 = Transform([
    //         [1.0, 2.0, 3.0, 4.0],
    //         [5.0, 6.0, 7.0, 8.0],
    //         [9.0, 10.0, 11.0, 12.0],
    //         [13.0, 14.0, 15.0, 16.0],
    //     ]);
    //     let v = Vector3d::new(17.0, 18.0, 19.0);
    // }
}
