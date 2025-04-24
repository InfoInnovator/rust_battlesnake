#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use std::time::Duration;

use battlesnakes::game::game_types::Move;
use crossterm::style::Stylize;
use reqwest::header::{ACCEPT_ENCODING, CONTENT_TYPE};

use crate::creator::simple_creator::ExportedGameState;

pub struct Executor {
    snake_name: String,
    statistics: TestStatistics,
}

impl Executor {
    #[must_use]
    pub fn new(snake_name: String) -> Self {
        Self {
            snake_name,
            statistics: TestStatistics::default(),
        }
    }

    /// Run all scenarios in the `scenarios` directory
    ///
    /// # Panics
    pub fn run_all_scenarios(&mut self) {
        println!("Running all scenarios...\n");

        let scenarios: Vec<_> = std::fs::read_dir("scenarios").unwrap().collect();

        if scenarios.is_empty() {
            println!("No scenarios found in the `scenarios` directory.");
            return;
        }

        for file in scenarios {
            let file = file.unwrap();
            println!("Test {:?}", file.file_name());

            // extract scenario from file
            let scenario_str = std::fs::read_to_string(file.path()).unwrap();
            let mut exported_game_state =
                serde_json::from_str::<ExportedGameState>(&scenario_str).unwrap();

            // replace "you" snake name with the snake name to be tested
            exported_game_state
                .game_state
                .you
                .name
                .clone_from(&self.snake_name);

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
                    println!("Response time: {time_elapsed:?}");
                    println!("{}", "TEST PASSED!".green());

                    self.statistics.num_passed += 1;
                } else {
                    println!(
                        "Got move [{}] but expected one of: {:?}",
                        response_move.r#move, exported_game_state.next_valid_moves
                    );
                    println!("Response time: {time_elapsed:?}");
                    println!("{}", "TEST FAILED!".red());

                    self.statistics.num_failed += 1;
                }

                self.statistics.response_times.push(time_elapsed);
            }

            println!();
        }

        // print summary of tests
        println!("Summary");
        println!("Total tests run: {}", {
            self.statistics.num_passed + self.statistics.num_failed
        });
        println!("Tests passed: {}", self.statistics.num_passed);
        println!("Tests failed: {}", self.statistics.num_failed);
        println!("Success rate: {:.2}%", {
            let total = self.statistics.num_passed + self.statistics.num_failed;
            if total == 0 {
                0.0
            } else {
                (f64::from(self.statistics.num_passed) / f64::from(total)) * 100.0
            }
        });
        println!(
            "Total time elapsed: {:?}",
            self.statistics.response_times.iter().sum::<Duration>()
        );
        println!("Average response time: {:?}", {
            let total_time: Duration = self.statistics.response_times.iter().sum();
            total_time / self.statistics.response_times.len().try_into().unwrap()
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
    num_passed: i32,
    num_failed: i32,
    response_times: Vec<std::time::Duration>,
}
