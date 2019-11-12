use nalgebra as na;

pub fn na64_to_g(v: na::Vector2<f64>) -> godot::Vector2 {
    let v: na::Vector2<f32> = na::convert(v);
    let v: mint::Vector2<f32> = v.into();
    v.into()
}

pub fn g_to_na64(v: godot::Vector2) -> na::Vector2<f64> {
    let v: mint::Vector2<f32> = v.into();
    let v: na::Vector2<f32> = v.into();
    na::convert(v)
}
