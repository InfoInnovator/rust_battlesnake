use std::collections::HashMap;

use log::info;
use serde_json::{json, Value};

use crate::{Battlesnake, Board, Coord, Game, Move};

use pathfinding::prelude::astar;

pub fn info() -> Value {
    info!("INFO");

    return json!({
        "apiversion": "1",
        "author": "Maltereality", // Battlesnake Username
        "color": "#879c6b", // color
        "head": "missile", // head
        "tail": "block-bum", // tail
    });
}

pub fn start(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME START");
}

pub fn end(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME OVER");
}

pub fn get_move(_game: &Game, _turn: &i32, board: &Board, you: &Battlesnake) -> Move {
    // calculate from 4 areas which are least crowded
    let mut crowd: HashMap<&str, i32> = HashMap::from([
        ("right-top", 0),
        ("right-bottom", 0),
        ("left-top", 0),     
        ("left-bottom", 0),
    ]); 
    
    // calc number of objects/body parts in an area
    for y in (0..board.height).rev() {
        for x in 0..board.width {
            let current = &Coord { x: x, y: y as i32 };

            // handle: head, own body, food and other snakes
            let mut accounting: Vec<Coord> = Vec::new();
            accounting.push(you.head.clone());
            accounting.append(&mut you.body.clone());
            accounting.append(&mut board.food.clone());
            for other_snake in &board.snakes {
                accounting.append(&mut other_snake.body.clone());
            }

            // check 4 areas
            if x > board.width / 2 as i32 {
                // right side
                if y > (board.height / 2) as u32 {
                    // top
                    if accounting.contains(current) {
                        *crowd.get_mut("right-top").unwrap() += 1;
                    }
                } else {
                    // bottom
                    if accounting.contains(current) {
                        *crowd.get_mut("right-bottom").unwrap() += 1;
                    }
                }
            } else {
                // left side
                if y > (board.height / 2) as u32 {
                    // top
                    if accounting.contains(current) {
                        *crowd.get_mut("left-top").unwrap() += 1;
                    }
                } else {
                    // bottom
                    if accounting.contains(current) {
                        *crowd.get_mut("left-bottom").unwrap() += 1;
                    }
                }
            }
        }
    }

    // TODO: choose goal by using the crowd map and determine the intermediate goal
    // determine area with lowest crowd score
    let best_next_area = crowd
        .iter()
        .min_by(|a, b| a.1.cmp(b.1))
        .map(|(k, _v)| k)
        .unwrap();

    // calc closest apple
    let mut goal = &board.food[0];
    for food_cand in &board.food {
        if &you.body[0].distance(food_cand) < &you.body[0].distance(goal) {
            goal = food_cand;
        }
    }
    
    // calculate path to goal
    let path = astar(
        &you.body[0],
        |coord| coord.successors(&you.body, &board.snakes, (board.width, board.height)),
        |coord| coord.distance(&goal),
        |coord| *coord == *goal,
    ).unwrap();

    // display board with calculated path
    board.display_board(you, &path.0);

    // local planner
    let next_step = &path.0[1];
    if you.body[0].check_right(next_step) { return Move::Right }
    if you.body[0].check_down(next_step) { return Move::Down }
    if you.body[0].check_left(next_step) { return Move::Left }
    if you.body[0].check_up(next_step) { return Move::Up }

    info!("no other moves found; moving down");
    return Move::Down;
}
