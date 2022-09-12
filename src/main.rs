use std::{error::Error, f32::consts::PI, sync::atomic::AtomicUsize, io::{Seek, Read}, mem};
use glam::{ Vec2, Vec3, Quat, Mat3, Mat4 };
use image::{RgbImage, Rgb, buffer::PixelsMut};
use itertools::Itertools;
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
    material::{Material, MaterialKind}
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

fn trace(scene: &Bvh, directional_light: &Vec3, ray: Ray, count: i32) -> Color {
    const MAX_COUNT: i32 = 7;

    if count >= MAX_COUNT { return Color::BLACK }

    let intensity = 30.0f32;

    if let Some(inter) = scene.intersects(&ray) {
        let material = inter.shape.material();

        // let direct: Color = (0..3).into_iter().map(|_| {
        //     let towards_light = Ray { start: ray.start, dir: (*directional_light + Vec3::new(random(), random(), random())/10.0).normalize() }.offset();
        //
        //     if let ( MaterialKind::Lambertian { .. }, None ) = ( material.kind, scene.intersects(&towards_light) ) {
        //         Color::WHITE * inter.normal.dot(towards_light.dir).max(0.0) * intensity
        //     } else {
        //         Color::BLACK
        //     }
        // }).reduce(|a, b| { a + b }).unwrap() / 3.0;

        let ( ray, attenuation ) = material.scatter(&ray, &inter);

        if let Some(ray) = ray {
            let indirect = trace(scene, directional_light, ray.offset(), count + 1);
            indirect * attenuation
        }
        else {
            attenuation
        }
    }
    else {
        let shadow = ray.dir.dot(*directional_light);

        // let direct = Color::WHITE * ray.dir.dot(*directional_light).max(0.0) * intensity;
        let sky = Color::new(0.1, 0.4, 0.7).lerp(Color::new(0.7, 0.8, 0.9), ray.dir.y/2.0 + 0.5); // Whiter towards top and bluer towards bottom

        // return direct + sky;

        if shadow >= 0.95 {
            Color::WHITE * intensity + sky
        }
        else {
            sky
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

#[derive(Debug, Default, Clone, Copy)]
#[repr(C, packed)]
struct StlTriangle {
    normal: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
    attribute: u16
}

fn load_stl_file<P: AsRef<std::path::Path>>(file: P) -> std::io::Result<Vec<Triangle<'static>>> {

    let mut data = std::fs::File::open(file)?;
    data.seek(std::io::SeekFrom::Current(80))?;

    let mut num_triangles: [u8; 4] = [0; 4];
    data.read_exact(&mut num_triangles)?;
    let num_triangles = u32::from_le_bytes(num_triangles);

    let mut triangles = Vec::new();

    for _ in 0..num_triangles {
        let mut t = StlTriangle::default();
        unsafe {
            let buffer: &mut [u8] = std::slice::from_raw_parts_mut(
                (&mut t as *mut StlTriangle).cast(), 
                mem::size_of::<StlTriangle>()
            );

            data.read_exact(buffer)?;
        }

        triangles.push( Triangle::new( 
            Vertex { pos: Vec3::from_array(t.v0), normal: Vec3::from_array(t.normal), tex: Vec2::ZERO }, 
            Vertex { pos: Vec3::from_array(t.v1), normal: Vec3::from_array(t.normal), tex: Vec2::ZERO }, 
            Vertex { pos: Vec3::from_array(t.v2), normal: Vec3::from_array(t.normal), tex: Vec2::ZERO }, 
            Material::new_lambertian(Color::WHITE) 
        ))
    }

    Ok(triangles)
}

fn aces(x: Color) -> Color {
    let a = 2.51f32;
    let b = 0.03f32;
    let c = 2.43f32;
    let d = 0.59f32;
    let e = 0.14f32;

    (x*(x*a + b))/(x*(x*c + d) + e)
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
            material: Material::new_lambertian(Color::new(0.6, 0.2, 0.0)).set_normal(&bumpy_grid_norm).set_size(( 1.0/20.0, 1.0/20.0 )) /* Material::new_lambertian(Color::new( 0.8, 0.4, 0.0 )) */
        })
    ];

    let dog = load_stl_file("/home/davawen/Documents/monke.stl").unwrap();
    let mat = Mat4::from_translation(Vec3::new(20.0, 10.0, -10.0)) * Mat4::from_rotation_y(PI/2.0) * Mat4::from_rotation_x(-PI/2.0) * Mat4::from_rotation_z(PI/1.7) * Mat4::from_scale(Vec3::splat(8.0));

    for mut t in dog { 
        t = t.transform(mat);
        // t.material = Material::new_transparent(1.31);
        t.material = Material::new_lambertian(Color::WHITE * 0.9);
        // t.material = Material::new_metal(Color::RED);
        
        shapes.push(Box::new(t)) 
    }

    macro_rules! square {
        ($a:expr, $b: expr, $c: expr, $d: expr) => {
            let p = square($a, $b, $c, $d);
            shapes.push(Box::new(p.0));
            shapes.push(Box::new(p.1));
        }
    }

    shapes.push(Box::new(Sphere {
        pos: Vec3::new(-15.0, 10.0, 20.0),
        radius: 12.0,
        material: Material::new_lambertian(Color::WHITE)
    }));
    shapes.push(Box::new(Sphere {
        pos: Vec3::new(50.0, 14.0, -10.0),
        radius: 7.0,
        material: Material::new_metal(Color::GRAY)
    }));
    shapes.push(Box::new(Sphere {
        pos: Vec3::new(5.0, 5.0, -10.0),
        radius: 5.0,
        material: Material::new_transparent(1.52)
    }));

    let fov = 90.0_f32.to_radians();

    let camera = Camera {
        position: Vec3::new(20.0, 20.0, -30.0),
        orientation: Quat::from_rotation_x(0.2)
    };

    let light_source = Vec3::new(-1.0, 1.0, -1.0).normalize();


    let mut canvas = RgbImage::new(800, 400);

    let mut shapes_ref: Vec<_> = shapes.iter().map(Box::as_ref).collect();

    let bvh = Bvh::construct(&mut shapes_ref, 0);

    unsafe {
        let count: AtomicUsize = AtomicUsize::new(0);
        let count_fraction = (canvas.width() * canvas.height() / 10) as usize;

        const NUM_SAMPLES: usize = 2048;

        let _canvas = (&mut canvas) as *mut RgbImage; // Ignore borrow checking, we know writes don't alias

        (*_canvas).enumerate_pixels_mut().par_bridge().for_each(|(x, y, pixel)| {
            let mut color = Color::BLACK;

            for _ in 0..NUM_SAMPLES {
                // Random direction through pixel for antialiasing
                let ray = pixel_as_ray(&canvas, &camera, x as f32 + random::<f32>(), y as f32 + random::<f32>(), fov);

                color += trace(&bvh, &light_source, ray, 0);
            }

            color /= NUM_SAMPLES as f32;

            color = aces(color);

            color.r = color.r.min(1.0);
            color.g = color.g.min(1.0);
            color.b = color.b.min(1.0);

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
