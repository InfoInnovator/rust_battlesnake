#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use game::game_types::{Battlesnake, Board, Game, Move};
use serde_json::Value;
use snakes::{battlerat::BattleratFactory, carlo_constrictor::CarloConstrictorFactory};

pub mod game;
pub mod snakes;

/// A trait to be implemented by all Battlesnake factories.
/// This trait is used to create a new Battlesnake instance and provide
/// information about the snake.
pub trait BattlesnakeFactory {
    /// Get the name of the snake.
    fn name(&self) -> String;

    /// Return the /info route for the specific snake.
    ///
    /// This does not work right now, because the new Battlesnake API
    /// does not provide the snake name with the info request.
    fn info(&self) -> Value;

    /// Get a move from the snake.
    fn handle_move(&self, game: &Game, turn: &i32, board: &Board, you: &Battlesnake) -> Move;
}

/// A trait for the `BattlesnakeFactory` to be used by the rocket server.
pub type BoxedBattlesnakeFactory = Box<dyn BattlesnakeFactory + Send + Sync>;

/// A function to create a vector of all available factories.
#[must_use]
pub fn add_all_factories() -> Vec<BoxedBattlesnakeFactory> {
    vec![
        Box::new(BattleratFactory),
        Box::new(CarloConstrictorFactory),
    ]
}
