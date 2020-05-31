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


mod states;
mod components;
mod systems;

type MyPrefabData = BasicScenePrefab<(Vec<Position>, Vec<Normal>, Vec<TexCoord>)>;


fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let resources = app_root.join("resources");
    let display_config = resources.join("display_config.ron");
    let input_bundle = InputBundle::<StringBindings>::new();

    let game_data = GameDataBuilder::default()
        .with_system_desc(PrefabLoaderSystemDesc::<MyPrefabData>::default(), "", &[])
        .with_bundle(input_bundle)?
        .with(crate::systems::active::ShowAsActive, "show_as_active", &[])
        .with(crate::systems::mousehandler::MouseHandler::new(), "mouse_handler", &["input_system"])
        .with(crate::systems::board::BoardSystem, "board_system", &[])
        .with_bundle(TransformBundle::new())?
        .with_bundle(UiBundle::<StringBindings>::new())?
        .with_system_desc(crate::systems::ui_event_handling::UiEventHandlerSystemDesc::default(), "ui_event_handler", &[])
        .with_bundle(FpsCounterBundle::default())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderDebugLines::default())
                .with_plugin(RenderUi::default())
                .with_plugin(
                    RenderToWindow::from_config_path(display_config)?
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                ),
        )?;

    let mut game = Application::new(resources, states::LoadingState {}, game_data)?;
    game.run();

    Ok(())
}

