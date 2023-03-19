use crate::vec_util::{length, V3d};

pub type Color = (u8, u8, u8);

pub trait SceneObject {
    fn get_sdf(&self, p: V3d) -> f64;
    fn get_color(&self) -> Color;
}

pub struct Sphere {
    pub center: V3d,
    pub color: Color,
}

impl SceneObject for Sphere {
    fn get_sdf(&self, p: V3d) -> f64 {
        f64::sqrt((self.center - p).norm2()) - 2.
    }

    fn get_color(&self) -> Color {
        self.color
    }
}

pub struct ExclusionObject {
    pub a: Box<dyn SceneObject>,
    pub b: Box<dyn SceneObject>,
}

impl SceneObject for ExclusionObject {
    fn get_sdf(&self, p: V3d) -> f64 {
        f64::max(self.a.get_sdf(p), -self.b.get_sdf(p))
    }

    fn get_color(&self) -> Color {
        (100, 100, 100)
    }
}

pub struct Sierpinski {
    pub color: Color,
}

impl SceneObject for Sierpinski {
    fn get_sdf(&self, mut z1: V3d) -> f64 {
        const SCALE: f64 = 1.85;

        let mut z = z1;
        let a1 = V3d::new(1., 1., 1.);
        let a2 = V3d::new(-1., -1., 1.);
        let a3 = V3d::new(1., -1., -1.);
        let a4 = V3d::new(-1., 1., -1.);
        let mut c;
        let mut n = 0;
        let mut dist;
        let mut d;
        while n < 10 {
            c = a1;
            dist = length(z - a1);
            d = length(z - a2);
            if d < dist {
                c = a2;
                dist = d;
            }
            d = length(z - a3);
            if d < dist {
                c = a3;
                dist = d;
            }
            d = length(z - a4);
            if d < dist {
                c = a4;
                dist = d;
            }
            z = z * SCALE - c * (SCALE - 1.0);
            n += 1;
        }

        length(z) * f64::powf(SCALE, -n as f64)
    }

    fn get_color(&self) -> Color {
        self.color
    }
}

#[derive(Default)]
pub struct Scene {
    pub objects: Vec<Box<dyn SceneObject>>,
    pub background_color: Color,
}

impl Scene {
    pub fn sdf(&self, p: V3d) -> f64 {
        self.objects
            .iter()
            .map(|obj| obj.get_sdf(p))
            .min_by(|a, b| a.partial_cmp(b).expect("NaN values should not happen"))
            .expect("Scene should not be empty")
    }

    pub fn sdf_with_color(&self, p: V3d) -> (f64, Color) {
        self.objects
            .iter()
            .map(|obj| (obj.get_sdf(p), obj.get_color()))
            .min_by(|(a, _), (b, _)| a.partial_cmp(b).expect("NaN values should not happen"))
            .expect("Scene should not be empty")
    }
}
