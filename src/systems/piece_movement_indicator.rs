use amethyst::{
    renderer::{
        resources::Tint,
        palette::{Srgba,Blend,Alpha, rgb::Rgb, encoding::srgb::Srgb},
    },
    ecs::{Join, ReadStorage, System, WriteStorage, Entities},
};
use crate::components::{Activatable};
use crate::components::active::{Selected, Hovered};
use crate::components::board::{Highlight, HighlightType};


pub struct PieceMovement;

impl<'a> System<'a> for PieceMovement {
    type SystemData = (
        WriteStorage<'a, Tint>,
        ReadStorage<'a, Highlight>,
        Entities<'a>,
    );

    fn run(&mut self, (mut tints, highlights, entities): Self::SystemData) {
        for (highlight, e) in (&highlights, &*entities).join() {
            let mut tint_color = Option::None;
            //let mut tint_color = Srgba::new(0., 0., 0., 0.);

            for highlight_type in &highlight.types {
                let h = match highlight_type {
                    HighlightType::Selected => Srgba::new(0.3, 0.4, 0.4, 0.5),
                    HighlightType::Hovered => Srgba::new(0.4, 1., 1., 0.5),
                    HighlightType::TargetOfHovered => Srgba::new(0.4, 1., 1., 0.5),
                    HighlightType:: TargetOfSelected => Srgba::new(1.0, 0.4, 0.4, 0.5),
                };

                tint_color = Some(h);
            }

            if let Some(t) = tint_color{
                tints.insert(e, Tint(t));
            } else {
                tints.remove(e);
            }
        }
    }
}
