use amethyst::{
    renderer::{
        resources::Tint,
        palette::Srgba,
    },
    ecs::{Join, ReadStorage, System, WriteStorage, Entities},
};
use crate::components::{Activatable};

pub struct ShowAsActive;

impl<'a> System<'a> for ShowAsActive {
    type SystemData = (
        WriteStorage<'a, Tint>,
        ReadStorage<'a, Activatable>,
        Entities<'a>,
    );

    fn run(&mut self, (mut tints, activatables, entities): Self::SystemData) {
        for (activatable, e) in (&activatables, &*entities).join() {
            if activatable.active {
                tints.insert(e, Tint(Srgba::new(1.0, 0.4, 0.4, 0.5)));
            } else {
                tints.remove(e);
            }
        }
    }
}
