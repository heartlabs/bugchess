mod utils;

use game_model::game::Game;
use nanoserde::DeJson;

use game_core::game_controller::GameCommand;

#[test]
fn test_snapshot() -> anyhow::Result<()> {
    let file_content = std::fs::read("tests/snapshots/exported_game.json")?;
    let json = std::str::from_utf8(&file_content).unwrap();
    let (events, game): (Vec<GameCommand>, Game) = DeJson::deserialize_json(json)?;

    let mut test_game = utils::test_utils::create_singleplayer_game();

    events.iter().for_each(|action| {
        let game_clone = (*test_game.game).borrow().clone();
        test_game
            .command_handler
            .handle_new_event(game_clone, action)
    });
    let test_game_game = (*test_game.game).borrow().clone();

    println!("{}", game.board);
    println!("{}", test_game_game.board);

    assert_eq!(game.teams, test_game_game.teams);
    assert_eq!(game.board, test_game_game.board);
    assert_eq!(game, test_game_game);

    //assert_eq!(game, test_game_game);

    Ok(())
}
