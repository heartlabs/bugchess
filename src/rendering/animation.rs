use crate::{
    constants::*,
    rendering::{SpriteKind, SpriteRender},
    BoardRender, PieceKind, Point2,
};
use instant::{Duration, Instant};
use macroquad::{
    color::{Color, WHITE},
    logging::info,
    math::Rect,
    rand::rand,
};
use std::fmt::{Debug, Formatter};

#[derive(Clone, Copy, Debug)]
pub struct AnimationPoint {
    pub(crate) x_pos: f32,
    pub(crate) y_pos: f32,
    pub(crate) sprite_width: f32,
    pub(crate) instant: Instant,
}

impl AnimationPoint {
    pub fn interpolate(&self, towards: AnimationPoint, at_instant: Instant) -> AnimationPoint {
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

    pub fn new_remove(at: Point2) -> Self {
        Animation {
            duration: Duration::from_millis(0),
            finished_at: Instant::now(),
            next_animations: vec![],
            expert: Box::new(RemovePieceAnimation { at }),
        }
    }

    pub fn new_piece(team: usize, to: Point2, piece_kind: PieceKind) -> Self {
        Animation {
            duration: Duration::from_millis(0),
            finished_at: Instant::now(),
            next_animations: vec![],
            expert: Box::new(NewPieceAnimation {
                team,
                to,
                piece_kind,
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

    pub fn append(&mut self, animation: Animation) {
        self.next_animations.push(animation)
    }
}

pub trait AnimationExpert: Debug {
    fn start(&self, board_render: &mut BoardRender);
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
pub struct PlacePieceAnimation {
    team: usize,
    to: Point2,
}

#[derive(Debug, Clone)]
pub struct NewPieceAnimation {
    pub(crate) team: usize,
    pub(crate) to: Point2,
    pub(crate) piece_kind: PieceKind,
}

#[derive(Debug, Clone)]
pub struct RemovePieceAnimation {
    pub at: Point2,
}

#[derive(Debug, Clone)]
pub struct RemoveSpriteAnimation {
    pub id: u32,
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
        PlacePieceAnimation { team, to }
    }
}

impl AnimationExpert for PlacePieceAnimation {
    fn start(&self, board_render: &mut BoardRender) {
        let mut unused = board_render.unused_pieces[self.team]
            .pop()
            .expect("No unused piece left in BoardRender");
        unused.move_towards(&self.to, PLACE_PIECE_SPEED);
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

        let (x, y) = cell_coords(&self.from);

        sprite_render.scale(self.span_cells * CELL_ABSOLUTE_WIDTH, BULLET_SPEED);

        board_render.special_sprites.insert(self.id, sprite_render);
    }
}

impl AnimationExpert for RemoveSpriteAnimation {
    fn start(&self, board_render: &mut BoardRender) {
        board_render.special_sprites.remove(&self.id);
    }
}
