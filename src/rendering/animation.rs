use std::fmt::{Debug, Formatter};
use instant::{Duration, Instant};
use macroquad::logging::info;
use crate::{constants::*, BoardRender, Point2, PieceKind};
use crate::rendering::PieceRender;


#[derive(Clone, Copy, Debug)]
pub struct AnimationPoint {
    pub(crate) x_pos: f32,
    pub(crate) y_pos: f32,
    pub(crate) scale: f32,
    pub(crate) instant: Instant,
}

impl AnimationPoint {
    pub fn interpolate(
        &self,
        towards: AnimationPoint,
        at_instant: Instant,
    ) -> AnimationPoint {
        //let elapsed_time_ms = elapsed_time.as_millis() as u32;
        let progress = if at_instant < self.instant {
            0.
        } else if at_instant > towards.instant {
            1.
        } else {
            let diff = towards.instant.duration_since(self.instant);
            let local_elapsed = at_instant.duration_since(self.instant);

            //println!("interpolating after {}ms diff is {} and local_elapsed {} leading to {}", elapsed_time_ms, diff, local_elapsed, local_elapsed as f32 / diff as f32);

            local_elapsed.as_millis() as f32 / diff.as_millis() as f32
        };

        AnimationPoint {
            x_pos: Self::interpolate_value(self.x_pos, towards.x_pos, progress),
            y_pos: Self::interpolate_value(self.y_pos, towards.y_pos, progress),
            scale: Self::interpolate_value(self.scale, towards.scale, progress),
            instant: at_instant,
        }
    }

    fn interpolate_value(from: f32, to: f32, progress: f32) -> f32 {
        let diff = to - from;
        let p = diff * progress;
        from + p
    }
}

#[derive(Debug)]
pub struct Animation {
    pub duration: Duration,
    pub finished_at: Instant,
    pub next_animations: Vec<Animation>,
    pub expert: Box<dyn AnimationExpert>
}

impl Animation {
    pub fn new(expert: Box<dyn AnimationExpert>) -> Self {
        Animation {
            duration: Duration::from_millis(ANIMATION_SPEED),
            finished_at: Instant::now(),
            next_animations: vec![],
            expert
        }
    }
    pub(crate) fn start(&mut self, board_render: &mut BoardRender) {
        self.finished_at = Instant::now() + self.duration;
        //info!("Starting {:?}", self.expert);
        self.expert.start(board_render);
    }
}

pub trait AnimationExpert : Debug {
    fn start(&self, board_render: &mut BoardRender);
}

#[derive(Debug)]
pub struct MovePieceAnimation {
    pub(crate) from: Point2,
    pub(crate) to: Point2,
}

#[derive(Debug)]
pub struct PlacePieceAnimation {
    team: usize,
    to: Point2,
}

#[derive(Debug)]
pub struct NewPieceAnimation {
    pub(crate) team: usize,
    pub(crate) to: Point2,
    pub(crate) piece_kind: PieceKind
}

#[derive(Debug)]
pub struct RemovePieceAnimation {
    pub at: Point2
}

impl AnimationExpert for NewPieceAnimation {
    fn start(&self, board_render: &mut BoardRender) {
        board_render.add_placed_piece(&self.to, self.piece_kind, self.team)
    }
}

impl AnimationExpert for RemovePieceAnimation {
    fn start(&self, board_render: &mut BoardRender) {
        board_render.placed_pieces.remove(&self.at);
    }
}

impl PlacePieceAnimation {
    pub fn new(team: usize, to: Point2) -> Self {
        PlacePieceAnimation {
            team,
            to
        }
    }
}

impl AnimationExpert for PlacePieceAnimation {
    fn start(&self, board_render: &mut BoardRender) {
        let mut unused = board_render.unused_pieces[self.team]
            .pop()
            .expect("No unused piece left in BoardRender");
        unused.move_towards(&self.to);
        //let color = board_render.team_colors[piece.team_id];
        //board_render.placed_pieces.push(PieceRender::from_piece(point, piece, color))
        board_render.placed_pieces.insert(self.to, unused);
    }
}
impl AnimationExpert for MovePieceAnimation {
    fn start(&self, board_render: &mut BoardRender) {
        let mut piece_render = board_render.placed_pieces.remove(&self.from)
            .expect(&*format!("No piece found at {:?}", self.from));

        piece_render.move_towards(&self.to);

        board_render.placed_pieces.insert(self.to, piece_render);
    }
}
