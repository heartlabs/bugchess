use game_core::core_game::CoreGameSubstate;
use game_model::{
    game::Game,
    piece::{EffectKind::Protection, PieceKind},
};
mod utils;
use utils::test_utils::*;

#[test]
fn test_merge_piece_multiplayer() {
    let (mut test_game1, mut test_game2) = create_multiplayer_game();

    // Make Move
    test_game1.click_at_pos((1, 1));
    test_game1.click_at_pos((1, 2));
    test_game1.click_at_pos((1, 3));

    // Recieve move in Game 2
    test_game2.recieve_multiplayer_events();

    // Assertions Game 1
    test_game1.assert_has_game_state(CoreGameSubstate::Place);
    test_game1.assert_num_pieces(1, 0);
    test_game1.assert_piece_at((1, 2), PieceKind::VerticalBar);

    // Assertions Game 2
    test_game2.assert_has_game_state(CoreGameSubstate::Place);
    test_game2.assert_num_pieces(1, 0);
    test_game2.assert_piece_at((1, 2), PieceKind::VerticalBar);
}

#[test]
fn test_place_single_piece_multiplayer() {
    let (mut test_game1, mut test_game2) = create_multiplayer_game();

    // Make Move
    test_game1.click_at_pos((0, 0));

    // Recieve move in Game 2
    test_game2.recieve_multiplayer_events();

    // Assertions Game 1
    test_game1.assert_has_game_state(CoreGameSubstate::Place);
    test_game1.assert_num_pieces(1, 0);
    test_game1.assert_piece_at((0, 0), PieceKind::Simple);

    // Assertions Game 2
    test_game2.assert_has_game_state(CoreGameSubstate::Place);
    test_game2.assert_num_pieces(1, 0);
    test_game2.assert_piece_at((0, 0), PieceKind::Simple);
}

#[test]
fn test_remove_effects() {
    let (mut test_game1, mut test_game2) = create_multiplayer_game();

    // skip a turn each to have enough unused pieces
    test_game1.next_turn();
    test_game2.recieve_multiplayer_events();
    test_game2.next_turn();
    test_game1.recieve_multiplayer_events();

    // Make Move
    test_game1.click_at_pos((1, 0));
    test_game1.click_at_pos((0, 1));
    test_game1.click_at_pos((2, 1));
    test_game1.click_at_pos((1, 2));

    test_game1.next_turn();

    // Recieve move in Game 2
    test_game2.recieve_multiplayer_events();

    // Assertions Game 1
    test_game1.assert_has_game_state(CoreGameSubstate::Place);

    {
        let game = &(*test_game1.game.borrow());
        println!("Board:\n{}", game.board);
        assert_eq!(game.board.placed_pieces(0).len(), 1);
        assert!(game.board.placed_pieces(1).is_empty());

        test_game1.assert_piece_at((1, 1), PieceKind::Castle);
        assert_protection_at(game, (0, 0));
        assert_protection_at(game, (0, 1));
        assert_protection_at(game, (0, 2));
        assert_protection_at(game, (1, 0));
        assert_protection_at(game, (1, 1));
        assert_protection_at(game, (1, 2));
        assert_protection_at(game, (2, 0));
        assert_protection_at(game, (2, 1));
        assert_protection_at(game, (2, 2));
    }

    // Prepare attack of castle
    test_game2.click_at_pos((4, 0));
    test_game2.click_at_pos((4, 1));
    test_game2.click_at_pos((3, 1));
    test_game2.click_at_pos((5, 1));
    test_game2.click_at_pos((4, 2));

    test_game2.next_turn();
    test_game1.recieve_multiplayer_events();
    test_game2.next_turn();

    // attack castle
    test_game2.click_at_pos((4, 1));
    test_game2.click_at_pos((1, 1));

    let game = &(*test_game2.game.borrow());
    assert_eq!(game.board.placed_pieces(1).len(), 1);
    assert!(game.board.placed_pieces(0).is_empty());

    game.board.for_each_cell(|c| assert!(c.effects.is_empty()));
}

fn assert_protection_at(game: &Game, pos: (u8, u8)) {
    assert!(game.board.has_effect_at(&Protection, &pos.into()));
}
