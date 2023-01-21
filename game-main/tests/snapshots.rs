mod utils;

use std::fs::File;
use std::path::Path;
use nanoserde::DeBin;
use game_events::actions::compound_events::GameAction;
use game_model::game::Game;

#[test]
fn test_snapshot() -> anyhow::Result<()> {
    let file_content = std::fs::read("tests/snapshots/exported_game4.938254117965698.txt")?;
    let (events, game) : (Vec<GameAction>, Game) = DeBin::deserialize_bin(&file_content)?;

    let mut test_game = utils::test_utils::create_singleplayer_game();

    events.iter().for_each(|action| test_game.event_broker.handle_new_event(action));
    let test_game_game = (*test_game.game).borrow().clone();

    println!("{}", game.board);
    println!("{}", test_game_game.board);

    assert_eq!(game.teams, test_game_game.teams);
    assert_eq!(game.board, test_game_game.board);
    assert_eq!(game, test_game_game);

    //assert_eq!(game, test_game_game);


    Ok(())
}