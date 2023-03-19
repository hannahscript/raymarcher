use vector3d::Vector3d;

pub type V3d = Vector3d<f64>;
pub fn normalize(v: V3d) -> V3d {
    v / f64::sqrt(v.norm2())
}

pub fn length(v: V3d) -> f64 {
    f64::sqrt(v.norm2())
}
