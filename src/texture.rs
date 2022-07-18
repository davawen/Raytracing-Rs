use image::{RgbImage, ImageError};
use lerp::Lerp;

use crate::material::Color;

#[derive(Debug, Clone, Copy)]
pub enum TextureWrapping {
    Repeat,
    MirroredRepeat,
    ClampToEdge
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub data: RgbImage,
    pub width: usize,
    pub height: usize,

    wrapping: TextureWrapping
}

impl Texture {
    pub fn new(data: RgbImage) -> Self {
        let width = data.width() as usize;
        let height = data.height() as usize;

        Texture {
            data,
            width,
            height,
            wrapping: TextureWrapping::Repeat
        }
    }

    pub fn set_wrapping(mut self, wrapping: TextureWrapping) -> Self {
        self.wrapping = wrapping;
        self
    }

    pub fn from_file<P>(filepath: P) -> Result<Self, ImageError>
    where
        P: AsRef<std::path::Path>
    {
        let data = image::open(filepath)?.into_rgb8();

        Ok(Texture::new(data))
    }

    /// Samples the texture from two u,v coordinates ranging from 0 to 1 and interpolates matching pixels with them
    pub fn sample(&self, u: f32, v: f32) -> Color {
        let ( u, v ) = match self.wrapping {
            TextureWrapping::Repeat => {
                let sawtooth = |x: f32| (x + 0.5) - (0.5 + (x + 0.5)).floor() + 0.5;
                ( sawtooth(u), sawtooth(v) )
            },
            TextureWrapping::MirroredRepeat => {
                let triangle = |x: f32| 2.0 * (x/2.0 - (x/2.0 + 0.5).floor()).abs(); // https://en.wikipedia.org/wiki/Triangle_wave
                ( triangle(u), triangle(v) )
            },
            TextureWrapping::ClampToEdge => ( u, v )
        };

        let ( u, v ) = ( u.clamp(0.0, 1.0), v.clamp(0.0, 1.0) );

        let ( x, y ) = ( u * (self.width - 1) as f32, (1.0 - v) * (self.height - 1) as f32 );

        let ( fx, cx ) = ( x.floor(), x.ceil() );
        let ( fy, cy ) = ( y.floor(), y.ceil() );

        let nw: Color = self.data.get_pixel(fx as u32, fy as u32).into();
        let ne: Color = self.data.get_pixel(cx as u32, fy as u32).into();
        let sw: Color = self.data.get_pixel(fx as u32, cy as u32).into();
        let se: Color = self.data.get_pixel(cx as u32, cy as u32).into();

        let north = nw.lerp(ne, x - fx);
        let south = sw.lerp(se, x - fx);

        north.lerp(south, y - fy)
    }
}

