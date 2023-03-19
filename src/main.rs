mod camera;
mod scene;
mod vec_util;

use crate::camera::Camera;
use crate::scene::{Color, ExclusionObject, Scene, Sierpinski, Sphere};
use show_image::{create_window, event, ImageInfo, ImageView};
use std::fs::OpenOptions;
use std::io;
use std::io::{BufWriter, Write};

use crate::vec_util::{length, normalize, V3d};

struct Image {
    pixels: Vec<Color>,
    image_width: u32,
    image_height: u32,
}

struct RayMarcher {
    ray_dist_epsilon: f64,
    normal_estimation_epsilon: f64,
    max_steps: u32,
    aspect_ratio: f64,
    image_width: u32,
}

impl Default for RayMarcher {
    fn default() -> Self {
        RayMarcher {
            ray_dist_epsilon: 1e-6,
            normal_estimation_epsilon: 0.1,
            max_steps: 100,
            image_width: 400,
            aspect_ratio: 16. / 9.,
        }
    }
}

impl RayMarcher {
    fn march(&self, scene: &Scene, cam: &Camera) -> Image {
        let image_height = (self.image_width as f64 / self.aspect_ratio) as u32;
        let distances: Vec<f64> = cam
            .ray_generator(self.image_width, image_height)
            .map(|dir| self.send_ray_dist(cam.center, dir, scene))
            .collect();

        let max_dist = distances
            .iter()
            .max_by(|&a, &b| a.partial_cmp(b).expect("NaN values should not happen"))
            .expect("Empty pixels?");

        let pixels = distances
            .iter()
            .map(|&d| {
                if d < 0.0 {
                    (0, 0, 0)
                } else {
                    let g = (1.0 - d / max_dist) * 255.0;
                    (g as u8, g as u8, g as u8)
                }
            })
            .collect();

        // let pixels = cam
        //     .ray_generator(self.image_width, image_height)
        //     .map(|dir| self.send_ray(cam.center, dir, scene))
        //     .collect();

        Image {
            pixels,
            image_width: self.image_width,
            image_height,
        }
    }

    fn send_ray<'a>(&'a self, origin: V3d, direction: V3d, scene: &'a Scene) -> Color {
        let mut depth = 0.0;
        for _step in 0..self.max_steps {
            let (dist, color) = scene.sdf_with_color(origin + direction * depth);
            if dist < self.ray_dist_epsilon {
                return color;
            }

            depth += dist
        }

        scene.background_color
    }

    fn send_ray_dist<'a>(&'a self, origin: V3d, direction: V3d, scene: &'a Scene) -> f64 {
        let mut depth = 0.0;
        for _step in 0..self.max_steps {
            let (dist, _color) = scene.sdf_with_color(origin + direction * depth);
            if dist < self.ray_dist_epsilon {
                return depth;
            }

            depth += dist
        }

        -1.0

        // while min_dist > self.ray_dist_epsilon && steps < self.max_steps {
        //     let (dist, color) = scene.sdf_with_color(current_point);
        //
        //     min_dist = dist;
        //     last_color = color;
        //     current_point = current_point + direction * min_dist;
        //     steps += 1;
        // }
        //
        // if steps >= self.max_steps {
        //     -1.0
        // } else {
        //     // self.apply_lighting(direction, last_color, scene)
        //     length(current_point - origin)
        // }
    }

    fn get_normal(&self, p: V3d, scene: &Scene) -> V3d {
        let v = V3d::new(
            scene.sdf(V3d::new(p.x + self.normal_estimation_epsilon, p.y, p.z))
                - scene.sdf(V3d::new(p.x - self.normal_estimation_epsilon, p.y, p.z)),
            scene.sdf(V3d::new(p.x, p.y + self.normal_estimation_epsilon, p.z))
                - scene.sdf(V3d::new(p.x, p.y - self.normal_estimation_epsilon, p.z)),
            scene.sdf(V3d::new(p.x, p.y, p.z + self.normal_estimation_epsilon))
                - scene.sdf(V3d::new(p.x, p.y, p.z - self.normal_estimation_epsilon)),
        );

        normalize(v)
    }

    fn apply_lighting(&self, p: V3d, color: Color, scene: &Scene) -> Color {
        let ambient = V3d::new(0.5, 0.5, 0.5);

        let normal = self.get_normal(p, scene);
        let light_color = V3d::new(1.0, 1.0, 1.0);
        let light_source = V3d::new(0., 1.5, -4.0);
        let diffuse_strength = f64::max(0.0, light_source.dot(normal));
        let diffuse = light_color * diffuse_strength;

        let lighting = ambient * 0.0 + diffuse;

        (
            (color.0 as f64 * lighting.x) as u8,
            (color.1 as f64 * lighting.y) as u8,
            (color.2 as f64 * lighting.z) as u8,
        )
    }
}

fn save_as_ppm(img: &Image) -> io::Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open("./renders/image.ppm")?;
    let mut writer = BufWriter::new(file);

    writer.write_all("P3\n".as_bytes())?;
    writer.write_all(format!("{} {}\n", img.image_width, img.image_height).as_bytes())?;
    writer.write_all("255".as_bytes())?;

    for (i, pixel) in img.pixels.iter().enumerate() {
        if i % (img.image_height as usize) == 0 {
            writer.write_all("\n".as_bytes())?;
        }

        writer.write_all(format!("{} {} {} ", pixel.0, pixel.1, pixel.2).as_bytes())?
    }

    writer.flush()
}

fn create_test_image(image_width: u32, image_height: u32) -> Vec<Color> {
    let mut image = vec![];
    for j in (0..image_height).rev() {
        for i in 0..image_width {
            let color = (
                i as f64 / (image_width - 1) as f64,
                j as f64 / (image_height - 1) as f64,
                0.25,
            );
            image.push((
                (color.0 * 255.999) as u8,
                (color.1 * 255.999) as u8,
                (color.2 * 255.999) as u8,
            ))
        }
    }

    image
}

fn convert_pixels(pixels: &Vec<Color>) -> Vec<u8> {
    let mut result = vec![];
    for x in pixels {
        result.push(x.0);
        result.push(x.1);
        result.push(x.2);
    }

    result
}

fn render_default_scene() -> Image {
    let mut scene = Scene::default();
    // let a = (Box::new(Sphere {
    //     center: V3d::new(0.0, 0., -5.),
    //     color: (200, 0, 0),
    // }));
    //
    // let b = (Box::new(Sphere {
    //     center: V3d::new(0., 1.5, -4.0),
    //     color: (200, 0, 0),
    // }));
    //
    // scene.objects.push(Box::new(ExclusionObject { a, b }));

    scene.objects.push(Box::new(Sierpinski {
        // center: V3d::new(0., 0., -5.),
        color: (200, 0, 0),
    }));

    let raymarcher = RayMarcher::default();
    let cam = Camera {
        center: V3d::new(0., 0., 3.0),
        viewport_height: 2.0,
        viewport_width: 2.0 * 16. / 9., // todo store aspect ratio in one place (cam + raymarcher)
        focal_length: 1.,
    };

    raymarcher.march(&scene, &cam)
}

#[show_image::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Rendering image ...");
    // let img = create_test_image(500, 900);
    let img = render_default_scene();

    println!("Saving to file ...");
    save_as_ppm(&img).expect("Could not save image");

    println!("Opening image in window ...");
    let pixels_u8 = convert_pixels(&img.pixels);
    let v_image = ImageView::new(
        ImageInfo::rgb8(img.image_width, img.image_height),
        &pixels_u8,
    );
    let window = create_window("Render", Default::default()).expect("Could not open window");
    window
        .set_image("render", v_image)
        .expect("Could not set image view");

    for event in window.event_channel()? {
        if let event::WindowEvent::KeyboardInput(event) = event {
            if event.input.key_code == Some(event::VirtualKeyCode::Escape)
                && event.input.state.is_pressed()
            {
                break;
            }
        }
    }

    Ok(())
}
