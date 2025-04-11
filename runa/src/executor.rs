use std::time::Duration;

use battlesnakes::game::game_types::Move;
use crossterm::style::Stylize;
use reqwest::header::{ACCEPT_ENCODING, CONTENT_TYPE};

use crate::creator::ExportedGameState;

pub struct Executor {
    snake_name: String,
    statistics: TestStatistics,
}

impl Executor {
    pub fn new(snake_name: String) -> Self {
        Self {
            snake_name,
            statistics: TestStatistics::default(),
        }
    }

    pub fn run_all_scenarios(&mut self) {
        println!("Running all scenarios...\n");

        let scenarios = std::fs::read_dir("./scenarios").unwrap();
        scenarios.for_each(|file| {
            let file = file.unwrap();
            println!("Test {:?}", file.file_name());

            // extract scenario from file
            let scenario_str = std::fs::read_to_string(file.path()).unwrap();
            let mut exported_game_state =
                serde_json::from_str::<ExportedGameState>(&scenario_str).unwrap();

            // replace "you" snake name with the snake name to be tested
            exported_game_state.game_state.you.name = self.snake_name.clone();

            // send request to the server
            let client = reqwest::blocking::Client::new();
            let response_time = std::time::Instant::now();
            let response = client
                .post("http://localhost:8000/move")
                .header(CONTENT_TYPE, "application/json")
                .header(ACCEPT_ENCODING, "gzip")
                .body(serde_json::to_string(&exported_game_state.game_state).unwrap())
                .send()
                .unwrap();

            if response.status() == 200 {
                let response_move =
                    serde_json::from_str::<MoveResponse>(&response.text().unwrap()).unwrap();

                let time_elapsed = response_time.elapsed();
                if exported_game_state
                    .next_valid_moves
                    .contains(&response_move.r#move)
                {
                    println!(
                        "Got move [{}] from valid moves {:?}",
                        response_move.r#move, exported_game_state.next_valid_moves
                    );
                    println!("Response time: {:?}", time_elapsed);
                    println!("{}", "TEST PASSED!".green());

                    self.statistics.num_passed += 1;
                } else {
                    println!(
                        "Got move [{}] but expected one of: {:?}",
                        response_move.r#move, exported_game_state.next_valid_moves
                    );
                    println!("Response time: {:?}", time_elapsed);
                    println!("{}", "TEST FAILED!".red());

                    self.statistics.num_failed += 1;
                }

                self.statistics.response_times.push(time_elapsed);
            }

            println!();
        });

        // print summary of tests
        println!("Summary");
        println!("Total tests run: {}", {
            self.statistics.num_passed + self.statistics.num_failed
        });
        println!("Tests passed: {}", self.statistics.num_passed);
        println!("Tests failed: {}", self.statistics.num_failed);
        println!("Test rate: {:.2}%", {
            let total = self.statistics.num_passed + self.statistics.num_failed;
            if total == 0 {
                0.0
            } else {
                (self.statistics.num_passed as f32 / total as f32) * 100.0
            }
        });
        println!(
            "Total time elapsed: {:?}",
            self.statistics.response_times.iter().sum::<Duration>()
        );
        println!("Average response time: {:?}", {
            let total_time: Duration = self.statistics.response_times.iter().sum();
            total_time / self.statistics.response_times.len() as u32
        });
        println!("Min response time: {:?}", {
            self.statistics
                .response_times
                .iter()
                .min()
                .unwrap_or(&Duration::ZERO)
        });
        println!("Max response time: {:?}", {
            self.statistics
                .response_times
                .iter()
                .max()
                .unwrap_or(&Duration::ZERO)
        });
    }
}

#[derive(serde::Deserialize, Debug)]
struct MoveResponse {
    r#move: Move,
}

#[derive(Default)]
struct TestStatistics {
    num_passed: u32,
    num_failed: u32,
    response_times: Vec<std::time::Duration>,
}
