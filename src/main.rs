use std::{fs::File, error::Error};
use glam::Vec3;

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

fn main() -> Result<(), Box<dyn Error>> {

    let mut canvas = Canvas::new(1920, 1080);

    let mut shapes: Vec<Box<dyn Traceable>> = Vec::new();
    
    macro_rules! rng {
        ($e:expr) => {
            thread_rng().gen_range($e)
        }
    }
    
    for _ in 0..2000 {
        shapes.push(
            Box::new(
                Sphere { 
                    pos: Vec3::new( rng!(0.0..canvas.width() as f32), rng!(0.0..canvas.height() as f32), 0.0 ),
                    radius: rng!(3.0..10.0)
                } 
            )
        )
    }
    
    // 
    // for shape in &shapes {
    //     canvas.draw(shape.as_ref(), Pixel(0, 255, 0));
    // }
    // 
    // canvas.draw(&bvh, thread_rng().gen());

    let bvh = Bvh::construct(shapes.iter().map(Box::as_ref).collect::<Vec<_>>().as_mut_slice(), 0);

    let mut results = vec![false; 2000];

    for _ in 0..(canvas.width()*canvas.height()) {
        let ray = Ray {
            start: Vec3::new(rng!(0.0..canvas.width() as f32), rng!(0.0..canvas.height() as f32), 0.0),
            dir: Vec3::new(thread_rng().gen(), thread_rng().gen(), 0.0).normalize()
        };

        results[0] = bvh.intersects(&ray).is_some();

        canvas.draw(&ray, Pixel::WHITE);
    }

    for (result, shape) in results.into_iter().zip(shapes.iter()) {
        if result {
            canvas.draw(shape.as_ref(), Pixel::gray(200));
        }
        else {
            canvas.draw(shape.as_ref(), Pixel::gray(128));
        }
    }

    let mut file = File::create("output.ppm")?;

    canvas.write_to(&mut file)?;

    Ok(())
}
