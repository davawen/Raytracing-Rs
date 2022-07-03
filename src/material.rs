use std::{ops::Mul, f32::consts::PI};

use crate::{shape::Ray, intersection::{Inter, Traceable}, reflect::Reflect, texture::Texture};
use derive_more::{ Add, AddAssign, Mul, MulAssign, Sub, SubAssign, Div, DivAssign };
use glam::{Vec3, Mat3, Vec3Swizzles};
use image::Rgb;
use rand::{Rng, prelude::Distribution, distributions::Standard, random};

#[derive(Debug, Clone, Copy, Add, AddAssign, Mul, MulAssign, Sub, SubAssign, Div, DivAssign)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32
}

impl Color {
    pub const WHITE: Color = Color::splat(1.0);
    pub const GRAY: Color = Color::splat(0.5);
    pub const BLACK: Color = Color::splat(0.0);
    pub const RED: Color = Color::new(1.0, 0.0, 0.0);
    pub const GREEN: Color = Color::new(0.0, 1.0, 0.0);
    pub const BLUE: Color = Color::new(0.0, 0.0, 1.0);
    pub const YELLOW: Color = Color::new(1.0, 1.0, 0.0);
    pub const PINK: Color = Color::new(1.0, 0.0, 1.0);
    pub const CYAN: Color = Color::new(0.0, 1.0, 1.0);

    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Color {
            r, g, b
        }
    }

    pub fn from_u8(r: u8, g: u8, b: u8) -> Self {
        Color {
            r: (r as f32) / 255.0,
            g: (g as f32) / 255.0,
            b: (b as f32) / 255.0
        }
    }

    pub const fn splat(c: f32) -> Self {
        Color::new(c, c, c)
    }

    pub fn splat_u8(c: u8) -> Self {
        Color::from_u8(c, c, c)
    }
}

impl From<Vec3> for Color {
    fn from(vec: Vec3) -> Self {
        Color::new(vec.x, vec.y, vec.z)
    }
}

/// Implements From<T> for both T and &T using the same implementation
macro_rules! impl_from_ref {
    ($from_name:ty, $to_name:ty, $variable:ident, $implementation:block) => {
        impl From<$from_name> for $to_name {
            fn from($variable: $from_name) -> Self $implementation
        }
        
        impl From<& $from_name> for $to_name {
            fn from($variable: &$from_name) -> Self $implementation
        }
    };
}

impl_from_ref!(Rgb<u8>, Color, p, {
    let p = &p.0;

    Color::from_u8(p[0], p[1], p[2])
});

#[allow(clippy::from_over_into)]
impl Into<Vec3> for Color {
    fn into(self) -> Vec3 {
        Vec3::new(self.r, self.g, self.b)
    }
}

#[allow(clippy::from_over_into)]
impl Into<Rgb<u8>> for Color {
    fn into(self) -> Rgb<u8> {
        Rgb([ self.r, self.g, self.b ].map(|x| (x.clamp(0.0, 1.0)*255.0) as u8))
    }
}

impl Mul<Color> for Color {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b
        }
    }
}

impl Distribution<Color> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Color {
        let ( r, g, b ) = rng.gen(); // Interval [0; 1[

        Color::new(r, g, b)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Material<'a> {
    texture: Option<&'a Texture>,
    normal_map: Option<&'a Texture>,
    kind: MaterialKind
}

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub enum MaterialKind {
    Lambertian { albedo: Color },
    Metal { albedo: Color },
    Transparent { refraction_index: f32 }
}

impl Default for Material<'_> {
    fn default() -> Self {
        Material {
            texture: None,
            normal_map: None,
            kind: MaterialKind::Lambertian { albedo: Color::WHITE }
        }
    }
}

fn tangent_to_world_matrix(normal: Vec3) -> Mat3 {
    let n_t = if normal.x.abs() > normal.y.abs() {
        Vec3::new(normal.z, 0.0, -normal.x)
    }
    else {
        Vec3::new(0.0, -normal.z, normal.y)
    }.normalize();

    let n_b = normal.cross(n_t);

    Mat3::from_cols(n_b, normal, n_t)
}

fn random_vector_in_hemisphere(tangent_matrix: Mat3) -> Vec3 {
    // Sample point on local hemisphere
    let r1: f32 = random();
    let r2: f32 = random();

    let sin_theta = ( 1.0 - r1*r1 ).sqrt();
    let phi = 2.0*PI*r2;
    let x = sin_theta * phi.cos();
    let z = sin_theta * phi.sin();

    let sample = Vec3::new(x, r1, z);

    // Transform(rotate) sample into normal coordinate space
    tangent_matrix * sample
}

impl<'a> Material<'a> {
    pub fn new_lambertian(albedo: Color) -> Self {
        Material { kind: MaterialKind::Lambertian { albedo }, ..Default::default() }
    }
    pub fn new_metal(albedo: Color) -> Self {
        Material { kind: MaterialKind::Metal { albedo }, ..Default::default() }
    }
    pub fn new_transparent(refraction_index: f32) -> Self {
        Material { kind: MaterialKind::Transparent { refraction_index }, ..Default::default() }
    }

    pub fn set_texture(mut self, texture: &'a Texture) -> Self {
        self.texture = Some(texture);
        self
    }

    pub fn set_normal(mut self, texture: &'a Texture) -> Self {
        self.normal_map = Some(texture);
        self
    }


    pub fn scatter(&self, ray: &Ray, inter: &Inter<&dyn Traceable>) -> ( Ray, Color) {
        use MaterialKind::*;

        let tex = if let Some(image) = self.texture { 
            let ( u, v ) = inter.shape.sample(inter.point);

            image.sample(u, v)
        }
        else {
            Color::WHITE
        };

        // Construct coordinate system aligned to original normal
        let tangent_matrix = tangent_to_world_matrix(inter.normal);

        let normal = if let Some(map) = self.normal_map {
            let ( u, v ) = inter.shape.sample(inter.point);

            let normal: Vec3 = map.sample(u, v).into();
            let normal = normal*2.0 - 1.0; // Transform normal from range [0; 1] to [-1; 1]

            (tangent_matrix * normal.xzy()).normalize()
        }
        else { inter.normal };

        match self.kind {
            Lambertian { albedo } => {
                let ray = Ray { start: inter.point, dir: random_vector_in_hemisphere(tangent_matrix) };
                let cosine_law = ray.dir.dot(normal).max(0.0);

                ( ray, albedo * tex * cosine_law )
            },
            Metal { albedo } => {
                let reflected = ray.dir.reflect(normal);

                let ray = Ray { start: inter.point, dir: reflected };

                ( ray, albedo * tex )
            },
            Transparent { refraction_index: index } => {
                let mu = if inter.front { 1.0 / index } else { index };

                let cos_theta = ray.dir.dot(-normal).min(1.0);
                let sin_theta = (1.0 - cos_theta*cos_theta).sqrt();

                let ray = if 
                    mu * sin_theta > 1.0 || // Snells law, if n1/n2 * sin(theta) > 1.0 -> Total internal reflection
                    Material::schlick_reflectance(cos_theta, mu) > random() // Randomly reflect or refract, but the steeper the angle of vision, the more reflection is choosen
                {
                    Ray { start: inter.point, dir: ray.dir.reflect(normal) }
                }
                else {  
                    let out_perp = mu * ( ray.dir + cos_theta*normal );
                    let out_parallel = -(1.0 - out_perp.length_squared()).abs().sqrt() * normal;

                    let refracted_dir = out_perp + out_parallel;

                    Ray { start: inter.point, dir: refracted_dir.normalize() }
                };

                ( ray, Color::WHITE )
            }
        }
    }

    fn schlick_reflectance(cosine: f32, mu: f32) -> f32 {
        let r0 = (1.0 - mu) / (1.0 + mu);
        let r0 = r0*r0;

        r0 + (1.0 - r0)*(1.0 - cosine).powf(5.0)
    }
}
