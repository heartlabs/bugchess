// use amethyst::{
//     prelude::*,
//     ecs::RunNow,
//     input::{is_close_requested, is_key_down, VirtualKeyCode},
//     ui::{UiEvent, UiEventType,},
// };
//
// pub struct CoreGameState<T> where T: RunNow<'static> + 'static + Copy + Clone {
//     pub(crate) on_start: Vec<Box<T>>,
//     pub(crate) delegate: Box<SimpleState>,
// }
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