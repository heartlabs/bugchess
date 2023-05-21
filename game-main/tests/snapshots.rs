mod utils;

use json_comments::StripComments;
use nanoserde::DeJson;
use std::{ffi::OsStr, io::Read, path::PathBuf};

use game_core::game_controller::GameCommand;
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

        let mut test_game = utils::test_utils::create_singleplayer_game();

        events.iter().for_each(|action| {
            let game_clone = (*test_game.game).borrow().clone();
            test_game
                .command_handler
                .handle_new_command(game_clone, action)
        });
        let game = (*test_game.game).borrow().clone();

        insta::assert_display_snapshot!(snapshot_name, game);
    }

    Ok(())
}
