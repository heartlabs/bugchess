use crate::{
    constants::*,
    rendering::{EffectRender, SpriteKind, SpriteRender,BoardRender},
};
use game_model::{
    board::Point2,
    piece::{EffectKind, Exhaustion, PieceKind},
};
use instant::{Duration, Instant};
use macroquad::{color::WHITE, logging::info, math::Rect, rand::rand};
use std::{
    fmt::Debug,
    ops::{Add, Mul, Sub},
};

#[derive(Clone, Copy, Debug)]
pub struct AnimationPoint {
    pub(crate) x_pos: f32,
    pub(crate) y_pos: f32,
    pub(crate) sprite_width: f32,
    pub(crate) instant: Instant,
}

impl AnimationPoint {
    pub fn interpolate(&self, towards: &AnimationPoint, at_instant: Instant) -> AnimationPoint {
        let progress = Self::calculate_progress(&self.instant, &towards.instant, &at_instant);

        let animation_point = AnimationPoint {
            x_pos: Self::interpolate_value(self.x_pos, towards.x_pos, progress),
            y_pos: Self::interpolate_value(self.y_pos, towards.y_pos, progress),
            sprite_width: Self::interpolate_value(
                self.sprite_width,
                towards.sprite_width,
                progress,
            ),
            instant: at_instant,
        };

        animation_point
    }

    pub fn calculate_progress(
        from_instant: &Instant,
        towards_instant: &Instant,
        at_instant: &Instant,
    ) -> f32 {
        if at_instant < from_instant {
            0.
        } else if at_instant > towards_instant {
            1.
        } else {
            let diff = towards_instant.duration_since(*from_instant);
            let local_elapsed = at_instant.duration_since(*from_instant);

            //println!("interpolating after {}ms diff is {} and local_elapsed {} leading to {}", elapsed_time_ms, diff, local_elapsed, local_elapsed as f32 / diff as f32);

            local_elapsed.as_millis() as f32 / diff.as_millis() as f32
        }
    }

    pub fn interpolate_value<T>(from: T, to: T, progress: f32) -> T
    where
        T: Mul<f32, Output = T> + Add<Output = T> + Sub<Output = T> + Copy,
    {
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
    pub expert: Box<dyn AnimationExpert>,
}

impl Animation {
    pub fn new(expert: Box<dyn AnimationExpert>) -> Self {
        Animation {
            duration: Duration::from_millis(ANIMATION_SPEED),
            finished_at: Instant::now(),
            next_animations: vec![],
            expert,
        }
    }

    pub fn new_place(team: usize, to: Point2) -> Self {
        Animation {
            duration: Duration::from_millis(ANIMATION_SPEED),
            finished_at: Instant::now(),
            next_animations: vec![],
            expert: Box::new(PlacePieceAnimation { team, to }),
        }
    }

    pub fn new_add_effect(effect: EffectKind, at: Point2) -> Self {
        Animation {
            duration: Duration::from_millis(0),
            finished_at: Instant::now(),
            next_animations: vec![],
            expert: Box::new(AddEffectAnimation { effect, at }),
        }
    }

    pub fn new_remove_effect(effect: EffectKind, at: Point2) -> Self {
        Animation {
            duration: Duration::from_millis(0),
            finished_at: Instant::now(),
            next_animations: vec![],
            expert: Box::new(RemoveEffectAnimation { effect, at }),
        }
    }

    pub fn new_exhaustion(to: Exhaustion, at: Point2) -> Self {
        Animation {
            duration: Duration::from_millis(0),
            finished_at: Instant::now(),
            next_animations: vec![],
            expert: Box::new(ExhaustAnimation { to, at }),
        }
    }

    pub fn new_add_unused(team_id: usize) -> Self {
        Animation {
            duration: Duration::from_millis(ADD_UNUSED_SPEED),
            finished_at: Instant::now(),
            next_animations: vec![],
            expert: Box::new(AddUnusedAnimation { team_id }),
        }
    }

    pub fn new_remove(at: Point2) -> Self {
        Animation {
            duration: Duration::from_millis(0),
            finished_at: Instant::now(),
            next_animations: vec![],
            expert: Box::new(RemovePieceAnimation { at }),
        }
    }

    pub fn new_piece(team: usize, to: Point2, piece_kind: PieceKind, exhausted: bool) -> Self {
        Animation {
            duration: Duration::from_millis(0),
            finished_at: Instant::now(),
            next_animations: vec![],
            expert: Box::new(NewPieceAnimation {
                team,
                to,
                piece_kind,
                exhausted,
            }),
        }
    }
    pub fn new_move(from: Point2, to: Point2) -> Self {
        Animation {
            duration: Duration::from_millis(MOVE_PIECE_SPEED),
            finished_at: Instant::now(),
            next_animations: vec![],
            expert: Box::new(MovePieceAnimation { from, to }),
        }
    }

    pub fn new_move_towards(from: Point2, to: Point2) -> Self {
        Animation {
            duration: Duration::from_millis(MOVE_PIECE_SPEED),
            finished_at: Instant::now(),
            next_animations: vec![],
            expert: Box::new(SwooshPieceAnimation { from, to }),
        }
    }

    pub fn new_bullet(from: Point2, to: Point2) -> Self {
        let bullet = BulletAnimation {
            from,
            to,
            id: rand(),
        };

        Animation {
            duration: Duration::from_millis(BULLET_SPEED),
            finished_at: Instant::now(),
            next_animations: vec![Animation::new(Box::new(RemoveSpriteAnimation {
                id: bullet.id,
            }))],
            expert: Box::new(bullet),
        }
    }

    pub fn new_blast(from: Point2) -> Self {
        let bullet = BlastAnimation {
            from,
            span_cells: 3.,
            id: rand(),
        };

        Animation {
            duration: Duration::from_millis(BULLET_SPEED),
            finished_at: Instant::now(),
            next_animations: vec![Animation::new(Box::new(RemoveSpriteAnimation {
                id: bullet.id,
            }))],
            expert: Box::new(bullet),
        }
    }

    pub(crate) fn start(&mut self, board_render: &mut BoardRender) {
        self.finished_at = Instant::now() + self.duration;
        //info!("Starting {:?}", self.expert);
        self.expert.start(board_render);
    }
}

pub trait AnimationExpert: Debug {
    fn start(&self, board_render: &mut BoardRender);
}

#[derive(Debug, Clone)]
pub struct AddEffectAnimation {
    pub(crate) effect: EffectKind,
    pub at: Point2,
}

#[derive(Debug, Clone)]
pub struct RemoveEffectAnimation {
    pub(crate) effect: EffectKind,
    pub at: Point2,
}

#[derive(Debug, Clone)]
pub struct ExhaustAnimation {
    pub(crate) to: Exhaustion,
    pub at: Point2,
}

#[derive(Debug, Clone)]
pub struct BulletAnimation {
    pub(crate) from: Point2,
    pub(crate) to: Point2,
    pub id: u32,
}

#[derive(Debug, Clone)]
pub struct BlastAnimation {
    pub(crate) from: Point2,
    pub(crate) span_cells: f32,
    pub id: u32,
}

#[derive(Debug, Clone)]
pub struct MovePieceAnimation {
    pub(crate) from: Point2,
    pub(crate) to: Point2,
}

#[derive(Debug, Clone)]
pub struct SwooshPieceAnimation {
    pub(crate) from: Point2,
    pub(crate) to: Point2,
}

#[derive(Debug, Clone)]
pub struct PlacePieceAnimation {
    team: usize,
    to: Point2,
}

#[derive(Debug, Clone)]
pub struct NewPieceAnimation {
    pub(crate) team: usize,
    pub(crate) to: Point2,
    pub(crate) piece_kind: PieceKind,
    pub exhausted: bool,
}

#[derive(Debug, Clone)]
pub struct RemovePieceAnimation {
    pub at: Point2,
}

#[derive(Debug, Clone)]
pub struct RemoveSpriteAnimation {
    pub id: u32,
}

#[derive(Debug, Clone)]
pub struct AddUnusedAnimation {
    pub team_id: usize,
}

#[derive(Debug, Clone)]
pub struct RemoveUnusedAnimation {
    pub team_id: usize,
}

impl AnimationExpert for AddUnusedAnimation {
    fn start(&self, board_render: &mut BoardRender) {
        board_render.add_unused_piece(self.team_id);
        let sprite_render = board_render
            .unused_pieces
            .get_mut(self.team_id)
            .unwrap()
            .last_mut()
            .unwrap();
        sprite_render.from = SpriteRender::scale_animation_point(&sprite_render.from, 100.);
        sprite_render.from.instant = Instant::now();
        sprite_render.to.instant = Instant::now() + Duration::from_millis(ADD_UNUSED_SPEED);
    }
}
impl AnimationExpert for RemoveUnusedAnimation {
    fn start(&self, board_render: &mut BoardRender) {
        board_render.unused_pieces.pop();
    }
}

impl AnimationExpert for NewPieceAnimation {
    fn start(&self, board_render: &mut BoardRender) {
        board_render.add_placed_piece(&self.to, self.piece_kind, self.team, self.exhausted)
    }
}

impl AnimationExpert for RemovePieceAnimation {
    fn start(&self, board_render: &mut BoardRender) {
        board_render.placed_pieces.remove(&self.at);
    }
}

impl PlacePieceAnimation {
    pub fn new(team: usize, to: Point2) -> Self {
        PlacePieceAnimation { team, to }
    }
}

impl AnimationExpert for PlacePieceAnimation {
    fn start(&self, board_render: &mut BoardRender) {
        let mut unused = board_render.unused_pieces[self.team]
            .pop()
            .expect("No unused piece left in BoardRender");
        unused.move_towards(&self.to, PLACE_PIECE_SPEED);
        unused.override_color = Some(SpriteRender::greyed_out(&unused.color));

        //let color = board_render.team_colors[piece.team_id];
        //board_render.placed_pieces.push(PieceRender::from_piece(point, piece, color))
        board_render.placed_pieces.insert(self.to, unused);
    }
}

impl AnimationExpert for MovePieceAnimation {
    fn start(&self, board_render: &mut BoardRender) {
        let mut piece_render = board_render
            .placed_pieces
            .remove(&self.from)
            .expect(&*format!("No piece found at {:?}", self.from));

        piece_render.move_towards(&self.to, MOVE_PIECE_SPEED);

        board_render.placed_pieces.insert(self.to, piece_render);
    }
}
impl AnimationExpert for SwooshPieceAnimation {
    fn start(&self, board_render: &mut BoardRender) {
        let piece_render = board_render
            .placed_pieces
            .get_mut(&self.from)
            .expect(&*format!("No piece found at {:?}", self.from));

        piece_render.move_towards(&self.to, MOVE_PIECE_SPEED);
    }
}

impl AnimationExpert for AddEffectAnimation {
    fn start(&self, board_render: &mut BoardRender) {
        info!("Starting addeffectanim");

        if !board_render.effects.contains_key(&self.at) {
            board_render.effects.insert(self.at, vec![]);
        }

        board_render
            .effects
            .get_mut(&self.at)
            .unwrap()
            .push(EffectRender::new());
    }
}

impl AnimationExpert for RemoveEffectAnimation {
    fn start(&self, board_render: &mut BoardRender) {
        board_render
            .effects
            .get_mut(&self.at)
            .expect(
                format!(
                    "Can't remove effect at {:?} because that position doesn't exist",
                    self.at
                )
                .as_str(),
            )
            .remove(0);
    }
}

impl AnimationExpert for ExhaustAnimation {
    fn start(&self, board_render: &mut BoardRender) {
        let mut sprite_render = board_render
            .placed_pieces
            .get_mut(&self.at)
            .expect(&*format!("No piece found at {:?}", self.at));

        if self.to.is_done() {
            sprite_render.override_color = Some(SpriteRender::greyed_out(&sprite_render.color));
        } else {
            sprite_render.override_color = None;
        }
    }
}

impl AnimationExpert for BulletAnimation {
    fn start(&self, board_render: &mut BoardRender) {
        let mut sprite_render = SpriteRender::new_at_point(
            &self.from,
            PIECE_SCALE,
            WHITE,
            SpriteKind::Special,
            Rect::new(0., 0., SPRITE_WIDTH, SPRITE_WIDTH),
        );
        sprite_render.move_towards(&self.to, BULLET_SPEED);
        board_render.special_sprites.insert(self.id, sprite_render);
    }
}

impl AnimationExpert for BlastAnimation {
    fn start(&self, board_render: &mut BoardRender) {
        let mut sprite_render = SpriteRender::new_at_point(
            &self.from,
            0.,
            WHITE,
            SpriteKind::Special,
            Rect::new(SPRITE_WIDTH, 0., SPRITE_WIDTH, SPRITE_WIDTH),
        );

        sprite_render.scale(self.span_cells * CELL_ABSOLUTE_WIDTH, BULLET_SPEED);

        board_render.special_sprites.insert(self.id, sprite_render);
    }
}

impl AnimationExpert for RemoveSpriteAnimation {
    fn start(&self, board_render: &mut BoardRender) {
        board_render.special_sprites.remove(&self.id);
    }
}
