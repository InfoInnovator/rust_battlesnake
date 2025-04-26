use std::{collections::VecDeque, io::Write};

use battlesnakes::game::game_types::GameState;

pub struct SaveGameState {
    pub states: VecDeque<GameState>,
}

impl SaveGameState {
    pub fn new() -> Self {
        SaveGameState {
            states: VecDeque::new(),
        }
    }

    pub fn save_state(&mut self, state: GameState) {
        self.states.push_back(state);

        if self.states.len() > 10 {
            self.states.pop_front();
        }

        // Save the states to a file
        let file_path = "game_state.json";
        let file = std::fs::File::create(file_path).expect("Unable to create file");
        let mut writer = std::io::BufWriter::new(file);
        let json = serde_json::to_string(&self.states).expect("Unable to serialize");
        writer
            .write_all(json.as_bytes())
            .expect("Unable to write data");
    }
}
