use std::{fs::File, error::Error, f32::consts::PI, cell::UnsafeCell, sync::{Mutex, mpsc}, thread};
use crossbeam::{scope};
use glam::{ Vec2, Vec3, Quat, Mat3 };

mod shape;
mod bvh;
mod canvas;
mod intersection;
mod material;

use canvas::*;
use intersection::{Intersection, Inter};
use lerp::Lerp;
use material::Color;
use rand::{thread_rng, Rng};
use shape::*;
use bvh::Bvh;

use crate::{intersection::Traceable, material::Material};

#[derive(Debug)]
struct Camera {
    position: Vec3,
    orientation: Quat
}

/// Returns the ray passing through the center of a pixel given its position
fn pixel_as_ray(canvas: &Canvas, camera: &Camera, x: usize, y: usize, fov: f32) -> Ray {
    // Offset by half a pixel so rays go through the center of the pixels instead of the top left corner
    let pos = Vec2::new(x as f32, y as f32) + 0.5;

    let canvas_size = Vec2::new(canvas.width(), canvas.height());

    let normalized_coordinates = pos / canvas_size * 2.0 - Vec2::ONE; // Range -1..1

    let aspect_ratio = canvas_size.x / canvas_size.y;

    let ray_dir = Vec2::new(normalized_coordinates.x * aspect_ratio * fov, -normalized_coordinates.y * fov);

    Ray {
        start: camera.position,
        dir: camera.orientation.mul_vec3(Vec3::new(ray_dir.x, ray_dir.y, 1.0).normalize())
    }

}

fn intersection<'a>(scene: &'a [&'a dyn Traceable], ray: &'a Ray) -> Option<Inter<'a, dyn Traceable>> {
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
    const MAX_COUNT: i32 = 3;

    if let Some(inter) = scene.intersects(&ray) {
        let material = inter.shape.material();

        if inter.shape.material().reflectivity == 0.0 { // Mat
            // let shadow = inter.normal.dot( (*light_source - inter.point).normalize() ).add(1.0).min(1.0);
            //
            // inter.shape.material().color * shadow

            let towards_light = (*light_source - inter.point).normalize();
            let ray_light = Ray { start: inter.point + inter.normal * 0.001, dir: towards_light };

            let direct_diffuse = if let Some(_in_shadow) = scene.intersects(&ray_light) {
                Color::BLACK
            }
            else {
                let shading = towards_light.dot(inter.normal).max(0.0);

                Color::splat(shading)
            };

            if count >= MAX_COUNT { return direct_diffuse * material.color / PI }

            const NUM_SAMPLES: i32 = 64;

            let mut indirect_diffuse = Color::BLACK;

            for _ in 0..NUM_SAMPLES {
                let ray = Ray { start: inter.point + inter.normal * 0.001, dir: random_vector_in_hemisphere(inter.normal) };
                let cosine_law = ray.dir.dot(inter.normal);

                indirect_diffuse += trace( scene, light_source, ray, count + 1 ) * cosine_law;
            }

            indirect_diffuse /= NUM_SAMPLES as f32 * (1.0 / (2.0 * PI));

            ( direct_diffuse + indirect_diffuse ) * material.color / PI
        }
        else {
            let reflected = ray.dir - 2.0 * ray.dir.dot(inter.normal) * inter.normal;

            let ray = Ray { start: inter.point + inter.normal * 0.001, dir: reflected };
            let cosine_law = ray.dir.dot(inter.normal);

            trace( scene, light_source, ray, count + 1 ) * cosine_law * ( material.color.lerp(Color::WHITE, material.reflectivity) )
        }
    }
    else {
        let shadow = ray.dir.dot((*light_source - ray.start).normalize());

        if shadow > 0.9 {
            Color::splat(shadow)
        }
        else {
            Color::new(0.1, 0.4, 0.7)
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {

    let mut canvas = Canvas::new(900, 600);

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
            material: Material { color: thread_rng().gen::<Pixel>().into(), reflectivity: rng!(0..=1) as f32 }
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
        Vec3::new( 0.0, 15.0, 50.0 ),
        Vec3::new( 20.0, 10.0, 60.0 ),
        Vec3::new( 10.0, 15.0, 55.0 ),
        Material { color: Color::GREEN, reflectivity: 0.0 }
    )));

    shapes.push(Box::new(Plane {
        pos: Vec3::new(0.0, -20.0, 0.0),
        normal: Vec3::new(0.0, 1.0, 0.0),
        material: Material { color: Color::new(0.8, 0.7, 0.0), reflectivity: 0.0 }
    }));

    let fov = 90.0_f32.to_radians();

    let camera = Camera {
        position: Vec3::new(0.0, 20.0, -5.0),
        orientation: Quat::from_rotation_x(0.5) 
    };

    let light_source = Vec3::new(0.0, 100.0, 0.0);

    let mut shapes_ref: Vec<_> = shapes.iter().map(Box::as_ref).collect();

    let bvh = Bvh::construct(&mut shapes_ref, 0);

    crossbeam::thread::scope(|s| {
        enum Message<T> {
            Message(T),
            Quit
        }

        use Message::*;

        println!("Creating handles");

        let ( mut targs, rargs ) = spmc::channel();
        let ( tx, rx ) = mpsc::channel();

        let mut handles = Vec::new();
        for _ in 0..thread::available_parallelism().unwrap().get() {

            let canvas = &canvas as *const Canvas as usize; // Reads/Writes to canvas are guaranteed to not alias
            let bvh = &bvh;
            let light_source = &light_source;
            let camera = &camera;

            let rargs = rargs.clone();
            let tx = tx.clone();

            handles.push( s.spawn(move |_| unsafe { 
                let canvas = &*(canvas as *const Canvas);

                while let Message(( x, y )) = rargs.recv().unwrap() {
                    let ray = pixel_as_ray(canvas, camera, x, y, (fov / 2.0).tan());

                    let out: Pixel = trace(bvh, light_source, ray, 0).into();

                    tx.send((x, y, out)).unwrap();
                }
            }) );
        }

        println!("Computing...");

        let mut num_sent = 0;
        let mut num_recieved = 0;

        for y in 0..canvas.height() {
            for x in 0..canvas.width() {
                targs.send(Message((x, y))).unwrap();

                num_sent += 1;
            }
        }

        println!("Recieving results...");

        while num_recieved != num_sent {
            let (x, y, out) = rx.recv().unwrap();
            num_recieved += 1;

            if x == 0 { println!("Recieved row {}", y); }

            canvas.set(x, y, out);
        }

        println!("Sending quit signals");

        // Terminate threads
        for _ in handles {
            targs.send(Quit).unwrap();
        }
    }).unwrap();

    let mut file = File::create("output.ppm")?;

    canvas.write_to(&mut file)?;

    Ok(())
}
