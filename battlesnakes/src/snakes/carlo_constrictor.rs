#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use crate::{BattlesnakeFactory, game::mcts::Mcts};

pub struct CarloConstrictorFactory;

impl BattlesnakeFactory for CarloConstrictorFactory {
    fn name(&self) -> String {
        "CarloConstrictor".to_string()
    }

    fn info(&self) -> serde_json::Value {
        serde_json::json!({
            "apiversion": "1",
            "author": "Malte",
            "color": "#00FF00",
            "head": "default",
            "tail": "default",
        })
    }

    fn handle_move(
        &self,
        _game: &crate::game::game_types::Game,
        turn: &i32,
        board: &crate::game::game_types::Board,
        _you: &crate::game::game_types::Battlesnake,
    ) -> crate::game::game_types::Move {
        Mcts::new(500).get_move(board, turn)
    }
}
