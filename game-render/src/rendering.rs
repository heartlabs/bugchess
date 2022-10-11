use crate::{animation::*, constants::*, ui::Button};
use game_events::core_game::CoreGameSubstate;
use game_model::{board::*, game::*, piece::*, ranges::*};
use instant::{Duration, Instant};
use macroquad::prelude::*;
use macroquad_canvas::Canvas2D;
use std::collections::{HashMap, VecDeque};
#[derive(Debug, Clone)]
pub struct CustomRenderContext {
    pieces_texture: Texture2D,
    special_texture: Texture2D,
    pub game_state: CoreGameSubstate,
    pub button_next: Button,
    pub button_undo: Button,
}

impl CustomRenderContext {
    pub fn new() -> Self {
        CustomRenderContext {
            pieces_texture: Texture2D::from_file_with_format(
                include_bytes!("../resources/sprites/pieces.png"),
                None,
            ),
            special_texture: Texture2D::from_file_with_format(
                include_bytes!("../resources/sprites/special.png"),
                None,
            ),
            game_state: CoreGameSubstate::Place,
            button_next: Button::new(10., "End Turn".to_string()),
            button_undo: Button::new(120., "Undo".to_string()),
        }
    }
}

pub struct BoardRender {
    pub(crate) special_sprites: HashMap<u32, SpriteRender>,
    pub(crate) unused_pieces: Vec<Vec<SpriteRender>>,
    pub(crate) placed_pieces: HashMap<Point2, SpriteRender>,
    pub(crate) effects: HashMap<Point2, Vec<EffectRender>>,
    pub(crate) team_colors: Vec<Color>,
    next_animations: VecDeque<Vec<Animation>>,
    current_animations: Vec<Animation>,
}

impl BoardRender {
    pub fn new(game: &Game) -> Self {
        let unused_pieces = vec![vec![], vec![]];
        let mut placed_pieces = HashMap::new();

        let board = &game.board;

        let team_colors = vec![
            Color::new(0.76, 0.17, 0.10, 1.),
            Color::new(0.90, 0.68, 0.15, 1.),
        ];

        board.for_each_placed_piece(|point, piece| {
            placed_pieces.insert(
                point,
                SpriteRender::for_piece(&point, piece.piece_kind, team_colors[piece.team_id]),
            );
        });

        BoardRender {
            unused_pieces,
            placed_pieces,
            team_colors,
            special_sprites: HashMap::new(),
            next_animations: VecDeque::new(),
            effects: HashMap::new(),
            current_animations: vec![],
        }
    }

    pub fn get_team_color(&self, index: usize) -> &Color {
        self.team_colors
            .get(index)
            .expect("Team with that index does not exist")
    }

    pub fn add_unused_piece(&mut self, team_id: usize) {
        let unused_pieces = &mut self.unused_pieces[team_id];

        let (x_pos, y_pos) = if team_id == 0 {
            let (upb_x, mut upb_y) = cell_coords_tuple(BOARD_WIDTH, BOARD_HEIGHT - 1);
            upb_y += CELL_ABSOLUTE_WIDTH / 4.;

            (upb_x, upb_y - unused_pieces.len() as f32 * 32.)
        } else {
            let (mut upb_x, upb_y) = cell_coords_tuple(0, 0);
            upb_x -= CELL_ABSOLUTE_WIDTH / 1.25;

            (upb_x, upb_y + unused_pieces.len() as f32 * 32.)
        };

        unused_pieces.push(SpriteRender::new(
            x_pos,
            y_pos,
            PIECE_SCALE,
            self.team_colors[team_id],
            SpriteKind::Piece,
            SpriteRender::piece_sprite_rect(PieceKind::Simple),
        ));
    }

    pub fn add_animation_sequence(&mut self, animations: Vec<Animation>) {
        self.next_animations.push_back(animations);
    }

    pub fn add_placed_piece(
        &mut self,
        point: &Point2,
        piece_kind: PieceKind,
        team_id: usize,
        exhausted: bool,
    ) {
        let mut piece_render =
            SpriteRender::for_piece(point, piece_kind, self.team_colors[team_id]);
        if exhausted {
            piece_render.override_color = Some(SpriteRender::greyed_out(&piece_render.color));
        }
        self.placed_pieces.insert(*point, piece_render);
    }

    pub fn update(&mut self) {
        let mut new_animations = vec![];
        self.current_animations
            .iter_mut()
            .filter(|a| a.finished_at <= Instant::now())
            .for_each(|a| {
                new_animations.append(&mut a.next_animations);
            });

        let anim_count = self.current_animations.len();
        self.current_animations
            .retain(|a| a.finished_at > Instant::now());

        if anim_count != self.current_animations.len() || !new_animations.is_empty() {
            //     info!("animation count {} -> {}; {} new", anim_count, self.animations.len(), new_animations.len());
        }

        for animation in new_animations.iter_mut() {
            animation.start(self);
        }

        self.current_animations.append(&mut new_animations);

        if self.current_animations.is_empty() {
            if let Some(mut animations) = self.next_animations.pop_front() {
                for animation in animations.iter_mut() {
                    animation.start(self);
                }

                self.current_animations = animations;
            }
        }
    }

    pub fn render(&self, board: &Board, render_context: &CustomRenderContext, canvas: &Canvas2D) {
        Self::render_cells(board, canvas);

        for (point, effects) in &self.effects {
            effects.iter().for_each(|e| e.render(point));
        }

        Self::render_highlights(board, render_context, canvas);

        //println!("rendered {:?}", self.unused_pieces.len());
        self.unused_pieces
            .iter()
            .flat_map(|p| p.iter())
            .for_each(|p| {
                p.render(render_context);
            });

        self.placed_pieces
            .iter()
            .for_each(|(_, p)| p.render(render_context));

        self.special_sprites
            .values()
            .for_each(|s| s.render(render_context));

        render_context.button_next.render(canvas);
        render_context.button_undo.render(canvas);
    }

    fn render_highlights(board: &Board, render_context: &CustomRenderContext, canvas: &Canvas2D) {
        let mut selected_point_option = Option::None;
        let hovered_point = cell_hovered(canvas);
        if let Some(hovered_piece) = board.get_piece_at(&hovered_point) {
            let range_context = match render_context.game_state {
                CoreGameSubstate::Place => RangeContext::Moving(*hovered_piece),
                CoreGameSubstate::Move(selected_point) => {
                    selected_point_option = Option::Some(selected_point);
                    RangeContext::Moving(*hovered_piece)
                }
                CoreGameSubstate::Activate(selected_point) => {
                    selected_point_option = Option::Some(selected_point);
                    RangeContext::Special(*hovered_piece)
                }
                CoreGameSubstate::Won(_) => RangeContext::Moving(*hovered_piece),
                CoreGameSubstate::Wait => RangeContext::Moving(*hovered_piece),
            };

            if let Some(m) = hovered_piece.movement.as_ref() {
                let range = m.range;
                Self::highlight_range(
                    board,
                    &hovered_point,
                    &range_context,
                    &range,
                    Color::from_rgba(90, 220, 90, 100),
                )
            }
        }

        if let Some(selected_point) = selected_point_option {
            if let Some(selected_piece) = board.get_piece_at(&selected_point) {
                let range_context = match render_context.game_state {
                    CoreGameSubstate::Place => RangeContext::Moving(*selected_piece),
                    CoreGameSubstate::Move(_) => RangeContext::Moving(*selected_piece),
                    CoreGameSubstate::Activate(_) => RangeContext::Special(*selected_piece),
                    CoreGameSubstate::Won(_) => RangeContext::Moving(*selected_piece),
                    CoreGameSubstate::Wait => RangeContext::Moving(*selected_piece),
                };
                let range_option: Option<Range> = match render_context.game_state {
                    CoreGameSubstate::Place => Option::None,
                    CoreGameSubstate::Move(_) => selected_piece.movement.map(|m| m.range),
                    CoreGameSubstate::Activate(_) => selected_piece.activatable.map(|m| m.range),
                    CoreGameSubstate::Won(_) => selected_piece.movement.map(|m| m.range),
                    CoreGameSubstate::Wait => Option::None,
                };
                if let Some(range) = range_option {
                    Self::highlight_range(
                        board,
                        &selected_point,
                        &range_context,
                        &range,
                        Color::from_rgba(0, 150, 0, 150),
                    )
                }
            }
        }
    }

    fn render_cells(board: &Board, canvas: &Canvas2D) {
        board.for_each_cell(|cell| {
            let (x_pos, y_pos) = cell_coords(&cell.point);

            let mouse_point = cell_hovered(canvas);

            let color = if cell.point == mouse_point {
                BLUE
            } else if (cell.point.x + cell.point.y + 1) % 2 == 0 {
                Color::from_rgba(187, 173, 160, 255)
            } else {
                Color::from_rgba(238, 228, 218, 255)
            };
            draw_rectangle(
                x_pos,
                y_pos,
                CELL_ABSOLUTE_WIDTH,
                CELL_ABSOLUTE_WIDTH,
                color,
            );
            draw_rectangle_lines(
                x_pos,
                y_pos,
                CELL_ABSOLUTE_WIDTH,
                CELL_ABSOLUTE_WIDTH,
                1.,
                BLACK,
            );
        });
    }

    fn highlight_range(
        board: &Board,
        source_point: &Point2,
        range_context: &RangeContext,
        range: &Range,
        color: Color,
    ) {
        for point in range
            .reachable_points(source_point, board, range_context)
            .iter()
        {
            let (x_pos, y_pos) = cell_coords(&point);

            let mut used_color = color;

            if let Some(_piece) = board.get_piece_at(point) {
                used_color = Color {
                    r: 1.,
                    ..used_color
                }
            }

            draw_rectangle(
                x_pos,
                y_pos,
                CELL_ABSOLUTE_WIDTH,
                CELL_ABSOLUTE_WIDTH,
                used_color,
            );
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EffectRender {
    pub from_color: Color,
    pub towards_color: Color,
    pub from_instant: Instant,
    pub towards_instant: Instant,
}

#[derive(Clone, Copy, Debug)]
pub struct SpriteRender {
    pub from: AnimationPoint,
    pub to: AnimationPoint,
    pub override_color: Option<Color>,
    pub color: Color,
    sprite_kind: SpriteKind,
    rect_in_sprite: Rect,
}

#[derive(Clone, Copy, Debug)]
pub enum SpriteKind {
    Piece,
    Special,
}

impl EffectRender {
    pub fn new() -> Self {
        EffectRender {
            from_color: Color::new(80., 0., 100., 0.0),
            towards_color: Color::new(80., 0., 100., 0.6),
            from_instant: Instant::now(),
            towards_instant: Instant::now() + Duration::from_millis(ANIMATION_SPEED * 3),
        }
    }

    pub fn render(&self, at: &Point2) {
        let (x_pos, y_pos) = cell_coords(at);
        let progress = AnimationPoint::calculate_progress(
            &self.from_instant,
            &self.towards_instant,
            &Instant::now(),
        );
        draw_rectangle(
            x_pos,
            y_pos,
            CELL_ABSOLUTE_WIDTH,
            CELL_ABSOLUTE_WIDTH,
            Color {
                r: AnimationPoint::interpolate_value(
                    self.from_color.r,
                    self.towards_color.r,
                    progress,
                ),
                g: AnimationPoint::interpolate_value(
                    self.from_color.g,
                    self.towards_color.g,
                    progress,
                ),
                b: AnimationPoint::interpolate_value(
                    self.from_color.b,
                    self.towards_color.b,
                    progress,
                ),
                a: AnimationPoint::interpolate_value(
                    self.from_color.a,
                    self.towards_color.a,
                    progress,
                ),
            },
        );
    }
}

impl SpriteRender {
    pub fn new(
        x_pos: f32,
        y_pos: f32,
        scale: f32,
        color: Color,
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
        color: Color,
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

    pub(crate) fn for_piece(point: &Point2, piece_kind: PieceKind, color: Color) -> SpriteRender {
        SpriteRender::new_at_point(
            point,
            PIECE_SCALE,
            color,
            SpriteKind::Piece,
            Self::piece_sprite_rect(piece_kind),
        )
    }

    fn render_pos(sprite_width: f32, point: &Point2) -> (f32, f32) {
        let (x_pos, y_pos) = cell_coords(point);
        let shift = (CELL_ABSOLUTE_WIDTH - sprite_width) / 2.;
        (x_pos + shift, y_pos + shift)
    }

    fn animated(
        from: AnimationPoint,
        to: AnimationPoint,
        color: Color,
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
        }
    }

    fn piece_sprite_rect(piece_kind: PieceKind) -> Rect {
        let (sprite_x, sprite_y) = match piece_kind {
            PieceKind::Simple => (0, 0),
            PieceKind::HorizontalBar => (1, 0),
            PieceKind::VerticalBar => (0, 1),
            PieceKind::Cross => (1, 1),
            PieceKind::Queen => (0, 2),
            PieceKind::Castle => (1, 2),
            PieceKind::Sniper => (0, 3),
        };

        Rect {
            x: sprite_x as f32 * SPRITE_WIDTH,
            y: sprite_y as f32 * SPRITE_WIDTH,
            w: SPRITE_WIDTH,
            h: SPRITE_WIDTH,
        }
    }

    pub fn greyed_out(color: &Color) -> Color {
        Color::new(
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

    fn render(&self, render_context: &CustomRenderContext) {
        let animation = self.from.interpolate(&self.to, Instant::now());

        let texture = match self.sprite_kind {
            SpriteKind::Piece => render_context.pieces_texture,
            SpriteKind::Special => render_context.special_texture,
        };

        draw_texture_ex(
            texture,
            animation.x_pos,
            animation.y_pos,
            self.override_color.unwrap_or(self.color),
            DrawTextureParams {
                dest_size: Some(Vec2::new(animation.sprite_width, animation.sprite_width)),
                source: Some(self.rect_in_sprite),
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