#![cfg(test)]

use glam::Vec3;

use crate::{shape::{Sphere, Ray}, intersection::{Intersection, Traceable}};

#[test]
fn inside_sphere_intersect() {
    let sphere = Sphere { pos: Vec3::ZERO, radius: 5.0, material: Default::default() };

    let ray = Ray { start: Vec3::ZERO, dir: Vec3::new(1.0, 0.0, -1.0).normalize() };

    let inter = sphere.ray_intersection(&ray).unwrap();

    println!("{:#?}", inter);
}
