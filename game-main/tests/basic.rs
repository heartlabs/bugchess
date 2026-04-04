use std::{cell::RefCell, rc::Rc};

use game_core::{core_game::CoreGameSubstate, multiplayer_connector::MultiplayerConector};
use game_model::{
    game::Game,
    piece::{EffectKind::Protection, PieceKind},
};
mod utils;
use utils::{fakebox::FakeboxClient, test_utils::*};

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

#[test]
fn test_accept_reconnection_updates_opponent_id() {
    let (client1, _client2) = FakeboxClient::new_client_pair();

    let mut connector = MultiplayerConector::new(Box::new(client1.clone()));
    connector.matchmaking(); // detects client2, sets opponent_id = "2"
    assert_eq!(connector.opponent_id, Some("2".to_string()));

    // Simulate client2 disconnecting and a new peer ("3") reconnecting
    client1.borrow_mut().disconnect();
    let mut client3 = FakeboxClient::new("3");
    client3.connect(client1.clone());
    let client3 = Rc::new(RefCell::new(client3));
    client1.borrow_mut().connect(client3.clone());

    // accept_reconnection detects the new peer and updates opponent_id
    let reconnected = connector.accept_connection();
    assert!(reconnected, "expected new peer to be detected");
    assert_eq!(connector.opponent_id, Some("3".to_string()));
}

#[test]
fn test_reconnect_resends_game_history() {
    // Set up initial 2-player game and play a move
    let (multiplayer_client1, multiplayer_client2) = FakeboxClient::new_client_pair();

    let mut game1 = create_singleplayer_game();
    make_multiplayer(multiplayer_client1.clone(), &mut game1);

    let mut game2 = create_singleplayer_game();
    make_multiplayer(multiplayer_client2.clone(), &mut game2);

    game1.add_unused_pieces(3, 3);
    game2.add_unused_pieces(3, 3);

    game1.click_at_pos((0, 0));
    game2.recieve_multiplayer_events();

    game1.assert_piece_at((0, 0), PieceKind::Simple);
    game2.assert_piece_at((0, 0), PieceKind::Simple);

    // Simulate game2 closing the tab: create a fresh session wired to game1's client
    multiplayer_client1.borrow_mut().disconnect();
    let mut game3 = create_singleplayer_game();
    let mut client3 = FakeboxClient::new("3");
    client3.connect(multiplayer_client1.clone());
    let client3 = Rc::new(RefCell::new(client3));
    multiplayer_client1.borrow_mut().connect(client3.clone());
    make_multiplayer(client3, &mut game3);
    game3.add_unused_pieces(3, 3);
    game3.signal_connect();

    // Game1 polls: detects the new peer via accept_connection, receives Connect,
    // then responds with NewGame + all past game commands (resend_game_events)
    game1.recieve_multiplayer_events();

    // Reconnected game receives and applies the full game history
    game3.recieve_multiplayer_events();

    // Board state must match what was there before disconnection
    game3.assert_piece_at((0, 0), PieceKind::Simple);
}
