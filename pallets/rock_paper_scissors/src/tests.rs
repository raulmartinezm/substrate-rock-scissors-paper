use frame_support::assert_noop;

use crate::{mock::*, Error, GameId, GameMovement, GameResult, GameState, Secret};

pub type MockAccountId = <Test as frame_system::Config>::AccountId;

pub const ALICE: MockAccountId = 1u64;
pub const BOB: MockAccountId = 2u64;
pub const DAVE: MockAccountId = 3u64;

pub const A_SECRET: Secret = 1u64;
pub const ANOTHER_SECRET: Secret = 2u64;
pub const GAME_ID: GameId = 1;

const BET_AMOUNT: u64 = 1_000;

const ENDOWED_ACCOUNTS: [MockAccountId; 3] = [ALICE, BOB, DAVE];
const ENDOWMENT_AMOUNT: u64 = 1_000_000_u64;

mod game_logic {
	use crate::{GameMovement, GameResult};

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
}

pub mod create_game {
	use crate::{
		mock::{self, *},
		tests::*,
	};
	#[test]
	fn should_create_a_game() {
		new_test_ext(&ENDOWED_ACCOUNTS, ENDOWMENT_AMOUNT).execute_with(|| {
			let _ = RPS::create_game(Origin::signed(ALICE));
			assert_eq!(RPS::games(1), Some(GameState::default()));
			assert_eq!(last_event(), mock::Event::RPS(crate::Event::GameCreated(1)));
		});
	}
}

pub mod play_game {
	use crate::{
		mock::{self, *},
		tests::*,
	};

	#[test]
	fn should_emit_error_when_a_game_is_not_found() {
		new_test_ext(&[], ENDOWMENT_AMOUNT).execute_with(|| {
			assert_noop!(
				RPS::play_game(Origin::signed(ALICE), 12, GameMovement::Rock, A_SECRET, BET_AMOUNT),
				Error::<Test>::GameNotFound
			);
		});
	}

	#[test]
	fn should_emit_error_when_a_game_is_full() {
		new_test_ext(&ENDOWED_ACCOUNTS, ENDOWMENT_AMOUNT).execute_with(|| {
			let _ = RPS::create_game(Origin::signed(ALICE));
			let _ = RPS::play_game(
				Origin::signed(ALICE),
				GAME_ID,
				GameMovement::Rock,
				A_SECRET,
				BET_AMOUNT,
			);
			let _ = RPS::play_game(
				Origin::signed(BOB),
				GAME_ID,
				GameMovement::Rock,
				A_SECRET,
				BET_AMOUNT,
			);
			assert_noop!(
				RPS::play_game(
					Origin::signed(DAVE),
					GAME_ID,
					GameMovement::Rock,
					A_SECRET,
					BET_AMOUNT
				),
				Error::<Test>::GameIsFull
			);
		});
	}

	#[test]
	fn should_emit_error_when_a_player_has_not_enough_balance() {
		new_test_ext(&[ALICE], ENDOWMENT_AMOUNT).execute_with(|| {
			let _ = RPS::create_game(Origin::signed(ALICE));
			let _ = RPS::play_game(
				Origin::signed(ALICE),
				GAME_ID,
				GameMovement::Rock,
				A_SECRET,
				BET_AMOUNT,
			);
			assert_eq!(
				last_event(),
				mock::Event::RPS(crate::Event::PlayerMadeMovement(ALICE)),
				"Alice should have enough balance and be allowed to play"
			);

			let _ = RPS::play_game(
				Origin::signed(BOB),
				GAME_ID,
				GameMovement::Rock,
				A_SECRET,
				BET_AMOUNT,
			);
			assert_noop!(
				RPS::play_game(
					Origin::signed(DAVE),
					GAME_ID,
					GameMovement::Rock,
					A_SECRET,
					BET_AMOUNT
				),
				Error::<Test>::InsufficientBalance
			);
		});
	}

	#[test]
	fn should_emit_error_when_a_player_tries_to_join_twice() {
		new_test_ext(&ENDOWED_ACCOUNTS, ENDOWMENT_AMOUNT).execute_with(|| {
			let _ = RPS::create_game(Origin::signed(ALICE));
			let _ = RPS::play_game(
				Origin::signed(ALICE),
				GAME_ID,
				GameMovement::Rock,
				A_SECRET,
				BET_AMOUNT,
			);
			assert_noop!(
				RPS::play_game(
					Origin::signed(ALICE),
					GAME_ID,
					GameMovement::Rock,
					A_SECRET,
					BET_AMOUNT
				),
				Error::<Test>::PlayerAlreadyInGame
			);
		});
	}

	#[test]
	fn saves_player_movement() {
		new_test_ext(&ENDOWED_ACCOUNTS, ENDOWMENT_AMOUNT).execute_with(|| {
			let _ = RPS::create_game(Origin::signed(ALICE));
			let _ = RPS::play_game(
				Origin::signed(ALICE),
				GAME_ID,
				GameMovement::Rock,
				A_SECRET,
				BET_AMOUNT,
			);
			assert_eq!(last_event(), mock::Event::RPS(crate::Event::PlayerMadeMovement(ALICE)));
			let _ = RPS::play_game(
				Origin::signed(BOB),
				GAME_ID,
				GameMovement::Paper,
				A_SECRET,
				BET_AMOUNT,
			);
			assert_eq!(last_event(), mock::Event::RPS(crate::Event::PlayerMadeMovement(BOB)));
		});
	}
}

pub mod reveal_winner {

	use crate::{
		mock::{self, *},
		tests::*,
	};

	#[test]
	fn should_emit_event_when_game_not_found() {
		const NOT_EXISING_GAME_ID: GameId = 1;
		new_test_ext(&ENDOWED_ACCOUNTS, ENDOWMENT_AMOUNT).execute_with(|| {
			assert_noop!(
				RPS::reveal_winner(
					Origin::signed(ALICE),
					NOT_EXISING_GAME_ID,
					GameMovement::Rock,
					A_SECRET,
					BOB,
					GameMovement::Paper,
					A_SECRET
				),
				Error::<Test>::GameNotFound
			);
		});
	}

	#[test]
	fn should_emit_error_when_player_not_in_a_game() {
		const NOT_EXISING_GAME_ID: GameId = 1;

		new_test_ext(&ENDOWED_ACCOUNTS, ENDOWMENT_AMOUNT).execute_with(|| {
			let _ = RPS::create_game(Origin::signed(ALICE));
			let _ = RPS::play_game(
				Origin::signed(ALICE),
				GAME_ID,
				GameMovement::Rock,
				A_SECRET,
				BET_AMOUNT,
			);
			let _ = RPS::play_game(
				Origin::signed(BOB),
				GAME_ID,
				GameMovement::Rock,
				A_SECRET,
				BET_AMOUNT,
			);

			assert_noop!(
				RPS::reveal_winner(
					Origin::signed(DAVE),
					NOT_EXISING_GAME_ID,
					GameMovement::Rock,
					A_SECRET,
					BOB,
					GameMovement::Paper,
					A_SECRET
				),
				Error::<Test>::PlayerNotInGame
			);
		});
	}

	#[test]
	fn should_emit_error_when_hash_does_not_match() {
		const NOT_EXISING_GAME_ID: GameId = 1;

		new_test_ext(&ENDOWED_ACCOUNTS, ENDOWMENT_AMOUNT).execute_with(|| {
			let _ = RPS::create_game(Origin::signed(ALICE));
			let _ = RPS::play_game(
				Origin::signed(ALICE),
				GAME_ID,
				GameMovement::Rock,
				A_SECRET,
				BET_AMOUNT,
			);
			let _ = RPS::play_game(
				Origin::signed(BOB),
				GAME_ID,
				GameMovement::Paper,
				ANOTHER_SECRET,
				BET_AMOUNT,
			);

			assert_noop!(
				RPS::reveal_winner(
					Origin::signed(ALICE),
					NOT_EXISING_GAME_ID,
					GameMovement::Rock,
					A_SECRET,
					BOB,
					GameMovement::Paper,
					A_SECRET
				),
				Error::<Test>::InvalidHash
			);

			assert_noop!(
				RPS::reveal_winner(
					Origin::signed(ALICE),
					NOT_EXISING_GAME_ID,
					GameMovement::Rock,
					ANOTHER_SECRET,
					BOB,
					GameMovement::Paper,
					ANOTHER_SECRET
				),
				Error::<Test>::InvalidHash
			);
		});
	}

	#[test]
	fn should_emit_event_when_game_finished() {
		new_test_ext(&ENDOWED_ACCOUNTS, ENDOWMENT_AMOUNT).execute_with(|| {
			let _ = RPS::create_game(Origin::signed(ALICE));
			let _ = RPS::play_game(
				Origin::signed(ALICE),
				GAME_ID,
				GameMovement::Rock,
				A_SECRET,
				BET_AMOUNT,
			);
			let _ = RPS::play_game(
				Origin::signed(BOB),
				GAME_ID,
				GameMovement::Paper,
				ANOTHER_SECRET,
				BET_AMOUNT,
			);
			let _ = RPS::reveal_winner(
				Origin::signed(ALICE),
				GAME_ID,
				GameMovement::Rock,
				A_SECRET,
				BOB,
				GameMovement::Paper,
				ANOTHER_SECRET,
			);
			assert_eq!(
				last_event(),
				mock::Event::RPS(crate::Event::GameFinished(GAME_ID, GameResult::Lose, Some(BOB)))
			);
		});
	}

	#[test]
	fn should_emit_same_event_when_game_is_already_finished() {
		new_test_ext(&ENDOWED_ACCOUNTS, ENDOWMENT_AMOUNT).execute_with(|| {
			let _ = RPS::create_game(Origin::signed(ALICE));
			let _ = RPS::play_game(
				Origin::signed(ALICE),
				GAME_ID,
				GameMovement::Rock,
				A_SECRET,
				BET_AMOUNT,
			);
			let _ = RPS::play_game(
				Origin::signed(BOB),
				GAME_ID,
				GameMovement::Paper,
				ANOTHER_SECRET,
				BET_AMOUNT,
			);
			let _ = RPS::reveal_winner(
				Origin::signed(ALICE),
				GAME_ID,
				GameMovement::Rock,
				A_SECRET,
				BOB,
				GameMovement::Paper,
				ANOTHER_SECRET,
			);
			assert_eq!(
				last_event(),
				mock::Event::RPS(crate::Event::GameFinished(GAME_ID, GameResult::Lose, Some(BOB)))
			);
			let _ = RPS::reveal_winner(
				Origin::signed(ALICE),
				GAME_ID,
				GameMovement::Rock,
				A_SECRET,
				BOB,
				GameMovement::Paper,
				ANOTHER_SECRET,
			);
			assert_eq!(
				last_event(),
				mock::Event::RPS(crate::Event::GameFinished(GAME_ID, GameResult::Lose, Some(BOB)))
			);
		});
	}
}
