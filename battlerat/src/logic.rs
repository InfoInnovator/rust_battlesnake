use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use log::{info, warn};
use rand::seq::SliceRandom;
use serde_json::{json, Value};

use crate::{Battlesnake, Board, Game, Move};

pub fn info() -> Value {
    info!("INFO");

    json!({
        "apiversion": "1",
        "author": "Maltereality",
        "color": "#879c6b",
        "head": "missile",
        "tail": "block-bum",
    })
}

pub fn start(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME START");
}

pub fn end(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME OVER");
}

pub fn get_move(_game: &Game, _turn: &i32, board: &Board, you: &Battlesnake) -> (Move, Duration) {
    let start = Instant::now();

    let mut is_move_safe: HashMap<_, _> = vec![
        (Move::Up, true),
        (Move::Down, true),
        (Move::Left, true),
        (Move::Right, true),
    ]
    .into_iter()
    .collect();

    you.head.check_collisions(board, &mut is_move_safe);

    // calculate floodfill for every possible direction
    let mut areas: HashMap<Move, i32> = HashMap::new();
    is_move_safe.iter().for_each(|(k, v)| {
        if *v {
            let area_size = k.get_coord(&you.head).floodfill(board);
            areas.insert(k.clone(), area_size);
        }
    });

    if !areas.is_empty() {
        let max_area = areas.iter().max_by(|a, b| a.1.cmp(b.1)).unwrap();
        let all_equal = areas.iter().all(|elem| elem.1 == max_area.1);

        // if direction doesnt mattter
        if !all_equal {
            return (max_area.0.clone(), start.elapsed());
        }
    }

    // do something with a "tree"
    // println!("running sim");
    // println!("safe moves before: {:?}", is_move_safe);
    let test = is_move_safe.clone();
    is_move_safe.clone().iter().for_each(|(m, k)| {
        if *k {
            let mut new_board = board.clone();
            for i in 0..10 {
                let prev_snake = board
                    .snakes
                    .iter()
                    .find(|snake| snake.id == you.id)
                    .unwrap();

                m.simulate_step(you, &mut new_board);

                let future_snake = board
                    .snakes
                    .iter()
                    .find(|snake| snake.id == you.id)
                    .unwrap();

                // -------------- evaluate board at the end of the simulation --------------

                // if snake hasnt changed -> bad game
                if prev_snake.equals(future_snake) {
                    is_move_safe.insert(m.clone(), false);
                }

                println!("game {} simulating {}", i, m);
                new_board.print();
                println!();
            }

            // remove move if snake lost game in sim
            // if new_board.snakes.iter().any(|snake| snake.id == you.id) {
            // is_move_safe.insert(m.clone(), false);
            // println!("bad move found");
            // }
        }
    });
    // println!("safes moves after: {:?}", is_move_safe);
    let changed = is_move_safe != test;
    println!("{}", changed);

    let safe_moves = is_move_safe
        .into_iter()
        .filter(|&(_, v)| v)
        .map(|(k, _)| k)
        .collect::<Vec<_>>();

    let chosen = safe_moves
        .choose(&mut rand::thread_rng())
        .unwrap_or_else(|| {
            warn!("NO POSSIBLE MOVES FOUND. returning DOWN as default");
            &Move::Down
        });

    (chosen.clone(), start.elapsed())
}
