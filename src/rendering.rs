use crate::{
    piece::PieceKind,
    ranges::*,
    states::core_game_state::{CoreGameSubstate},
    *,
};
use instant::{Duration, Instant};

pub struct CustomRenderContext {
    pieces_texture: Texture2D,
    pub game_state: CoreGameSubstate,
    start_time: Instant,
}

impl CustomRenderContext {
    pub(crate) fn new() -> Self {
        CustomRenderContext {
            pieces_texture: Texture2D::from_file_with_format(
                include_bytes!("../resources/sprites/pieces.png"),
                None,
            ),
            game_state: CoreGameSubstate::Place,
            start_time: Instant::now(),
        }
    }

    pub fn elapsed_time(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn reset_elapsed_time(&mut self) {
        self.start_time = Instant::now();
    }
}

pub struct BoardRender {
    unused_pieces: Vec<PieceRender>,
    placed_pieces: Vec<PieceRender>,
}

impl BoardRender {
    pub fn new(board: &Board) -> Self {
        let mut unused_pieces = vec![];
        let mut placed_pieces = vec![];

        {
            let (upb_x, mut upb_y) = cell_coords_tuple(board.w, board.h - 1);
            upb_y += CELL_ABSOLUTE_WIDTH / 4.;

            let first_team = board.get_team(0);
            let color = first_team.color;
            for i in 0..first_team.unused_pieces {
                unused_pieces.push(PieceRender::new(
                    upb_x,
                    upb_y - i as f32 * 32.,
                    color,
                    PieceKind::Simple,
                ));
            }
        }
        {
            let (mut upb_x, upb_y) = cell_coords_tuple(0, 0);
            upb_x -= CELL_ABSOLUTE_WIDTH / 1.25;

            let second_team = board.get_team(1);
            let color = second_team.color;
            for i in 0..second_team.unused_pieces {
                unused_pieces.push(PieceRender::new(
                    upb_x,
                    upb_y + i as f32 * 32.,
                    color,
                    PieceKind::Simple,
                ));
            }
        }

        board.for_each_placed_piece(|point, piece| {
            let (x_pos, y_pos) = cell_coords(&point);
            let piece_scale = 60.;
            let shift = (CELL_ABSOLUTE_WIDTH - piece_scale) / 2.;

            placed_pieces.push(PieceRender::new(
                x_pos + shift,
                y_pos + shift,
                board.get_team(piece.team_id).color,
                piece.piece_kind,
            ));
        });

        BoardRender {
            unused_pieces,
            placed_pieces,
        }
    }

    pub fn render(&self, board: &Board, render_context: &CustomRenderContext) {
        board.for_each_cell(|cell| {
            let (x_pos, y_pos) = cell_coords(&cell.point);

            let mouse_point = cell_hovered();

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

        let mut selected_point_option = Option::None;
        let hovered_point = cell_hovered();
        if let Some(hovered_piece) = board.get_piece_at(&hovered_point) {
            let range_context = match render_context.game_state {
                CoreGameSubstate::Place => RangeContext::Moving(*hovered_piece),
                CoreGameSubstate::Move(selected_point) => { selected_point_option = Option::Some(selected_point); RangeContext::Moving(*hovered_piece)}
                CoreGameSubstate::Activate(selected_point) => {selected_point_option = Option::Some(selected_point); RangeContext::Special(*hovered_piece)}
                CoreGameSubstate::Won(_) => RangeContext::Moving(*hovered_piece),
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
                };
                let range_option: Option<Range> = match render_context.game_state {
                    CoreGameSubstate::Place => Option::None,
                    CoreGameSubstate::Move(_) => selected_piece.movement.map(|m| m.range),
                    CoreGameSubstate::Activate(_) => selected_piece.activatable.map(|m| m.range),
                    CoreGameSubstate::Won(_) => selected_piece.movement.map(|m| m.range),
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

        //println!("rendered {:?}", self.unused_pieces.len());
        self.unused_pieces.iter().for_each(|p| {
            p.render(render_context);
        });
        self.placed_pieces
            .iter()
            .for_each(|p| p.render(render_context));
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
    from: PieceAnimationPoint,
    to: PieceAnimationPoint,
    color: Color,
    rect_in_sprite: Rect,
}

#[derive(Clone, Copy, Debug)]
pub struct PieceAnimationPoint {
    x_pos: f32,
    y_pos: f32,
    scale: f32,
    elapsed_time_ms: u32,
}

impl PieceRender {
    fn new(x_pos: f32, y_pos: f32, color: Color, piece_kind: PieceKind) -> Self {
        const PIECE_SCALE: f32 = 60.;

        let pap = PieceAnimationPoint {
            x_pos,
            y_pos,
            scale: PIECE_SCALE,
            elapsed_time_ms: 0,
        };

        Self::animated(pap, pap, color, piece_kind)
    }

    fn animated(
        from: PieceAnimationPoint,
        to: PieceAnimationPoint,
        color: Color,
        piece_kind: PieceKind,
    ) -> Self {
        const PIECE_SCALE: f32 = 60.;
        const SPRITE_WIDTH: f32 = 64.;

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

    fn render(&self, render_context: &CustomRenderContext) {
        let animation = self
            .from
            .interpolate(self.to, render_context.elapsed_time());
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

impl PieceAnimationPoint {
    pub fn interpolate(
        &self,
        towards: PieceAnimationPoint,
        elapsed_time: Duration,
    ) -> PieceAnimationPoint {
        if true {
            return towards;
        }

        let elapsed_time_ms = elapsed_time.as_millis() as u32;
        let progress = if elapsed_time_ms < self.elapsed_time_ms {
            0.
        } else if elapsed_time_ms > towards.elapsed_time_ms {
            1.
        } else {
            let diff = towards.elapsed_time_ms - self.elapsed_time_ms;
            let local_elapsed = elapsed_time_ms - self.elapsed_time_ms;

            //println!("interpolating after {}ms diff is {} and local_elapsed {} leading to {}", elapsed_time_ms, diff, local_elapsed, local_elapsed as f32 / diff as f32);

            local_elapsed as f32 / diff as f32
        };

        PieceAnimationPoint {
            x_pos: Self::interpolate_value(self.x_pos, towards.x_pos, progress),
            y_pos: Self::interpolate_value(self.y_pos, towards.y_pos, progress),
            scale: Self::interpolate_value(self.scale, towards.scale, progress),
            elapsed_time_ms,
        }
    }

    fn interpolate_value(from: f32, to: f32, progress: f32) -> f32 {
        let diff = to - from;
        let p = diff * progress;
        from + p
    }
}
