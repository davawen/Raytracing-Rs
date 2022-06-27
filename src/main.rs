use std::{fs::File, error::Error, ops::Add};
use glam::{ Vec2, Vec3 };

mod shape;
mod bvh;
mod canvas;
mod intersection;
mod material;

use canvas::*;
use intersection::{Intersection, Inter};
use material::Color;
use rand::{thread_rng, Rng};
use shape::*;
use bvh::Bvh;

use crate::{intersection::Traceable, material::Material};

/// Returns the ray passing through the center of a pixel given its position
fn pixel_as_ray(canvas: &Canvas, x: usize, y: usize, fov: f32) -> Ray {
    // Offset by half a pixel so rays go through the center of the pixels instead of the top left corner
    let pos = Vec2::new(x as f32, y as f32) + 0.5;

    let canvas_size = Vec2::new(canvas.width(), canvas.height());

    let normalized_coordinates = pos / canvas_size * 2.0 - Vec2::ONE; // Range -1..1

    let aspect_ratio = canvas_size.x / canvas_size.y;

    let ray_dir = Vec2::new(normalized_coordinates.x * aspect_ratio * fov, -normalized_coordinates.y * fov);

    Ray {
        start: Vec3::ZERO,
        dir: Vec3::new(ray_dir.x, ray_dir.y, 1.0).normalize()
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

fn trace(scene: &[&dyn Traceable], ray: Ray) -> Color {
    if let Some(inter) = intersection(scene, &ray) {
        let light = inter.normal.dot( Vec3::new(0.0, 1.0, 0.0) ).add(1.0).min(1.0);

        inter.shape.material().color * light
    }
    else {
        Color::BLACK
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

    // for _ in 0..10 {
    //     shapes.push(
    //         Box::new(
    //             Sphere {
    //                 pos: Vec3::new( rng!(-20.0..20.0), rng!(-20.0..20.0), rng!(15.0..30.0) ),
    //                 radius: rng!(3.0..10.0)
    //             }
    //         )
    //     )
    // }

        shapes.push(
            Box::new(
                Sphere {
                    pos: Vec3::new( 30.0, 0.0, 30.0 ),
                    radius: rng!(3.0..10.0),
                    material: Material { color: Color::GREEN, reflectivity: 0.0 }
                }
            )
        );

    // shapes.push(Box::new(Plane {
    //     pos: Vec3::new(0.0, 3.0, 0.0),
    //     normal: Vec3::new(0.0, 1.0, 0.0)
    // }));

    // let bvh = Bvh::construct(shapes.iter().map(Box::as_ref).collect::<Vec<_>>().as_mut_slice(), 0);

    let shapes_ref: Vec<_> = shapes.iter().map(Box::as_ref).collect();

    for y in 0..canvas.height() {
        for x in 0..canvas.width() {
            let ray = pixel_as_ray(&canvas, x, y, (90.0_f32/2.0).tan());

            canvas.set(x, y, trace(&shapes_ref, ray).into());
        }
    }

    let mut file = File::create("output.ppm")?;

    canvas.write_to(&mut file)?;

    Ok(())
}
