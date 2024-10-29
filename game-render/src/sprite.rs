use crate::{animation::*, constants::*, CustomRenderContext};
use game_model::{piece::*, Point2};
use instant::{Duration, Instant};
use macroquad::{
    prelude::{Color, Rect, Vec2, WHITE},
    texture::{draw_texture_ex, DrawTextureParams},
};

#[derive(Clone, Copy, Debug)]
pub struct Colour {
    pub(crate) r: f32,
    pub(crate) g: f32,
    pub(crate) b: f32,
    pub(crate) a: f32,
}

impl Colour {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const BLACK: Colour = Self::new(0., 0., 0., 1.);
    pub const WHITE: Colour = Self::new(1., 1., 1., 1.);
}

impl From<Color> for Colour {
    fn from(Color { r, g, b, a }: Color) -> Self {
        Self { r, g, b, a }
    }
}

impl From<[f32; 4]> for Colour {
    fn from([r, g, b, a]: [f32; 4]) -> Self {
        Self { r, g, b, a }
    }
}

impl Into<[f32; 4]> for Colour {
    fn into(self) -> [f32; 4] {
        let Colour { r, g, b, a } = self;
        [r, g, b, a]
    }
}

impl Into<Color> for Colour {
    fn into(self) -> Color {
        let Colour { r, g, b, a } = self;
        [r, g, b, a].into()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpriteRender {
    pub from: AnimationPoint,
    pub to: AnimationPoint,
    pub override_color: Option<Colour>,
    pub color: Colour,
    pub rotation: f32,
    sprite_kind: SpriteKind,
    rect_in_sprite: Rect,
}

#[derive(Clone, Copy, Debug)]
pub enum SpriteKind {
    Piece,
    Special,
}

impl SpriteRender {
    pub fn new(
        x_pos: f32,
        y_pos: f32,
        scale: f32,
        color: Colour,
        sprite_kind: SpriteKind,
        rect_in_sprite: Rect,
    ) -> Self {
        let pap = AnimationPoint {
            x_pos,
            y_pos,
            sprite_width: scale,
            instant: Instant::now(),
        };

        Self::animated(pap, pap, color, sprite_kind, rect_in_sprite)
    }

    pub(crate) fn new_at_point(
        point: &Point2,
        sprite_width: f32,
        color: Colour,
        sprite_kind: SpriteKind,
        rect_in_sprite: Rect,
    ) -> SpriteRender {
        let (x_pos, y_pos) = Self::render_pos(sprite_width, point);

        SpriteRender::new(
            x_pos,
            y_pos,
            sprite_width,
            color,
            sprite_kind,
            rect_in_sprite,
        )
    }

    pub(crate) fn for_piece(point: &Point2, piece_kind: PieceKind, color: Colour) -> SpriteRender {
        let mut sprite_render = SpriteRender::new_at_point(
            point,
            PIECE_SCALE,
            color,
            SpriteKind::Piece,
            Self::piece_sprite_rect(piece_kind),
        );

        if piece_kind == PieceKind::HorizontalBar {
            sprite_render.rotation = 1.57;
        }

        sprite_render
    }

    fn render_pos(sprite_width: f32, point: &Point2) -> (f32, f32) {
        let (x_pos, y_pos) = cell_coords(point);
        let shift = (CELL_ABSOLUTE_WIDTH - sprite_width) / 2.;
        (x_pos + shift, y_pos + shift)
    }

    fn animated(
        from: AnimationPoint,
        to: AnimationPoint,
        color: Colour,
        sprite_kind: SpriteKind,
        rect_in_sprite: Rect,
    ) -> Self {
        SpriteRender {
            from,
            to,
            override_color: None,
            color,
            sprite_kind,
            rect_in_sprite,
            rotation: 0.,
        }
    }

    pub fn piece_sprite_rect(piece_kind: PieceKind) -> Rect {
        let (sprite_x, sprite_y) = match piece_kind {
            PieceKind::Simple => (0, 0),
            PieceKind::HorizontalBar => (2, 1),
            PieceKind::VerticalBar => (2, 1),
            PieceKind::Cross => (2, 0),
            PieceKind::Queen => (1, 1),
            PieceKind::Castle => (0, 2),
            PieceKind::Sniper => (0, 1),
        };

        Rect {
            x: sprite_x as f32 * 295. + 250.,
            y: sprite_y as f32 * 255. + 100.,
            w: 295.,
            h: 255.,
        }
    }

    pub fn greyed_out(color: &Colour) -> Colour {
        Colour::new(
            (color.r + WHITE.r * 2.) / 3.,
            (color.g + WHITE.g * 2.) / 3.,
            (color.b + WHITE.b * 2.) / 3.,
            255.,
        )
    }

    pub fn move_towards(&mut self, point: &Point2, speed_ms: u64) {
        self.from = self.to;
        self.from.instant = Instant::now();

        let (x_pos, y_pos) = Self::render_pos(self.from.sprite_width, point);

        self.to = AnimationPoint {
            x_pos,
            y_pos,
            sprite_width: self.from.sprite_width,
            instant: Instant::now() + Duration::from_millis(speed_ms),
        };
    }

    pub fn scale(&mut self, sprite_width: f32, speed_ms: u64) {
        self.from.instant = Instant::now();

        self.to = SpriteRender::scale_animation_point(&self.from, sprite_width);

        self.to.instant = Instant::now() + Duration::from_millis(speed_ms);
    }

    pub fn scale_animation_point(
        animation_point: &AnimationPoint,
        sprite_width: f32,
    ) -> AnimationPoint {
        let shift = (CELL_ABSOLUTE_WIDTH - sprite_width) / 2.;

        AnimationPoint {
            x_pos: animation_point.x_pos + animation_point.sprite_width / 2.
                - CELL_ABSOLUTE_WIDTH / 2.
                + shift,
            y_pos: animation_point.y_pos + animation_point.sprite_width / 2.
                - CELL_ABSOLUTE_WIDTH / 2.
                + shift,
            sprite_width,
            instant: Instant::now(),
        }
    }

    pub fn render(&self, render_context: &CustomRenderContext) {
        let animation = self.from.interpolate(&self.to, Instant::now());

        let texture = match self.sprite_kind {
            SpriteKind::Piece => render_context.pieces_texture,
            SpriteKind::Special => render_context.special_texture,
        };

        draw_texture_ex(
            texture,
            animation.x_pos,
            animation.y_pos,
            self.override_color.unwrap_or(self.color).into(),
            DrawTextureParams {
                dest_size: Some(Vec2::new(animation.sprite_width, animation.sprite_width)),
                source: Some(self.rect_in_sprite),
                rotation: self.rotation,
                ..Default::default()
            },
        );

        /*draw_rectangle_lines(
            animation.x_pos,
            animation.y_pos,
            animation.sprite_width,
            animation.sprite_width,
            2.,
            GREEN
        )*/
    }
}
