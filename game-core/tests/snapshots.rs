use json_comments::StripComments;
use nanoserde::DeJson;
use std::{ffi::OsStr, io::Read, path::PathBuf};

use game_core::{
    board_event_consumer::BoardEventConsumer,
    command_handler::CommandHandler,
    game_controller::GameCommand,
};
use game_events::event_broker::EventBroker;
use game_model::game::{Game, Team};
use std::{cell::RefCell, rc::Rc, sync::{Arc, Mutex}};

fn create_test_game() -> (CommandHandler, Rc<RefCell<Game>>) {
    let mut event_broker = EventBroker::new();
    let game = Rc::new(RefCell::new(Game::new(
        vec![
            Team { id: 0, lost: false, unused_pieces: 0 },
            Team { id: 1, lost: false, unused_pieces: 0 },
        ],
        8,
        8,
    )));
    event_broker.subscribe(Box::new(BoardEventConsumer::new(game.clone())));
    let command_handler = CommandHandler::new(event_broker, Arc::new(Mutex::new(vec![])));
    (command_handler, game)
}

#[test]
fn test_all_snapshots() -> anyhow::Result<()> {
    let mut exported_games: Vec<PathBuf> = std::fs::read_dir("tests/exported_games")?
        .map(|f| f.expect("Could not read file").path())
        .filter(|f| f.file_name().unwrap().to_str().unwrap().ends_with(".json"))
        .collect();

    exported_games.sort();
    for path in exported_games {
        println!("Testing snapshot {:?}", path);

        let snapshot_name: String = path
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or("No Description")
            .to_string();

        let file_content = std::fs::read(path)?;
        let mut json = String::new();
        StripComments::new(&file_content as &[u8]).read_to_string(&mut json)?;

        let events: Vec<GameCommand> = DeJson::deserialize_json(&json)?;

        let (mut command_handler, game) = create_test_game();

        events.iter().for_each(|action| {
            let game_clone = (*game).borrow().clone();
            command_handler.handle_new_command(game_clone, action)
        });
        let game = (*game).borrow().clone();

        insta::assert_snapshot!(snapshot_name, game);
    }

    Ok(())
}
