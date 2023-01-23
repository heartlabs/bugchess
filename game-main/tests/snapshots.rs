mod utils;

use game_model::game::Game;
use nanoserde::DeJson;

use game_core::game_controller::GameCommand;
#[test]
fn test_snapshot() -> anyhow::Result<()> {
    let file_content = std::fs::read("tests/snapshots/exported_game.json")?;
    let json = std::str::from_utf8(&file_content).unwrap();
    let (events, _): (Vec<GameCommand>, Game) = DeJson::deserialize_json(json)?;

    let mut test_game = utils::test_utils::create_singleplayer_game();

    events.iter().for_each(|action| {
        let game_clone = (*test_game.game).borrow().clone();
        test_game
            .command_handler
            .handle_new_command(game_clone, action)
    });
    let game = (*test_game.game).borrow().clone();

    insta::assert_display_snapshot!(game);

    Ok(())
}
