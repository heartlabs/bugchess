use amethyst::{
    core::{
        transform::Transform,
        math::Vector3,
    },
    ecs::{Join, ReadStorage, System, WriteStorage, ReadExpect, Entities},
};
use crate::components::{Cell};
use crate::components::board::{BoardPosition};

use std::ops::Sub;
use crate::components::bounded::{MovingTowards, PowerAnimation};
use crate::resources::board::Board;
use std::time::Instant;

pub struct PowerAnimationSystem;

impl<'a> System<'a> for PowerAnimationSystem {
    type SystemData = (
        WriteStorage<'a, Transform>,
        WriteStorage<'a, PowerAnimation>,
        Entities<'a>,
    );

    fn run(&mut self, (mut transforms, mut power_animations, entities): Self::SystemData) {

        let mut finished_moving = Vec::new();

        for (transform, animation, e) in (&mut transforms, &mut power_animations, &*entities).join() {
            let distance: Vector3<f32> = animation.to_pos.sub(&animation.from_pos);

            let elapsed: u128 = animation.start_time.elapsed().as_millis();
            let animation_progress = (elapsed as f64 / animation.duration as f64) as f32;

            let scaled_distance = distance.scale(animation_progress as f32);

            if animation_progress < 1. {
                transform.set_translation(&animation.from_pos + scaled_distance);
                let scale: f32 = animation.start_scale + (animation.end_scale - animation.start_scale)*animation_progress;
                println!("Progress {} = {} / {}", animation_progress, animation.duration, elapsed);
                println!("Scale {}", scale);
                println!("append {:?}", scaled_distance);
                transform.set_scale(Vector3::new(scale,scale,1.));
            } else {
                finished_moving.push(e);
            }
        }

        for e in finished_moving {
           entities.delete(e);
        }
    }
}
