#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
pub use pallet::*;

use std::convert::Into;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub type GameId = u64;
pub type Secret = u64;

#[derive(Clone, Copy, Encode, Debug, Decode, Eq, TypeInfo, MaxEncodedLen, PartialEq)]
pub struct SecretGameMovement([u8; 8]);

impl SecretGameMovement {
	pub fn new(movement: GameMovement, secret: Secret) -> Self {
		let mut bytes = sp_std::vec::Vec::new();
		bytes.extend(movement.to_bytes());
		bytes.extend(secret.to_ne_bytes());
		SecretGameMovement(sp_io::hashing::twox_64(&bytes))
	}

	/// Returns true if the secret movement correspond to
	pub fn is_equal(&self, movement: GameMovement, secret: Secret) -> bool {
		SecretGameMovement::new(movement, secret) == *self
	}
}

#[derive(Clone, Copy, Encode, Debug, Decode, Eq, TypeInfo, MaxEncodedLen, PartialEq)]
pub struct PlayerMovement<AccountId> {
	player: AccountId,
	movement: SecretGameMovement,
}

#[derive(Clone, Copy, Encode, Debug, Decode, Eq, TypeInfo, MaxEncodedLen, PartialEq)]
pub struct GameState<AccountId: PartialEq + Clone> {
	pub player1: Option<PlayerMovement<AccountId>>,
	pub player2: Option<PlayerMovement<AccountId>>,
	pub game_result: GameResult,
	pub winner: Option<AccountId>,
}

impl<AccountId: PartialEq + Clone> Default for GameState<AccountId> {
	fn default() -> Self {
		Self { player1: None, player2: None, game_result: GameResult::NotPlayed, winner: None }
	}
}

impl<AccountId: PartialEq + Clone> GameState<AccountId> {
	pub fn has_player(&self, player: AccountId) -> bool {
		let mut found = false;
		if let Some(val) = &self.player1 {
				if val.player == player {
					found = true;
				}
			}
		if let Some(val) = &self.player2 {
			if val.player == player {
				found = true;
			}
		}
		return found;
	}

	/// Tells if there are free slots in a game
	pub fn has_free_slots(&self) -> bool {
		self.player1.is_none() || self.player2.is_none()
	}

	/// Add a player
	pub fn add_player(
		&mut self,
		player: AccountId,
		movement: GameMovement,
		secret: Secret,
	) -> bool {
		let player_movement =
			Some(PlayerMovement { player, movement: SecretGameMovement::new(movement, secret) });
		if self.player1.is_none() {
			self.player1 = player_movement;
		} else if self.player2.is_none() {
			self.player2 = player_movement;
		} else {
			return false;
		}
		true
	}
}

#[derive(Eq, PartialEq, Clone, Encode, Debug, Decode, TypeInfo, MaxEncodedLen)]
pub enum GameMovement {
	Rock,
	Paper,
	Scissors,
}

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, PartialEq, TypeInfo, MaxEncodedLen)]
pub enum GameResult {
	NotPlayed,
	Win,
	Lose,
	Draw,
}

impl GameMovement {
	/// Tells if wins, loses or draws for a given movement.
	/// A Rock should draw for another rock, win for scissor and Lose for paper.
	pub fn play(&self, other: GameMovement) -> GameResult {
		match self {
			GameMovement::Rock => match other {
				GameMovement::Rock => GameResult::Draw,
				GameMovement::Paper => GameResult::Lose,
				GameMovement::Scissors => GameResult::Win,
			},
			GameMovement::Paper => match other {
				GameMovement::Rock => GameResult::Win,
				GameMovement::Paper => GameResult::Draw,
				GameMovement::Scissors => GameResult::Lose,
			},
			GameMovement::Scissors => match other {
				GameMovement::Rock => GameResult::Lose,
				GameMovement::Paper => GameResult::Win,
				GameMovement::Scissors => GameResult::Draw,
			},
		}
	}
	fn to_bytes(&self) -> [u8; 1] {
		match self {
			GameMovement::Rock => 1_u8.to_ne_bytes(),
			GameMovement::Paper => 2_u8.to_ne_bytes(),
			GameMovement::Scissors => 3_u8.to_ne_bytes(),
		}
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn next_challenge_id)]
	pub type NextGameId<T> = StorageValue<_, GameId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn games)]
	pub type Games<T: Config> = StorageMap<_, Blake2_128Concat, GameId, GameState<T::AccountId>>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		GameCreated(GameId),
		PlayerMadeMovement(T::AccountId),
		GameFinished(GameId, GameResult, Option<T::AccountId>)
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Game with given id not found
		GameNotFound,
		/// Tried to join a game which already has all players
		GameIsFull,
		/// The player is already in the game
		PlayerAlreadyInGame,
		/// Player not found in a game
		PlayerNotInGame,
		/// The secret movement (hash) doesn't match with the secret and movement
		InvalidHash,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_game(origin: OriginFor<T>) -> DispatchResult {
			ensure_signed(origin)?;

			<NextGameId<T>>::mutate(|x| *x += 1);
			let game_id = NextGameId::<T>::get();
			Games::<T>::insert(game_id, GameState::default());

			Self::deposit_event(Event::GameCreated(game_id));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn play_game(
			origin: OriginFor<T>,
			game_id: GameId,
			movement: GameMovement,
			secret: Secret,
		) -> DispatchResult {
			let account_id = ensure_signed(origin)?;

			let mut game_state = Games::<T>::get(game_id).ok_or(Error::<T>::GameNotFound)?;

			ensure!(game_state.has_free_slots(), Error::<T>::GameIsFull);
			ensure!(!game_state.has_player(account_id.clone()), Error::<T>::PlayerAlreadyInGame);

			game_state.add_player(account_id.clone(), movement, secret);
			Games::<T>::insert(game_id, game_state);

			Self::deposit_event(Event::PlayerMadeMovement(account_id));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn reveal_winner(
			origin: OriginFor<T>,
			game_id: GameId,
			player1_movement: GameMovement,
			player1_secret: Secret,
			player2: T::AccountId,
			player2_movement: GameMovement,
			player2_secret: Secret,
		) -> DispatchResult {
			let player1 = ensure_signed(origin)?;

			let mut game_state = Games::<T>::get(game_id).ok_or(Error::<T>::GameNotFound)?;
			ensure!(game_state.has_player(player1.clone()), Error::<T>::PlayerNotInGame);
			ensure!(game_state.has_player(player2.clone()), Error::<T>::PlayerNotInGame);

			let p1 = game_state.player1.as_ref().unwrap();
			let p2 = game_state.player2.as_ref().unwrap();

			ensure!(p1.movement.is_equal(player1_movement.clone(), player1_secret), Error::<T>::InvalidHash);
			ensure!(p2.movement.is_equal(player2_movement.clone(), player2_secret), Error::<T>::InvalidHash);

			if game_state.game_result != GameResult::NotPlayed {
				Self::deposit_event(Event::GameFinished(game_id, game_state.game_result, game_state.winner));
				return Ok(());
			}

			let game_result = player1_movement.play(player2_movement);

			match game_result {
				GameResult::Win => {
					// Player1 wins
					game_state.winner = Some(player1.clone());
				},
				GameResult::Lose => {
					// Player2 wins
					game_state.winner = Some(player2.clone());
				}
				GameResult::Draw => {
					// Nobody wins or loses
					game_state.game_result = GameResult::Draw;
				},
				_ => {}
			}
			let winner = game_state.winner.clone();
			game_state.game_result = game_result.clone();
			Games::<T>::insert(game_id, game_state);
			Self::deposit_event(Event::GameFinished(game_id, game_result, winner));

			Ok(())
		}
	}
}
