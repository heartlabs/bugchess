use amethyst::{
    prelude::*,
    ecs::RunNow,
    input::{is_close_requested, is_key_down, VirtualKeyCode},
    ui::{UiEvent, UiEventType,},
};
use crate::components::board::BoardEvent;
use crate::systems::actions::actions::HasRunNow;
use crate::resources::board::Board;

pub trait CoreGameState {
    fn handle_board_event(&mut self, event: BoardEvent, board: &mut Board, data: &StateData<'_, GameData<'_, '_>>) -> SimpleTrans;
    fn run_on_start(&self) -> Vec<Box<HasRunNow>>;
}

impl SimpleState for CoreGameState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        data.world.maintain(); // This makes sure that deleted entities are actually deleted

        for action in self.run_on_start() {
            action.get_run_now().run_now(&data.world);
        }
    }

    fn handle_event(
        &mut self,
        _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                    return Trans::Quit
                }
            }
            StateEvent::Ui(UiEvent{target: _, event_type: UiEventType::ClickStart}) => {
            }
            StateEvent::Input(_input) => {
            }
            _ => {}
        }
        Trans::None
    }

    fn update(&mut self, mut data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans  {

        let mut board = data.world.write_resource::<Board>();

        if let Some(event) = board.poll_event() {
            self.handle_board_event(event, &mut board, &data)
        } else {
            Trans::None
        }
    }
}

//
// impl<'a> State<GameData<'static, 'static>,StateEvent> for CoreGameState<RunNow<'static> + 'static + Copy + Clone> {
//
//     fn on_start<'c>(&mut self, data: StateData<'c, GameData<'static, 'static>>) {
//         data.world.maintain(); // This makes sure that deleted entities are actually deleted
//         for mut action in self.on_start {
//             let mut cloned: Box<dyn RunNow<'c> + 'c> = action.clone();
//             cloned.run_now(&data.world);
//         }
//         println!("on_start() called");
//     }
//
//     fn handle_event(
//         &mut self,
//         _data: StateData<'_, GameData<'static, 'static>>,
//         event: StateEvent,
//     ) -> Trans<GameData<'static, 'static>, StateEvent> {
//         println!("handle_event() called");
//         match &event {
//             StateEvent::Window(event) => {
//                 if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
//                     return Trans::Quit
//                 }
//             }
//             StateEvent::Ui(UiEvent{target: _, event_type: UiEventType::ClickStart}) => {
//             }
//             StateEvent::Input(_input) => {
//             }
//             _ => {}
//         }
//         Trans::None
//     }
//
//     fn update(&mut self, mut data: StateData<'_, GameData<'static, 'static>>) -> Trans<GameData<'static, 'static>, StateEvent> {
//         self.delegate.update(&mut data)
//     }
// }