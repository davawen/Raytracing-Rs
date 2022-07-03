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
    const MAX_COUNT: i32 = 60;

    if count >= MAX_COUNT { return Color::BLACK }

    if let Some(inter) = scene.intersects(&ray) {
        let material = inter.shape.material();

        // if material.transparent {
        //     let mu = if inter.backface { material.refraction } else { 1.0 / material.refraction };
        //
        //     let cos_theta = ray.dir.dot(-inter.normal).min(1.0);
        //     let sin_theta = (1.0 - cos_theta*cos_theta).sqrt();
        //
        //     let ray = if mu * sin_theta > 1.0 {
        //         Ray { start: inter.point + inter.normal * 0.01, dir: ray.dir.reflect(inter.normal) }
        //     }
        //     else {  
        //         let out_perp = mu * ( ray.dir + cos_theta*inter.normal );
        //         let out_parallel = -(1.0 - out_perp.length_squared()).abs().sqrt() * inter.normal;
        //
        //         let refracted_dir = out_perp + out_parallel;
        //
        //         Ray { start: inter.point - inter.normal * 0.01, dir: refracted_dir.normalize() }
        //     };
        //
        //     trace( scene, light_source, ray, count + 1 )
        // }
        // else {
        //     if material.reflectivity == 0.0 { // Mat
        //         // let shadow = inter.normal.dot( (*light_source - inter.point).normalize() ).add(1.0).min(1.0);
        //         //
        //         // inter.shape.material().color * shadow
        //
        //         let towards_light = (*light_source - inter.point).normalize();
        //         let ray_light = Ray { start: inter.point + inter.normal * 0.001, dir: towards_light };
        //
        //         let direct_diffuse = if let Some(_in_shadow) = scene.intersects(&ray_light) {
        //             Color::BLACK
        //         }
        //         else {
        //             let shading = towards_light.dot(inter.normal).max(0.0);
        //
        //             Color::splat(shading)
        //         };
        //
        //         if count >= MAX_COUNT { return ( direct_diffuse * material.color) / PI }
        //
        //         let indirect_diffuse = {
        //             let ray = Ray { start: inter.point + inter.normal * 0.01, dir: random_vector_in_hemisphere(inter.normal) };
        //             let cosine_law = ray.dir.dot(inter.normal);
        //
        //             trace( scene, light_source, ray, count + 1 ) * cosine_law
        //         };
        //
        //         ( ( direct_diffuse + indirect_diffuse ) * material.color ) / PI
        //     }
        //     else {
        //         let reflected = ray.dir.reflect(inter.normal);
        //
        //         let ray = Ray { start: inter.point + inter.normal * 0.01, dir: reflected };
        //         let cosine_law = ray.dir.dot(inter.normal);
        //
        //         trace( scene, light_source, ray, count + 1 ) * cosine_law * ( material.color.lerp(Color::WHITE, material.reflectivity) )
        //     }
        // }

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

fn main() -> Result<(), Box<dyn Error>> {

    let image = Texture::from_file("/home/davawen/Pictures/funi.png")?;
    let earth = Texture::from_file("/home/davawen/Pictures/earth.jpg")?;
    let normal = Texture::from_file("/home/davawen/Pictures/3314-normal.jpg")?;
    let earth_normal = Texture::from_file("/home/davawen/Pictures/2k_earth_normal_map.tif")?;
    let metal_normal = Texture::from_file("/home/davawen/Pictures/metal.png")?.set_wrapping(TextureWrapping::MirroredRepeat);

    let mut shapes: Vec<Box<dyn Traceable>> = Vec::new();
    macro_rules! rng {
        ($e:expr) => {
            thread_rng().gen_range($e)
        }
    }

    for _ in 0..30 {
        let new_sphere = Sphere {
            pos: Vec3::new( rng!(-70.0..70.0), 0.0, rng!(30.0..90.0) ),
            radius: 10.0,
            material: if random() { Material::new_lambertian(random()) }
                else { Material::new_metal(random()).set_normal(&normal) }
        };

        // We know there is no shape other than spheres
        if !shapes.iter().any(|x| unsafe {
            let x = x as *const Box<dyn Traceable> as *const Box<Sphere>;
            (*x).intersects(&new_sphere)
        })
        {
            shapes.push(
                Box::new( new_sphere )
            );
        }
    }

    shapes.push(Box::new(Triangle::new(
        Vertex { pos: Vec3::new( 0.0, 15.0, 50.0 ), ..Default::default() },
        Vertex { pos: Vec3::new( 20.0, 10.0, 60.0 ), ..Default::default() },
        Vertex { pos: Vec3::new( 10.0, 15.0, 55.0 ), ..Default::default() },
        Material::new_lambertian(Color::GREEN)
    )));

    shapes.push(Box::new(Plane {
        pos: Vec3::new(0.0, -10.0, 0.0),
        normal: Vec3::new(0.0, 1.0, 0.0),
        material: Material::new_metal(Color::new(0.8, 0.4, 0.0)).set_normal(&metal_normal) /* Material::new_lambertian(Color::new( 0.8, 0.4, 0.0 )) */
    }));

    shapes.push(Box::new(Triangle::new(
        Vertex { pos: Vec3::new( 0.0, 20.0, 20.0 ), tex: Vec2::new(0.0, 0.0), ..Default::default() },
        Vertex { pos: Vec3::new( 0.0, 30.0, 20.0 ), tex: Vec2::new(0.0, 2.0), ..Default::default() },
        Vertex { pos: Vec3::new( 20.0, 20.0, 20.0 ), tex: Vec2::new(2.0, 0.0), ..Default::default() },
        Material::default().set_texture(&image)
    )));

    shapes.push(Box::new(Triangle::new(
        Vertex { pos: Vec3::new( 0.0, 30.0, 20.0 ), tex: Vec2::new(0.0, 2.0), ..Default::default() },
        Vertex { pos: Vec3::new( 20.0, 30.0, 20.0 ), tex: Vec2::new(2.0, 2.0), ..Default::default() },
        Vertex { pos: Vec3::new( 20.0, 20.0, 20.0 ), tex: Vec2::new(2.0, 0.0), ..Default::default() },
        Material::default().set_texture(&image)
    )));

    // shapes.push(Box::new(Sphere {
    //     pos: Vec3::new(-20.0, 0.0, 20.0),
    //     radius: 15.0,
    //     material: Material::default().set_texture(&earth)
    // }));
    //
    // shapes.push(Box::new(Sphere {
    //     pos: Vec3::new(20.0, 0.0, 20.0),
    //     radius: 15.0,
    //     material: Material::default().set_texture(&earth).set_normal(&earth_normal)
    // }));


    // shapes.push(Box::new(Sphere {
    //     pos: Vec3::new(7.0, 25.0, 20.0),
    //     radius: 5.0,
    //     material: Material::Transparent { refraction_index: 1.5 }
    // }));

    // shapes.push(Box::new(Sphere {
    //     pos: Vec3::new(10.0, 25.0, 20.0),
    //     radius: -4.0,
    //     material: Material::Transparent { refraction_index: 1.5 }
    // }));

    // shapes.push(Box::new({
    //     let mut t = Triangle::new(
    //         Vec3::new( 0.0, 30.0, 20.0 ),
    //         Vec3::new( 20.0, 10.0, 30.0 ),
    //         Vec3::new( 0.0, 10.0, 20.0 ),
    //         Material::Transparent { refraction_index: 2.0 }
    //     );
    //     t
    // }));

    let fov = 90.0_f32.to_radians();

    let camera = Camera {
        position: Vec3::new(0.0, 20.0, -5.0),
        orientation: Quat::from_rotation_x(0.5) 
    };

    let light_source = Vec3::new(0.0, 100.0, 0.0);

    let mut shapes_ref: Vec<_> = shapes.iter().map(Box::as_ref).collect();

    let bvh = Bvh::construct(&mut shapes_ref, 0);
    
    const NUM_SAMPLES: usize = 256;

    let mut canvas = RgbImage::new(1600, 900);

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
