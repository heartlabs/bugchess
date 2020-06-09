use amethyst::{
    renderer::{
        resources::Tint,
        palette::Srgba,
    },
    ecs::{Join, ReadStorage, System, WriteStorage, ReadExpect, Entities},
};
use crate::components::{Activatable};
use crate::components::board::{BoardPosition, Target, Highlight, HighlightType};
use crate::components::active::{Selected, Hovered};
use crate::resources::board::Board;

pub struct TargetHighlightingSystem;

impl<'a> System<'a> for TargetHighlightingSystem {
    type SystemData = (
        WriteStorage<'a, Tint>,
        ReadStorage<'a, Target>,
        WriteStorage<'a, Highlight>,
        ReadStorage<'a, BoardPosition>,
        ReadStorage<'a, Selected>,
        ReadStorage<'a, Hovered>,
        ReadExpect<'a, Board>,
        Entities<'a>,
    );

    fn run(&mut self, (mut tints, targets, mut highlights, positions, selected, hovered, board, entities): Self::SystemData) {
        let mut s = None;
        let mut h = None;

        for (_target, mut highlight, pos, e) in (&targets, &mut highlights, &positions, &*entities).join() { // TODO: hovered and selected should be resources
            highlight.types.clear();
            if let Some(sel) = selected.get(e) {
                // tints.insert(e, Tint());
                highlight.types.push(HighlightType::Selected);
                s = board.get_piece(pos.coords.x, pos.coords.y);
            } else if let Some(hov) = hovered.get(e) {

                // tints.insert(e, Tint());
                highlight.types.push(HighlightType::Hovered);
                h = board.get_piece(pos.coords.x, pos.coords.y);
            } else {
                // tints.remove(e);
            }
        }

        for (target, mut highlight, target_entity) in (&targets, &mut highlights, &*entities).join() {
            if let Some(selected_piece) = s {

                if (target.is_possible_target_of(selected_piece)){
                    highlight.types.push(HighlightType::TargetOfSelected);
                    //tints.insert(target_entity, Tint(Srgba::new(0.1, 0.9, 0.1, 0.5)));
                }
            }
            if let Some(hovered_piece) = h {

                if (target.is_possible_target_of(hovered_piece)){
                    highlight.types.push(HighlightType::TargetOfHovered);
                    //tints.insert(target_entity, Tint(Srgba::new(0.4, 0.9, 0.7, 0.3)));
                }
            }
        }
    }
}
