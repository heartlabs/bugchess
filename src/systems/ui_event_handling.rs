use amethyst::{
    ui::{UiEvent, UiEventType},
    ecs::*,
    derive::SystemDesc,
    shrev::{EventChannel, ReaderId},
};
use crate::components::{Activatable};
use crate::resources::board::Board;

/// This shows how to handle UI events.
#[derive(SystemDesc)]
#[system_desc(name(UiEventHandlerSystemDesc))]
pub struct UiEventHandlerSystem {
    #[system_desc(event_channel_reader)]
    reader_id: ReaderId<UiEvent>,
}

impl UiEventHandlerSystem {
    pub fn new(reader_id: ReaderId<UiEvent>) -> Self {
        Self { reader_id }
    }
}

impl<'a> System<'a> for UiEventHandlerSystem {
    type SystemData = (
        ReadStorage<'a, Activatable>,
        Write<'a, EventChannel<UiEvent>>,
        WriteExpect<'a, Board>,
    );

    fn run(&mut self, (activatables, events, mut board): Self::SystemData) {
        // Reader id was just initialized above if empty
        for event in events.read(&mut self.reader_id) {

            if let UiEvent{target, event_type: UiEventType::ClickStart} = event {

                if let Some(activatable) = activatables.get(*target) {
                    println!("[SYSTEM] That element had an activatable: {:?}", activatable.event);
                    board.set_event(activatable.event)
                }
            }
        }
    }
}