use std::{io::{ Write, Result as IOResult }, fs::File, slice::from_raw_parts};
use derive_more::{ Div, DivAssign };

use glam::Vec3;

use num::{FromPrimitive, integer::Roots};
use rand::{ Rng, distributions::{ Distribution, Standard } };

use crate::{shape::{ Rect, Sphere, Ray }, intersection::Intersection, material::Color};

#[derive(Default, Clone, Copy, Debug, Div, DivAssign)]
pub struct Pixel(pub u8, pub u8, pub u8);

impl Pixel {
    pub const WHITE: Pixel = Pixel::splat(255);
    pub const BLACK: Pixel = Pixel::splat(0);
    pub const RED: Pixel = Pixel(255, 0, 0);
    pub const GREEN: Pixel = Pixel(0, 255, 0);
    pub const BLUE: Pixel = Pixel(0, 0, 255);
    pub const YELLOW: Pixel = Pixel(255, 255, 0);
    pub const PINK: Pixel = Pixel(255, 0, 255);
    pub const CYAN: Pixel = Pixel(0, 255, 255);

    pub const fn splat(v: u8) -> Self {
        Pixel( v, v, v )
    }

    fn as_slice(&self) -> &[u8] {
        unsafe {
            let this: *const u8 = &self.0;
            from_raw_parts(this, 3)
        }
    }
}

impl Distribution<Pixel> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Pixel {
        let ( r, g, b ) = rng.gen();
        Pixel( r, g, b )
    }
}

impl From<Color> for Pixel {
    fn from(c: Color) -> Self {
        Pixel( (c.r.clamp(0.0, 1.0) * 255.0) as u8, (c.g.clamp(0.0, 1.0) * 255.0) as u8, (c.b.clamp(0.0, 1.0) * 255.0) as u8 )
    }
}

pub struct Canvas {
    width: usize,
    height: usize,
    pub data: Vec<Pixel>
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Canvas {
            width,
            height,
            data: vec![ Pixel::default(); width * height ]
        }
    }

    /// Returns a slice over the individual components of the pixels
    pub fn flat_pixels(&self) -> &[u8] {
        unsafe {
            let data: *const u8 = self.data.get(0).unwrap().as_slice().as_ptr();

            from_raw_parts(data, self.data.len() * 3)
        }
    }

    pub fn write_to(&self, file: &mut File) -> IOResult<()> {
        writeln!(file, "P6\n{} {}\n255", self.width, self.height)?;

        let buffer: Vec<_> = self.flat_pixels().iter().map(|p| {
            (((*p as f64) / 256.0).sqrt() * 256.0) as u8 // Gamma correction
        }).collect();

        file.write_all( &buffer )?;

        Ok(())
    }

    fn check(&self, x: usize, y: usize) {
        if cfg!(debug_assertions) && (x > self.width || y > self.height) {
            panic!("Out of bounds access! ( x: {} for width {}, y: {} for height {})", x, self.width, y, self.height);
        }
    }

    pub fn set(&mut self, x: usize, y: usize, p: Pixel) {
        self.check(x, y);
        self.data[y * self.width + x] = p;
    }

    pub fn add(&mut self, x: usize, y: usize, o: Pixel) {
        let p = self.get_mut(x, y);

        *p = Pixel(
            p.0.saturating_add(o.0),
            p.1.saturating_add(o.1),
            p.2.saturating_add(o.2)
        );
    }

    #[allow(unused)]
    pub fn get(&self, x: usize, y: usize) -> &Pixel {
        self.check(x, y);
        &self.data[y * self.width + x]
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut Pixel {
        self.check(x, y);
        &mut self.data[y * self.width + x]
    }

    pub fn draw<T: Drawable + ?Sized>(&mut self, shape: &T, color: Pixel) {
        shape.draw(self, color);
    }

    pub fn draw_outline<T: Drawable + ?Sized>(&mut self, shape: &T, color: Pixel) {
        shape.draw_outline(self, color);
    }

    pub fn width<T: FromPrimitive>(&self) -> T { T::from_usize(self.width).unwrap() }
    pub fn height<T: FromPrimitive>(&self) -> T { T::from_usize(self.height).unwrap() }
}

pub trait Drawable {
    fn draw(&self, canvas: &mut Canvas, color: Pixel);

    fn draw_outline(&self, _canvas: &mut Canvas, _color: Pixel) {}
}

impl Drawable for Rect {
    fn draw(&self, canvas: &mut Canvas, color: Pixel) {
        let x1 = self.min.x.floor() as usize;
        let y1 = self.min.y.floor() as usize;
        let x2 = self.max.x.floor() as usize;
        let y2 = self.max.y.floor() as usize;

        for y in y1..=y2 {
            for x in x1..=x2 {
                if !(0..canvas.width).contains(&x) || !(0..canvas.height).contains(&y) { continue }

                canvas.add(x, y, color);
            }
        }
    }

    fn draw_outline(&self, canvas: &mut Canvas, color: Pixel) {
        let x1 = self.min.x.floor().max(0.0) as usize;
        let y1 = self.min.y.floor().max(0.0) as usize;
        let x2 = (self.max.x.floor() as usize).min(canvas.width::<usize>()-1);
        let y2 = (self.max.y.floor() as usize).min(canvas.height::<usize>()-1);

        for y in y1..=y2 {
            canvas.set(x1, y, color);
            canvas.set(x2, y, color);
        }

        for x in x1..=x2 {
            canvas.set(x, y1, color);
            canvas.set(x, y2, color);
        }
    }
}

impl Drawable for Sphere {
    fn draw(&self, canvas: &mut Canvas, color: Pixel) {
        let uradius = self.radius.ceil() as usize;
        let radius = self.radius;

        let x = self.pos.x;
        let y = self.pos.y;
        let ux = self.pos.x as usize;
        let uy = self.pos.y as usize;

        for cell_y in uy.saturating_sub(uradius)..=(uy+uradius).min(canvas.height-1) {
            for cell_x in ux.saturating_sub(uradius)..=(ux+uradius).min(canvas.width-1) {
                let fcell_x = cell_x as f32;
                let fcell_y = cell_y as f32;

                if (fcell_x - x)*(fcell_x - x) + (fcell_y - y)*(fcell_y - y) <= radius*radius {
                    canvas.set(cell_x, cell_y, color)
                }
            }
        }
    }
}

impl Drawable for Ray {
    fn draw(&self, canvas: &mut Canvas, color: Pixel) {
        let canvas_bounds = Rect {
            min: Vec3::ZERO,
            max: Vec3::new( canvas.width as f32 - 1.0, canvas.height as f32 - 1.0, 0.0 )
        };

        let mut p = self.start;
        let slope = self.dir.y / self.dir.x;

        while canvas_bounds.intersects(&p) { 
            canvas.set( p.x.floor() as usize, p.y.floor() as usize, color );

            p += Vec3::new(1.0, slope, 0.0);
        }
    }
}
