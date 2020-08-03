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
use crate::components::bounded::MovingTowards;
use crate::resources::board::Board;

pub struct MoveToPosition;

impl<'a> System<'a> for MoveToPosition {
    type SystemData = (
        ReadStorage<'a, Cell>,
        ReadStorage<'a, BoardPosition>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, MovingTowards>,
        Entities<'a>,
        ReadExpect<'a, Board>,
    );

    fn run(&mut self, (cells, positions, mut transforms, mut move_to, entities, board): Self::SystemData) {
        for (_cell, pos, e) in (!&cells, &positions, &*entities).join() {
            let cell_at_position = board.get_cell(pos.coords.x, pos.coords.y);
            let mut cell_translation = transforms.get(cell_at_position).unwrap().translation().clone_owned();
            cell_translation.z = 0.1;

            if transforms.contains(e) {
                move_to.insert(e, MovingTowards{screen_pos: cell_translation});
            } else {
                let mut transform = Transform::default();
                transform.set_translation_xyz(cell_translation.x, cell_translation.y, cell_translation.z);
                transforms.insert(e, transform);
            }
        }

        let mut finished_moving = Vec::new();

        for (transform, target_screen_pos, e) in (&mut transforms, &mut move_to, &*entities).join() {
            let distance = target_screen_pos.screen_pos.sub(transform.translation());

            //if let Some(normalized_distance) = distance.try_normalize(Vector3::new(0.1 as f32,0.1 as f32,0.1 as f32).norm()) {
            let length = distance.norm();
            let scaled_distance = distance.scale(50./length);

            if length > scaled_distance.norm() {
                transform.append_translation(scaled_distance);
            } else {
                transform.set_translation_xyz(
                    target_screen_pos.screen_pos.x,
                    target_screen_pos.screen_pos.y,
                    target_screen_pos.screen_pos.z);
                transform.set_scale(Vector3::new(1.,1.,1.));

                finished_moving.push(e);
            }
        }

        for e in finished_moving {
            move_to.remove(e);
        }
    }
}
