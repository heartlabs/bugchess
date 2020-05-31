use amethyst::{
    input::{InputHandler, StringBindings},
    core::SystemDesc,
    derive::SystemDesc,
    ecs::{Read, ReadStorage, ReadExpect, Write, WriteStorage, WriteExpect, System, SystemData, Join, Entity},
    winit::MouseButton,
    core::Transform,
    window::ScreenDimensions,
    renderer::{
        palette::Srgba,
        debug_drawing::{DebugLines},
    },
};

use amethyst::core::math::{Point3, Point2};
use ncollide3d::query::PointQuery;

use crate::components::{Activatable, Bounded, Mouse, Board};


#[derive(SystemDesc)]
pub struct MouseHandler {
    mouse_button_was_already_handled: bool,
}


impl MouseHandler {
    fn mouse_pos(input: &InputHandler<StringBindings>, screen: &ScreenDimensions) -> Option<(f32, f32)> {
        let hidpi : f32 = screen.hidpi_factor() as f32;
        input.mouse_position().map(|(x, y)| {
            (x / screen.width() * 800. * hidpi,
             (600. - y / screen.height() * 600.) * hidpi)
        })
    }

    pub fn new() -> MouseHandler {
        MouseHandler{mouse_button_was_already_handled: false}
    }
}

impl<'s> System<'s> for MouseHandler {
    type SystemData = (
        Read<'s, InputHandler<StringBindings>>,
        ReadExpect<'s, ScreenDimensions>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Bounded>,
        ReadStorage<'s, Mouse>,
        WriteStorage<'s, Activatable>,
        Write<'s, DebugLines>,
        WriteExpect<'s, Board>,
    );

    fn run(&mut self, (input, screen, mut transforms, boundings, mouses, mut activatables, mut debug_lines_resource, mut board): Self::SystemData) {

        if let Some((x, y)) = MouseHandler::mouse_pos(&*input, &*screen) {
            if input.mouse_button_is_down(MouseButton::Left) {
                if !self.mouse_button_was_already_handled{
                    self.mouse_button_was_already_handled = true;
                    for (activatable, transform, bounded) in (&mut activatables, &mut transforms, &boundings).join() {
                        let inside = bounded.bounds.contains_point(transform.isometry(), &Point3::new(x, y, 0.0));

                        if inside {
                            board.set_event(activatable.event);
                        }

                        activatable.active = inside;
                        //println!("DIMENSIONS: {:?}",*screen);
                        //println!("transform {:?}", transform);
                        //println!("isometry {}", transform.isometry());
                        //println!("Activated at {} {} {}", x, y, inside);

                    }
                }
            } else {
                self.mouse_button_was_already_handled = false;
            }

            for (_mouse, transform) in (&mouses, &mut transforms).join() {
                transform.set_translation_x(x);
                transform.set_translation_y(y);
            }
        }

        for (transform, bounded) in (&transforms, &boundings).join() {
            let rect_x = transform.translation().x;
            let rect_y = transform.translation().y;

            let half_height = bounded.bounds.half_extents().x;
            let half_width = bounded.bounds.half_extents().y;
            debug_lines_resource.draw_rectangle(
                Point2::new(rect_x - half_height, rect_y - half_width),
                Point2::new(rect_x + half_height, rect_y + half_width),
                0.0,
                Srgba::new(0.5, 0.9, 0.0, 1.0),
            );
        }


    }
}
