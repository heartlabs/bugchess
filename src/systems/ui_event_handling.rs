use amethyst::{
    core::transform::TransformBundle,
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow, RenderDebugLines},
        types::DefaultBackend,
        RenderingBundle,
        rendy::mesh::{Normal, Position, TexCoord},
    },
    ui::{RenderUi, UiBundle, UiCreator, UiEvent, UiFinder, UiText, UiEventType},
    utils::{
        application_root_dir,
        fps_counter::{FpsCounter, FpsCounterBundle},
        scene::BasicScenePrefab,
    },
    input::{InputBundle, StringBindings},
    ecs::*,
    derive::SystemDesc,
    shrev::{EventChannel, ReaderId},
    assets::{PrefabLoader, PrefabLoaderSystemDesc, Processor, RonFormat},
};
use crate::components::{Board, Activatable};

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
            println!("[SYSTEM] You just interacted with a ui element: {:?}", event);

            if let UiEvent{target, event_type: UiEventType::ClickStart} = event {

                if let Some(activatable) = activatables.get(*target) {
                    println!("[SYSTEM] That element had an activatable: {:?}", activatable.event);
                    board.set_event(activatable.event)
                }
            }
        }
    }
}