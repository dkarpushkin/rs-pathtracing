use std::{fmt::Display, ops::Mul};

use super::Vector3d;

#[derive(Debug)]
pub struct Matrix4x4d([[f64; 4]; 4]);

impl Matrix4x4d {
    fn translate(vec: Vector3d) -> Matrix4x4d {
        Matrix4x4d([
            [0.0, 0.0, 0.0, vec.x],
            [0.0, 0.0, 0.0, vec.y],
            [0.0, 0.0, 0.0, vec.z],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    fn scale_matrix(vec: Vector3d) -> Matrix4x4d {
        Matrix4x4d([
            [vec.x, 0.0, 0.0, 0.0],
            [0.0, vec.y, 0.0, 0.0],
            [0.0, 0.0, vec.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    fn rotate_matrix(vec: Vector3d) -> Matrix4x4d {
        Matrix4x4d([
            [
                vec.z.cos() * vec.y.cos(),
                vec.z.cos() * vec.y.sin() * vec.x.sin() - vec.z.sin() * vec.x.cos(),
                vec.z.cos() * vec.y.sin() * vec.x.cos() + vec.z.sin() * vec.x.sin(),
                0.0,
            ],
            [
                vec.z.sin() * vec.y.cos(),
                vec.z.sin() * vec.y.sin() * vec.x.sin() - vec.z.cos() * vec.x.cos(),
                vec.z.sin() * vec.y.sin() * vec.x.cos() + vec.z.cos() * vec.x.sin(),
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
}

impl Display for Matrix4x4d {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[\n[{:?}]\n[{:?}]\n[{:?}]\n[{:?}]\n]", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

impl Mul<Vector3d> for Matrix4x4d {
    type Output = Vector3d;

    fn mul(self, rhs: Vector3d) -> Self::Output {
        Vector3d {
            x: rhs.x * self.0[0][0] + rhs.y * self.0[0][1] + rhs.z * self.0[0][2] + self.0[0][3],
            y: rhs.x * self.0[1][0] + rhs.y * self.0[1][1] + rhs.z * self.0[1][2] + self.0[1][3],
            z: rhs.x * self.0[2][0] + rhs.y * self.0[2][1] + rhs.z * self.0[2][2] + self.0[2][3],
        }
    }
}

impl Mul<&Vector3d> for Matrix4x4d {
    type Output = Vector3d;

    fn mul(self, rhs: &Vector3d) -> Self::Output {
        Vector3d {
            x: rhs.x * self.0[0][0] + rhs.y * self.0[0][1] + rhs.z * self.0[0][2] + self.0[0][3],
            y: rhs.x * self.0[1][0] + rhs.y * self.0[1][1] + rhs.z * self.0[1][2] + self.0[1][3],
            z: rhs.x * self.0[2][0] + rhs.y * self.0[2][1] + rhs.z * self.0[2][2] + self.0[2][3],
        }
    }
}

impl Mul<Vector3d> for &Matrix4x4d {
    type Output = Vector3d;

    fn mul(self, rhs: Vector3d) -> Self::Output {
        Vector3d {
            x: rhs.x * self.0[0][0] + rhs.y * self.0[0][1] + rhs.z * self.0[0][2] + self.0[0][3],
            y: rhs.x * self.0[1][0] + rhs.y * self.0[1][1] + rhs.z * self.0[1][2] + self.0[1][3],
            z: rhs.x * self.0[2][0] + rhs.y * self.0[2][1] + rhs.z * self.0[2][2] + self.0[2][3],
        }
    }
}

impl Mul<&Vector3d> for &Matrix4x4d {
    type Output = Vector3d;

    fn mul(self, rhs: &Vector3d) -> Self::Output {
        Vector3d {
            x: rhs.x * self.0[0][0] + rhs.y * self.0[0][1] + rhs.z * self.0[0][2] + self.0[0][3],
            y: rhs.x * self.0[1][0] + rhs.y * self.0[1][1] + rhs.z * self.0[1][2] + self.0[1][3],
            z: rhs.x * self.0[2][0] + rhs.y * self.0[2][1] + rhs.z * self.0[2][2] + self.0[2][3],
        }
    }
}

impl Mul<Matrix4x4d> for Matrix4x4d {
    type Output = Matrix4x4d;

    fn mul(self, rhs: Matrix4x4d) -> Self::Output {
        let mut result = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self.0[i][0] * rhs.0[0][j] + self.0[i][1] * rhs.0[1][j] + self.0[i][2] * rhs.0[2][j] + self.0[i][3] * rhs.0[3][j];
            }
        }

        Matrix4x4d(result)
    }
}

impl Mul<&Matrix4x4d> for Matrix4x4d {
    type Output = Matrix4x4d;

    fn mul(self, rhs: &Matrix4x4d) -> Self::Output {
        let mut result = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self.0[i][0] * rhs.0[0][j] + self.0[i][1] * rhs.0[1][j] + self.0[i][2] * rhs.0[2][j] + self.0[i][3] * rhs.0[3][j];
            }
        }

        Matrix4x4d(result)
    }
}

impl Mul<Matrix4x4d> for &Matrix4x4d {
    type Output = Matrix4x4d;

    fn mul(self, rhs: Matrix4x4d) -> Self::Output {
        let mut result = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self.0[i][0] * rhs.0[0][j] + self.0[i][1] * rhs.0[1][j] + self.0[i][2] * rhs.0[2][j] + self.0[i][3] * rhs.0[3][j];
            }
        }

        Matrix4x4d(result)
    }
}

impl Mul<&Matrix4x4d> for &Matrix4x4d {
    type Output = Matrix4x4d;

    fn mul(self, rhs: &Matrix4x4d) -> Self::Output {
        let mut result = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self.0[i][0] * rhs.0[0][j] + self.0[i][1] * rhs.0[1][j] + self.0[i][2] * rhs.0[2][j] + self.0[i][3] * rhs.0[3][j];
            }
        }

        Matrix4x4d(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::algebra::approx_equal;

    use super::{Matrix4x4d, Vector3d};

    #[test]
    fn test_rotate_matrix() {
        let mat = Matrix4x4d::rotate_matrix(Vector3d::new(0.0, (-90.0 as f64).to_radians(), 0.0));
        let v = Vector3d::new(0.0, 0.0, -1.0);

        let v1 = mat * &v;

        assert!(approx_equal(v1.x, 1.0));
        assert!(approx_equal(v1.y, 0.0));
        assert!(approx_equal(v1.z, 0.0));
    }

    #[test]
    fn test_matrix_multiplication() {
        let m1 = Matrix4x4d([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0]
        ]);
        let m2 = Matrix4x4d([
            [17.0, 18.0, 19.0, 20.0],
            [21.0, 22.0, 23.0, 24.0],
            [25.0, 26.0, 27.0, 28.0],
            [29.0, 30.0, 31.0, 32.0]
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

    fn test_matrix_vector_multiplication() {
        let m1 = Matrix4x4d([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0]
        ]);
        let v = Vector3d::new(17.0, 18.0, 19.0);
    }
}
