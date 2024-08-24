// Welcome to
// __________         __    __  .__                               __
// \______   \_____ _/  |__/  |_|  |   ____   ______ ____ _____  |  | __ ____
//  |    |  _/\__  \\   __\   __\  | _/ __ \ /  ___//    \\__  \ |  |/ // __ \
//  |    |   \ / __ \|  |  |  | |  |_\  ___/ \___ \|   |  \/ __ \|    <\  ___/
//  |________/(______/__|  |__| |____/\_____>______>___|__(______/__|__\\_____>
//
// This file can be a nice home for your Battlesnake logic and helper functions.
//
// To get you started we've included code to prevent your Battlesnake from moving backwards.
// For more info see docs.battlesnake.com

use log::info;
use serde_json::{json, Value};

use crate::{Battlesnake, Board, Game, Move};

use pathfinding::prelude::astar;

/*
TODO:
- [ ] handle path none value
*/

// info is called when you create your Battlesnake on play.battlesnake.com
// and controls your Battlesnake's appearance
// TIP: If you open your Battlesnake URL in a browser you should see this data
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

// start is called when your Battlesnake begins a game
pub fn start(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME START");
}

// end is called when your Battlesnake finishes a game
pub fn end(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME OVER");
}

pub fn get_move(_game: &Game, _turn: &i32, board: &Board, you: &Battlesnake) -> Move {
    // calculate from 4 areas which are least crowded
    // let mut crowd: HashMap<i32, i32> = HashMap::from([
    //     (0, 0),
    //     (1, 0),
    //     (2, 0),
    //     (3, 0),
    // ]); 
    
    // // map area to score
    // you.body.iter().for_each(|body_part| {
    //     if body_part.x < board.width / 2 { // left field
    //         if body_part.y > (board.height / 2) as i32 { // top
    //             crowd.entry(0).and_modify(|e| *e += 1);
    //         } else { // bottom
    //             crowd.entry(2).and_modify(|e| *e += 1);
    //         }
    //     } else { // right field
    //         if body_part.y > (board.height / 2) as i32 { // top
    //             crowd.entry(1).and_modify(|e| *e += 1);
    //         } else { // bottom
    //             crowd.entry(3).and_modify(|e| *e += 1);
    //         }
    //     }
    // });

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
        |coord| coord.successors(&you.body, (board.width, board.height)),
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

    return Move::Down;
}
