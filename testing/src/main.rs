use std::{
    env::{self},
    process::Command,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut wins: u16 = 0;
    let num_games = args[1].parse::<i16>().unwrap();

    for i in 0..num_games {
        let output = Command::new("/home/malte/go/bin/battlesnake")
            .arg("play")
            .args(["-W", "11"])
            .args(["-H", "11"])
            .args(["--name", "'old'"])
            .args(["--url", "https://snake.maltesparenb.org"])
            .args(["--name", "'new'"])
            .args(["--url", "http://localhost:8000"])
            .args(["-g", "duels"])
            .output()
            .unwrap();

        let output_str = String::from_utf8(output.stderr.clone()).unwrap();
        let winner = output_str
            .split("Game completed after")
            .nth(1)
            .unwrap()
            .split("'")
            .nth(1)
            .unwrap();

        println!("game ({i}) won by {}", winner);

        if winner == "new" {
            wins += 1;
        }
    }

    let win_rate = (f32::from(wins) / f32::from(num_games)) * 100.0;
    println!(
        "{:.2}% win rate with {} wins in {} games",
        win_rate, wins, num_games
    );
}
