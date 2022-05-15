use crate::{
    constants::*,
    game_logic::{board::*, game::*, piece::*, ranges::*},
    states::core_game_state::CoreGameSubstate,
    rendering::animation::*
};
use egui_macroquad::egui::TextBuffer;
use instant::{Duration, Instant};
use macroquad::prelude::*;
use macroquad_canvas::Canvas2D;
use std::{cell::Cell, collections::HashMap, rc::Rc};
use crate::rendering::ui::Button;

#[derive(Debug, Clone)]
pub struct CustomRenderContext {
    pieces_texture: Texture2D,
    pub game_state: CoreGameSubstate,
    pub button_next: Button,
    pub button_undo: Button,
    start_time: Instant,
}

impl CustomRenderContext {
    pub(crate) fn new() -> Self {
        CustomRenderContext {
            pieces_texture: Texture2D::from_file_with_format(
                include_bytes!("../../resources/sprites/pieces.png"),
                None,
            ),
            game_state: CoreGameSubstate::Place,
            button_next: Button::new(10., "End Turn".to_string()),
            button_undo: Button::new(120., "Undo".to_string()),
            start_time: Instant::now(),
        }
    }

    pub fn elapsed_time(&self) -> Duration {
        self.start_time.elapsed()
    }

/*    pub fn reset_elapsed_time(&mut self) {
        self.start_time = Instant::now();
    }*/
}

pub struct BoardRender {
    pub(crate) unused_pieces: Vec<Vec<PieceRender>>,
    pub(crate) placed_pieces: HashMap<Point2, PieceRender>,
    pub(crate) team_colors: Vec<Color>,
    animations: Vec<Animation>
}

impl BoardRender {
    pub fn new(game: &Game) -> Self {
        let mut unused_pieces = vec![vec![], vec![]];
        let mut placed_pieces = HashMap::new();

        let board = &game.board;

        board.for_each_placed_piece(|point, piece| {
            placed_pieces.insert(
                point,
                PieceRender::from_piece(&point, piece, game.get_team(piece.team_id).color),
            );
        });

        info!("!! New board with {}&{} unused and {} placed pieces", unused_pieces[0].len(), unused_pieces[1].len(), placed_pieces.len());

        BoardRender {
            unused_pieces,
            placed_pieces,
            team_colors: game.teams.iter().map(|t| t.color).collect(),
            animations: vec![]
        }
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

        unused_pieces.push(PieceRender::new(
            x_pos,
            y_pos,
            self.team_colors[team_id],
            PieceKind::Simple,
        ));
    }

    pub fn add_animation(&mut self, mut animation: Animation) {
        if let Some(last) = self.animations.last_mut() {
            info!("Append animation");
            last.next_animations.push(animation);
        } else {
            info!("New Animation");
            animation.start(self);
            self.animations.push(animation);
        }
    }

    pub fn add_placed_piece(&mut self, point: &Point2, piece_kind: PieceKind, team_id: usize){
        let (x_pos, y_pos) = PieceRender::render_pos(point);
        self.placed_pieces.insert(*point, PieceRender::new(x_pos, y_pos, self.team_colors[team_id], piece_kind));
    }

    pub fn update(&mut self) {
        let mut new_animations = vec![];
        self.animations.iter_mut()
            .filter(|a|a.finished_at <= Instant::now())
            .for_each(|a| {
                new_animations.append(&mut a.next_animations);
            });

        let anim_count = self.animations.len();
        self.animations.retain(|a|a.finished_at > Instant::now());

        if anim_count != self.animations.len() || !new_animations.is_empty() {
       //     info!("animation count {} -> {}; {} new", anim_count, self.animations.len(), new_animations.len());
        }

        for animation in new_animations.iter_mut() {
            animation.start(self);
        }

        self.animations.append(&mut new_animations);
    }

    pub fn render(&self, board: &Board, render_context: &CustomRenderContext, canvas: &Canvas2D) {

        Self::render_cells(board, canvas);

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

        board.for_each_cell(|cell| {
            let (x_pos, y_pos) = cell_coords(&cell.point);

            if !cell.effects.is_empty() {
                draw_rectangle(
                    x_pos,
                    y_pos,
                    CELL_ABSOLUTE_WIDTH,
                    CELL_ABSOLUTE_WIDTH,
                    Color::new(80., 0., 100., 0.6),
                );
            }
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
pub struct PieceRender {
    from: AnimationPoint,
    to: AnimationPoint,
    color: Color,
    rect_in_sprite: Rect,
}



impl PieceRender {
    pub fn new(x_pos: f32, y_pos: f32, color: Color, piece_kind: PieceKind) -> Self {
        const PIECE_SCALE: f32 = 60.;

        let pap = AnimationPoint {
            x_pos,
            y_pos,
            scale: PIECE_SCALE,
            instant: Instant::now(),
        };

        Self::animated(pap, pap, color, piece_kind)
    }

    pub(crate) fn from_piece(point: &Point2, piece: &Piece, color: Color) -> PieceRender {
        let (x_pos, y_pos) = Self::render_pos(point);

        PieceRender::new(x_pos, y_pos, color, piece.piece_kind)
    }

    fn render_pos(point: &Point2) -> (f32, f32) {
        let (x_pos, y_pos) = cell_coords(point);
        let piece_scale = 60.;
        let shift = (CELL_ABSOLUTE_WIDTH - piece_scale) / 2.;
        (x_pos + shift, y_pos + shift)
    }

    fn animated(
        from: AnimationPoint,
        to: AnimationPoint,
        color: Color,
        piece_kind: PieceKind,
    ) -> Self {
        let (sprite_x, sprite_y) = match piece_kind {
            PieceKind::Simple => (0, 0),
            PieceKind::HorizontalBar => (1, 0),
            PieceKind::VerticalBar => (0, 1),
            PieceKind::Cross => (1, 1),
            PieceKind::Queen => (0, 2),
            PieceKind::Castle => (1, 2),
            PieceKind::Sniper => (0, 3),
        };

        let rect_in_sprite = Rect {
            x: sprite_x as f32 * SPRITE_WIDTH,
            y: sprite_y as f32 * SPRITE_WIDTH,
            w: SPRITE_WIDTH,
            h: SPRITE_WIDTH,
        };

        PieceRender {
            from,
            to,
            color,
            rect_in_sprite,
        }
    }

    pub fn move_towards(&mut self, point: &Point2) {
        self.from = self.to;
        self.from.instant = Instant::now();

        let (x_pos, y_pos) = Self::render_pos(point);

        self.to = AnimationPoint {
            x_pos,
            y_pos,
            scale: PIECE_SCALE,
            instant: Instant::now() + Duration::from_millis(PLACE_PIECE_SPEED),
        };
    }

    fn render(&self, render_context: &CustomRenderContext) {
        let animation = self
            .from
            .interpolate(self.to, Instant::now());
        draw_texture_ex(
            render_context.pieces_texture,
            animation.x_pos,
            animation.y_pos,
            self.color,
            DrawTextureParams {
                dest_size: Some(Vec2::new(animation.scale, animation.scale)),
                source: Some(self.rect_in_sprite),
                ..Default::default()
            },
        );
    }
}
