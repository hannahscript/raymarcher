use crate::vec_util::{normalize, V3d};

pub struct Camera {
    pub center: V3d,
    pub viewport_width: f64,
    pub viewport_height: f64,
    pub focal_length: f64,
}

impl Camera {
    pub fn ray_generator(&self, image_width: u32, image_height: u32) -> RayGenerator {
        RayGenerator::new(self, image_width, image_height)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            center: V3d::new(0., 0., 0.),
            viewport_height: 2.0,
            viewport_width: 2.0 * 16. / 9., // todo store aspect ratio in one place (cam + raymarcher)
            focal_length: 1.,
        }
    }
}

pub struct RayGenerator {
    origin: V3d,
    horizontal: V3d,
    vertical: V3d,
    lower_left_corner: V3d,
    image_width: u32,
    image_height: u32,
    x: u32,
    y: u32,
}

impl RayGenerator {
    fn new(cam: &Camera, image_width: u32, image_height: u32) -> Self {
        let horizontal = V3d::new(cam.viewport_width, 0., 0.);
        let vertical = V3d::new(0., cam.viewport_height, 0.);

        RayGenerator {
            origin: cam.center,
            horizontal,
            vertical,
            lower_left_corner: cam.center
                - V3d::new(0., 0., cam.focal_length)
                - horizontal / 2.
                - vertical / 2.,
            image_width,
            image_height,
            x: 0,
            y: image_height - 1,
        }
    }
}

impl Iterator for RayGenerator {
    type Item = V3d;

    fn next(&mut self) -> Option<Self::Item> {
        if self.y == 0 && self.x >= self.image_width {
            return None;
        }

        let v = self.y as f64 / (self.image_height - 1) as f64;
        let u = self.x as f64 / (self.image_width - 1) as f64;
        let screen_v = self.lower_left_corner + self.horizontal * u + self.vertical * v;
        let direction = normalize(screen_v - self.origin);

        self.x += 1;
        if self.x >= self.image_width && self.y > 0 {
            self.x = 0;
            self.y -= 1;
        }

        Some(direction)
    }
}
