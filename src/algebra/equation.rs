use num::Complex;

use super::approx_equal;

pub fn solve_quadratic_equation(a: f64, half_b: f64, c: f64) -> Option<(f64, f64)> {
    let d = half_b * half_b - a * c;
    let d_sqrt = d.sqrt();
    if d < 0.0 {
        None
    } else if d == 0.0 {
        Some((-half_b, -half_b))
    } else {
        Some(((-half_b - d_sqrt) / a, (-half_b + d_sqrt) / a))
    }
}

pub fn solve_quantic_equation(
    a: Complex<f64>,
    b: Complex<f64>,
    c: Complex<f64>,
    d: Complex<f64>,
    e: Complex<f64>,
) -> [Complex<f64>; 4] {
    let b = b / a;
    let c = c / a;
    let d = d / a;
    let e = e / a;

    let b2 = b * b;
    let alpha = c - (3.0 / 8.0) * b2;
    let beta = (b2 * b) / 8.0 - (b * c) / 2.0 + d;
    let gamma = (-3.0 / 256.0) * b2 * b2 + b2 * c / 16.0 - b * d / 4.0 + e;

    let alpha2 = alpha * alpha;
    let t = -b / 4.0;
    if approx_equal(beta.re, 0.0) && approx_equal(beta.im, 0.0) {
        let r = (alpha2 - 4.0 * gamma).sqrt();
        let r1 = ((-alpha + r) / 2.0).sqrt();
        let r2 = ((-alpha - r) / 2.0).sqrt();

        [t + r1, t - r1, t + r2, t - r2]
    } else {
        let p = -(alpha2 / 12.0 + gamma);
        let q = -alpha2 * alpha / 108.0 + alpha * gamma / 3.0 - beta * beta / 8.0;
        let r = -q / 2.0 + (q * q / 4.0 + p * p * p / 27.0).sqrt();
        let u = r.cbrt();
        let mut y = (-5.0 / 6.0) * alpha + u;

        if approx_equal(u.re, 0.0) && approx_equal(u.im, 0.0) {
            y -= q.cbrt();
        } else {
            y -= p / (3.0 * u);
        }

        let w = (alpha + 2.0 * y).sqrt();

        let r1 = (-(3.0 * alpha + 2.0 * y + 2.0 * beta / w)).sqrt();
        let r2 = (-(3.0 * alpha + 2.0 * y - 2.0 * beta / w)).sqrt();

        [
            t + (w - r1) / 2.0,
            t + (w + r1) / 2.0,
            t + (-w - r2) / 2.0,
            t + (-w + r2) / 2.0,
        ]
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use num::Zero;

    use super::solve_quantic_equation;

    #[test]
    fn test_solve_quantic_equation() {
        let roots = solve_quantic_equation(
            3.0.into(),
            6.0.into(),
            (-123.0).into(),
            (-126.0).into(),
            1080.0.into(),
        );
        let real_roots = roots
            .iter()
            .filter_map(|n| if n.im.is_zero() { Some(n.re) } else { None })
            .collect_vec();
        println!("Imag roots: {:?}", roots);
        println!("Real roots: {:?}", real_roots);

        let roots = solve_quantic_equation(
            (-20.0).into(),
            5.0.into(),
            17.0.into(),
            (-29.0).into(),
            87.0.into(),
        );
        let real_roots = roots
            .iter()
            .filter_map(|n| if n.im.is_zero() { Some(n.re) } else { None })
            .collect_vec();
        println!("Imag roots: {:?}", roots);
        println!("Real roots: {:?}", real_roots);

        let roots = solve_quantic_equation(
            1.0.into(),
            (-4.0).into(),
            6.48.into(),
            (-4.96).into(),
            1.0376.into(),
        );
        let real_roots = roots
            .iter()
            .filter_map(|n| if n.im.is_zero() { Some(n.re) } else { None })
            .collect_vec();
        println!("Imag roots: {:?}", roots);
        println!("Real roots: {:?}", real_roots);
    }
}
