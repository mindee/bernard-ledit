/// Linearly interpolates between `a` and `b` by `t` (`t = 0.0` gives `a`, `t = 1.0` gives `b`).
#[allow(dead_code)]
#[must_use]
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    (b - a).mul_add(t, a)
}
