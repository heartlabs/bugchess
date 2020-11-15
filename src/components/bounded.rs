use amethyst::ecs::{Component, DenseVecStorage};
use amethyst::core::math::{Vector3};

use ncollide3d::shape::Cuboid;
use std::time::Instant;

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct Bounded {
    pub bounds: Cuboid<f32>,
}

impl Bounded {
    pub fn new(width: f32, height: f32) -> Bounded {
        Bounded{bounds: Cuboid::new(Vector3::new(width/2., height/2., 0.0))}
    }
}


#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct MovingTowards {
    pub screen_pos: Vector3<f32>,
}

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct PowerAnimation {
    pub from_pos: Vector3<f32>,
    pub to_pos: Vector3<f32>,
    pub start_time: Instant,
    pub duration: u128,
    pub start_scale: f32,
    pub end_scale: f32,
}