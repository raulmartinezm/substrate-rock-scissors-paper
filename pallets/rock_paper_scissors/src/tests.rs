use frame_support::assert_noop;

use crate::{
	mock::{self, *},
	Error, GameMovement, GameResult, GameState, Secret, SecretGameMovement,
};

pub const ALICE: <Test as frame_system::Config>::AccountId = 1u64;
pub const BOB: <Test as frame_system::Config>::AccountId = 2u64;
pub const DAVE: <Test as frame_system::Config>::AccountId = 3u64;
pub const A_SECRET: Secret = 1u64;

#[test]
fn it_should_create_a_game() {
	new_test_ext().execute_with(|| {
		let _ = RPS::create_game(Origin::signed(ALICE));
		assert_eq!(RPS::games(1), Some(GameState::default()));
		assert_eq!(last_event(), mock::Event::RPS(crate::Event::GameCreated(1)));
	});
}

#[test]
fn check_movement_rules() {
	let rock = GameMovement::Rock;
	assert_eq!(rock.play(GameMovement::Rock), GameResult::Draw);
	assert_eq!(rock.play(GameMovement::Paper), GameResult::Lose);
	assert_eq!(rock.play(GameMovement::Scissors), GameResult::Win);

	let paper = GameMovement::Paper;
	assert_eq!(paper.play(GameMovement::Rock), GameResult::Win);
	assert_eq!(paper.play(GameMovement::Paper), GameResult::Draw);
	assert_eq!(paper.play(GameMovement::Scissors), GameResult::Lose);

	let scissors = GameMovement::Scissors;
	assert_eq!(scissors.play(GameMovement::Rock), GameResult::Lose);
	assert_eq!(scissors.play(GameMovement::Paper), GameResult::Win);
	assert_eq!(scissors.play(GameMovement::Scissors), GameResult::Draw);
}

#[test]
fn game_movement_can_be_converted_to_bytes() {
	assert!(GameMovement::Rock.to_bytes()[0] == 1_u8);
	assert!(GameMovement::Paper.to_bytes()[0] == 2_u8);
	assert!(GameMovement::Scissors.to_bytes()[0] == 3_u8);
}

#[test]
fn play_game_should_emit_error_when_a_game_is_not_found() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			RPS::play_game(Origin::signed(ALICE), 12, GameMovement::Rock, A_SECRET),
			Error::<Test>::GameNotFound
		);
	});
}

#[test]
fn play_game_should_emit_error_when_a_game_is_full() {
	new_test_ext().execute_with(|| {
		let _ = RPS::create_game(Origin::signed(ALICE));
		let _ = RPS::play_game(Origin::signed(ALICE), 1, GameMovement::Rock, A_SECRET);
		let _ = RPS::play_game(Origin::signed(BOB), 1, GameMovement::Rock, A_SECRET);
		assert_noop!(
			RPS::play_game(Origin::signed(DAVE), 1, GameMovement::Rock, A_SECRET),
			Error::<Test>::GameIsFull
		);
	});
}

#[test]
fn play_game_should_emit_error_when_a_player_tries_to_join_twice() {
	new_test_ext().execute_with(|| {
		let _ = RPS::create_game(Origin::signed(ALICE));
		let _ = RPS::play_game(Origin::signed(ALICE), 1, GameMovement::Rock, A_SECRET);
		assert_noop!(
			RPS::play_game(Origin::signed(ALICE), 1, GameMovement::Rock, A_SECRET),
			Error::<Test>::PlayerAlreadyInGame
		);
	});
}

#[test]
fn play_game_saves_player_movement() {
	new_test_ext().execute_with(|| {
		let _ = RPS::create_game(Origin::signed(ALICE));
		let _ = RPS::play_game(Origin::signed(ALICE), 1, GameMovement::Rock, A_SECRET);
		assert_eq!(last_event(), mock::Event::RPS(crate::Event::PlayerMadeMovement(ALICE)));
		let _ = RPS::play_game(Origin::signed(BOB), 1, GameMovement::Paper, A_SECRET);
		assert_eq!(last_event(), mock::Event::RPS(crate::Event::PlayerMadeMovement(BOB)));
	});
}
