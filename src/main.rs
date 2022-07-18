use std::{error::Error, f32::consts::PI, sync::atomic::AtomicUsize};
use glam::{ Vec2, Vec3, Quat, Mat3 };
use image::{RgbImage, Rgb, buffer::PixelsMut};
use rayon::prelude::*;

mod shape;
mod bvh;
mod intersection;
mod material;
mod reflect;
mod texture;

use intersection::{Intersection, Inter};
use texture::*;
use lerp::Lerp;
use material::Color;
use rand::{thread_rng, Rng, random};
use shape::*;
use bvh::Bvh;

use crate::{
    intersection::Traceable,
    material::Material
};

// 144s


#[derive(Debug)]
struct Camera {
    position: Vec3,
    orientation: Quat
}

/// Returns the ray passing through a pixel given its position
fn pixel_as_ray(canvas: &RgbImage, camera: &Camera, x: f32, y: f32, fov: f32) -> Ray {
    let pos = Vec2::new(x, y);

    let canvas_size = Vec2::new(canvas.width() as f32, canvas.height() as f32);

    let normalized_coordinates = pos / canvas_size * 2.0 - Vec2::ONE; // Range -1..1

    let aspect_ratio = canvas_size.x / canvas_size.y;

    let ray_dir = Vec2::new(normalized_coordinates.x * aspect_ratio * fov, -normalized_coordinates.y * fov);

    Ray {
        start: camera.position,
        dir: camera.orientation.mul_vec3(Vec3::new(ray_dir.x, ray_dir.y, 1.0).normalize())
    }

}

fn intersection<'a>(scene: &'a [&'a dyn Traceable], ray: &'a Ray) -> Option<Inter<&'a dyn Traceable>> {
    scene.iter()
        .filter_map(|shape| {
            shape.ray_intersection(ray)
        })
        .min_by(|a, b|{
            a.point.distance_squared(ray.start).partial_cmp(&b.point.distance_squared(ray.start)).unwrap()
        })
}

fn random_vector_in_hemisphere(normal: Vec3) -> Vec3 {
    // Sample point on local hemisphere
    let r1: f32 = thread_rng().gen_range(0.0..1.0);
    let r2: f32 = thread_rng().gen_range(0.0..1.0);

    let sin_theta = ( 1.0 - r1*r1 ).sqrt();
    let phi = 2.0*PI*r2;
    let x = sin_theta * phi.cos();
    let z = sin_theta * phi.sin();

    let sample = Vec3::new(x, r1, z);

    // Construct coordinate system aligned to normal
    let n_t = if normal.x.abs() > normal.y.abs() {
        Vec3::new(normal.z, 0.0, -normal.x)
    }
    else {
        Vec3::new(0.0, -normal.z, normal.y)
    }.normalize();

    let n_b = normal.cross(n_t);

    // Transform(rotate) sample into normal coordinate space
    let matrix = Mat3::from_cols(n_b, normal, n_t);

    matrix * sample
}

fn trace(scene: &Bvh, light_source: &Vec3, ray: Ray, count: i32) -> Color {
    const MAX_COUNT: i32 = 20;

    if count >= MAX_COUNT { return Color::BLACK }

    if let Some(inter) = scene.intersects(&ray) {
        let material = inter.shape.material();

        let ( ray, attenuation ) = material.scatter(&ray, &inter);

        trace(scene, light_source, ray.offset(), count + 1) * attenuation
    }
    else {
        let shadow = ray.dir.dot((*light_source - ray.start).normalize());

        if shadow > 0.9 {
            Color::splat(shadow)
        }
        else {
            Color::new(0.1, 0.4, 0.7).lerp(Color::new(0.7, 0.8, 0.9), ray.dir.y/2.0 + 0.5) // Whiter towards top and bluer towards bottom
        }
    }
}

/// Creates a z-axis aligned rectangle out of two triangles
fn square( center: Vec3, size: Vec2, orientation: Quat, material: Material ) -> ( Triangle, Triangle ) {
    let p1 = orientation * Vec3::new(-size.x/2.0, 0.0, -size.y/2.0);
    let p2 = orientation * Vec3::new( size.x/2.0, 0.0, -size.y/2.0);
    let p3 = orientation * Vec3::new(-size.x/2.0, 0.0,  size.y/2.0);
    let p4 = orientation * Vec3::new( size.x/2.0, 0.0,  size.y/2.0);

    let p1 = Vertex { pos: center + p1, tex: Vec2::new(0.0, 1.0), ..Default::default() };
    let p2 = Vertex { pos: center + p2, tex: Vec2::new(1.0, 1.0), ..Default::default() };
    let p3 = Vertex { pos: center + p3, tex: Vec2::new(0.0, 0.0), ..Default::default() };
    let p4 = Vertex { pos: center + p4, tex: Vec2::new(1.0, 0.0), ..Default::default() };

    (
        Triangle::new( p1, p2, p3, material.clone() ),
        Triangle::new( p2, p3, p4, material )
    )
}

fn main() -> Result<(), Box<dyn Error>> {

    // let image = Texture::from_file("/home/davawen/Pictures/funi.png")?;
    // let earth = Texture::from_file("/home/davawen/Pictures/earth.jpg")?;
    let earth_normal = Texture::from_file("/home/davawen/Pictures/2k_earth_normal_map.tif")?;
    let rusty_metal_norm = Texture::from_file("/home/davawen/Pictures/3314-normal.jpg")?;
    let bumpy_grid_norm = Texture::from_file("/home/davawen/Pictures/metal.png")?.set_wrapping(TextureWrapping::Repeat);
    let bumpy_norm = Texture::from_file("/home/davawen/Pictures/bumpy_normal.jpg")?.set_wrapping(TextureWrapping::MirroredRepeat);
    let scratched_norm = Texture::from_file("/home/davawen/Pictures/reduced.png")?.set_wrapping(TextureWrapping::MirroredRepeat);

    let mut shapes: Vec<Box<dyn Traceable>> = vec![
        Box::new(Plane {
            pos: Vec3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(0.0, 1.0, 0.0),
            material: Material::new_metal(Color::new(0.8, 0.4, 0.0)).set_normal(&bumpy_grid_norm).set_size(( 1.0/20.0, 1.0/20.0 )) /* Material::new_lambertian(Color::new( 0.8, 0.4, 0.0 )) */
        })
    ];

    macro_rules! square {
        ($a:expr, $b: expr, $c: expr, $d: expr) => {
            let p = square($a, $b, $c, $d);
            shapes.push(Box::new(p.0));
            shapes.push(Box::new(p.1));
        }
    }

    square!(
        Vec3::new(-30.0, 20.0, 15.0),
        Vec2::new(40.0, 30.0),
        Quat::from_rotation_z(PI/2.0),
        Material::new_lambertian(Color::RED)  
    );
    square!(
        Vec3::new(30.0, 20.0, 15.0),
        Vec2::new(40.0, 30.0),
        Quat::from_rotation_z(PI/2.0),
        Material::new_lambertian(Color::BLUE)  
    );
    square!(
        Vec3::new(0.0, 20.0, 30.0),
        Vec2::new(60.0, 40.0),
        Quat::from_rotation_x(PI/2.0),
        Material::new_lambertian(Color::GREEN)  
    );
    square!(
        Vec3::new(0.0, 40.0, 15.0),
        Vec2::new(60.0, 30.0),
        Quat::IDENTITY,
        Material::new_lambertian(Color::WHITE)  
    );

    shapes.push(Box::new(Sphere {
        pos: Vec3::new(-15.0, 10.0, 20.0),
        radius: 12.0,
        material: Material::new_lambertian(Color::WHITE)
    }));
    shapes.push(Box::new(Sphere {
        pos: Vec3::new(10.0, 10.0, 12.0),
        radius: 7.0,
        material: Material::new_lambertian(Color::WHITE)
    }));
    shapes.push(Box::new(Sphere {
        pos: Vec3::new(5.0, 25.0, 10.0),
        radius: 5.0,
        material: Material::new_transparent(1.52)
    }));

    let fov = 60.0_f32.to_radians();

    let camera = Camera {
        position: Vec3::new(20.0, 20.0, -7.0),
        orientation: Quat::from_rotation_x(0.2) * Quat::from_rotation_y(-0.4)
    };

    let light_source = Vec3::new(0.0, 500.0, 15.0);

    const NUM_SAMPLES: usize = 256;

    let mut canvas = RgbImage::new(1200, 900);

    let mut shapes_ref: Vec<_> = shapes.iter().map(Box::as_ref).collect();

    let bvh = Bvh::construct(&mut shapes_ref, 0);

    unsafe {
        let count: AtomicUsize = AtomicUsize::new(0);
        let count_fraction = (canvas.width() * canvas.height() / 10) as usize;


        let _canvas = (&mut canvas) as *mut RgbImage; // Ignore borrow checking, we know writes don't alias

        (*_canvas).enumerate_pixels_mut().par_bridge().for_each(|(x, y, pixel)| {
            let mut color = Color::BLACK;

            for _ in 0..NUM_SAMPLES {
                // Random direction through pixel for antialiasing
                let ray = pixel_as_ray(&canvas, &camera, x as f32 + random::<f32>(), y as f32 + random::<f32>(), fov);

                color += trace(&bvh, &light_source, ray, 0);
            }

            color /= NUM_SAMPLES as f32;

            *pixel = color.into();

            let val = count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if val % count_fraction == 0 {
                println!("{} % done", val/count_fraction * 10);
            }
        });
    }

    // Gamma correction
    canvas.iter_mut().for_each(|p| *p = (((*p as f64) / 256.0).sqrt() * 256.0) as u8 );

    canvas.save("output.png")?;

    Ok(())
}
