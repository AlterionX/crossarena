use nalgebra as na;

pub fn rotation(angle: f64) -> na::Matrix2<f64> {
    na::Matrix2::new(
        angle.cos(), -angle.sin(),
        angle.sin(),  angle.cos(),
    )
}
