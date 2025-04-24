#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use std::path::PathBuf;

use clap::Parser;
use creator::simple_creator::Creator;
use executor::execute_all::Executor;

mod creator;
mod executor;

/*
TODO
+ add legend for shortcuts
+ add better layout

+ visualize export
+ visualize selected next_valid_moves
+ read json scenarios from file to be used by the creator
+ better error handling for sending requests

+ implement tests respecting the given timeout from the game state
*/

/// R.U.N.A. - Real-time Unfolding of Navigational Actions
fn main() {
    let cli = Cli::parse();

    if let Some(output_file) = cli.output_file {
        let mut desc = String::new();
        if let Some(description) = cli.description {
            desc = description;
        }

        let mut creator = Creator::new(cli.field_size, output_file, desc);
        creator.run();
    } else if cli.test {
        if let Some(snake_name) = cli.snake_name {
            let mut executor = Executor::new(snake_name);
            executor.run_all_scenarios();
        }
    }
}

#[derive(Parser)]
struct Cli {
    /// Run tests using all available scenarios
    #[arg(short, long)]
    test: bool,

    /// Name of the snake to be tested
    #[arg(short, long)]
    snake_name: Option<String>,

    /// Filepath to the scenario to be created
    #[arg(short)]
    output_file: Option<PathBuf>,

    /// Size of the field (assuming square field)
    #[arg(short, default_value_t = 11)]
    field_size: i32,

    /// Description of what the scenario is about
    #[arg(short, long)]
    description: Option<String>,
}
