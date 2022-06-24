use std::{fs::File, error::Error};
use glam::{ Vec2, Vec3 };

mod shape;
mod bvh;
mod canvas;
mod intersection;

use canvas::*;
use intersection::Intersection;
use rand::{thread_rng, Rng};
use shape::*;
use bvh::Bvh;

use crate::intersection::Traceable;

/// Returns the ray passing through the center of a pixel given its position
fn pixel_as_ray(canvas: &Canvas, x: usize, y: usize, fov: f32) -> Ray {
    let pos = Vec2::new(x as f32, y as f32);
    let normalized_coordinates = pos / Vec2::new(canvas.width() as f32, canvas.height() as f32) * 2.0 - Vec2::NEG_ONE; // Range -1..1


}

fn main() -> Result<(), Box<dyn Error>> {

    let mut canvas = Canvas::new(900, 600);

    let mut shapes: Vec<Box<dyn Traceable>> = Vec::new();
    
    // macro_rules! rng {
    //     ($e:expr) => {
    //         thread_rng().gen_range($e)
    //     }
    // }
    // 
    // for _ in 0..2000 {
    //     shapes.push(
    //         Box::new(
    //             Sphere { 
    //                 pos: Vec3::new( rng!(0.0..canvas.width() as f32), rng!(0.0..canvas.height() as f32), 0.0 ),
    //                 radius: rng!(3.0..10.0)
    //             } 
    //         )
    //     )
    // }
    
    // 
    // for shape in &shapes {
    //     canvas.draw(shape.as_ref(), Pixel(0, 255, 0));
    // }
    // 
    // canvas.draw(&bvh, thread_rng().gen());

    // let bvh = Bvh::construct(shapes.iter().map(Box::as_ref).collect::<Vec<_>>().as_mut_slice(), 0);


    let mut file = File::create("output.ppm")?;

    canvas.write_to(&mut file)?;

    Ok(())
}
