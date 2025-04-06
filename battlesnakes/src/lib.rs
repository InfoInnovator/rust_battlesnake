use game::game_types::{Battlesnake, Board, Game, Move};
use serde_json::Value;
use snakes::battlerat::BattleratFactory;

pub mod game;
pub mod snakes;

pub trait BattlesnakeFactory {
    fn name(&self) -> String;
    fn info(&self) -> Value;
    fn handle_move(&self, game: &Game, turn: &i32, board: &Board, you: &Battlesnake) -> Move;
}

pub type BoxedBattlesnakeFactory = Box<dyn BattlesnakeFactory + Send + Sync>;

pub fn add_all_factories() -> Vec<BoxedBattlesnakeFactory> {
    vec![Box::new(BattleratFactory)]
}
