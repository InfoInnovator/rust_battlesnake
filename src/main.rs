#[macro_use]
extern crate rocket;

use core::fmt;
use log::info;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::serde::{json::Json, Deserialize};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::hash::Hash;
use std::{env, vec};

mod logic;

// API and Response Objects
// See https://docs.battlesnake.com/api

#[derive(Deserialize, Serialize, Debug)]
pub struct Game {
    id: String,
    ruleset: HashMap<String, Value>,
    timeout: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Board {
    height: i32,
    width: i32,
    food: Vec<Coord>,
    snakes: Vec<Battlesnake>,
    hazards: Vec<Coord>,
}

/*
Snake:
    + Own
        + Head: h
        + Body: O
    + Other
        + Head: l
        + Body: L
Path: +
Apple: a
*/
// impl Board {
//     fn display_board(&self, me: &Battlesnake, path: &[Coord]) {
//         print!("  |");
//         for x in 0..self.width {
//             print!("{}|", x);
//         }
//         println!();

//         for y in (0..self.height).rev() {
//             print!("{:2}", y);
//             print!("|");

//             for x in 0..self.width {
//                 let current = &Coord { x, y };

//                 if self.food.contains(current) {
//                     // draw food
//                     print!("a");
//                 } else if me.body.contains(current) && !me.head.eq(current) {
//                     // draw body
//                     print!("O");
//                 } else if me.head.eq(current) {
//                     // draw head
//                     print!("h");
//                 } else if path.contains(current) {
//                     // draw path
//                     print!("+");
//                 } else {
//                     // draw empty field
//                     print!("_");
//                 }

//                 print!("|");
//             }

//             println!();
//         }
//     }
// }

#[derive(Deserialize, Serialize, Debug)]
pub struct Battlesnake {
    id: String,
    name: String,
    health: i32,
    body: Vec<Coord>,
    head: Coord,
    length: i32,
    latency: String,
    shout: Option<String>,
}

#[derive(PartialEq)]
enum FieldType {
    Discovered,
    Free,
    Blocked,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Coord {
    x: i32,
    y: i32,
}

impl Coord {
    fn check_out_of_bounds(&self, board: &Board, input_moves: &mut HashMap<Move, bool>) {
        if self.x - 1 < 0 {
            input_moves.insert(Move::Left, false);
        }
        if self.x + 1 > board.width - 1 {
            input_moves.insert(Move::Right, false);
        }
        if self.y - 1 < 0 {
            input_moves.insert(Move::Down, false);
        }
        if self.y + 1 > board.height - 1 {
            input_moves.insert(Move::Up, false);
        }
    }

    fn check_body_collision(&self, board: &Board, input_moves: &mut HashMap<Move, bool>) {
        let mut bodies: Vec<&Coord> = Vec::new();
        board.snakes.iter().for_each(|snake| {
            snake.body.iter().for_each(|body_part| {
                bodies.push(body_part);
            });
        });
        bodies.retain(|elem| *elem != self);

        input_moves.clone().iter().for_each(|(m, k)| {
            if *k && bodies.contains(&&m.get_coord(self)) {
                input_moves.insert(m.clone(), false);
            }
        });
    }

    fn check_head_to_head_collisions(&self, board: &Board, input_moves: &mut HashMap<Move, bool>) {
        let mut snake_heads: Vec<Coord> = board
            .snakes
            .iter()
            .map(|snake| snake.body[0].clone())
            .collect();
        snake_heads.retain(|head| head != self);

        let input_moves_before = input_moves.clone();

        input_moves.clone().iter().for_each(|(m, k)| {
            if *k {
                let snake_head_moves = [Move::Down, Move::Left, Move::Right, Move::Up];

                snake_heads.iter().for_each(|snake_head| {
                    snake_head_moves.iter().for_each(|head_move| {
                        if m.get_coord(self) == head_move.get_coord(snake_head) {
                            input_moves.insert(m.clone(), false);
                        }
                    });
                });
            }
        });

        if input_moves.values().all(|elem| elem == &false) {
            input_moves_before.iter().for_each(|(m, k)| {
                input_moves.insert(m.clone(), *k);
            });
        }
    }

    fn check_collisions(&self, board: &Board, input_moves: &mut HashMap<Move, bool>) {
        self.check_out_of_bounds(board, input_moves);
        self.check_body_collision(board, input_moves);
        self.check_head_to_head_collisions(board, input_moves);
    }

    fn floodfill(&self, board: &Board) -> i32 {
        let mut custom_board: HashMap<Coord, FieldType> = HashMap::new();

        // add all coords
        for y in (0..board.height).rev() {
            for x in 0..board.width {
                custom_board.insert(Coord { x, y }, FieldType::Free);
            }
        }

        // add all snakes
        for snake in &board.snakes {
            for part in &snake.body {
                custom_board.insert(part.clone(), FieldType::Blocked);
            }
        }

        let mut stack: Vec<Coord> = Vec::new();
        stack.push(self.clone());

        let mut area_size = 0;

        while let Some(v) = stack.pop() {
            if custom_board.get(&v) != Some(&FieldType::Discovered)
                && custom_board.get(&v) != Some(&FieldType::Blocked)
            {
                custom_board.insert(v.clone(), FieldType::Discovered);

                let mut is_move_safe: HashMap<_, _> = vec![
                    (Move::Up, true),
                    (Move::Down, true),
                    (Move::Left, true),
                    (Move::Right, true),
                ]
                .into_iter()
                .collect();

                v.check_collisions(board, &mut is_move_safe);

                for (m, k) in is_move_safe {
                    if k {
                        let next = m.get_coord(&v);
                        stack.push(next);
                    }
                }

                area_size += 1;
            }
        }

        area_size
    }

    fn distance(&self, other: &Coord) -> i32 {
        i32::abs(self.x - other.x) + i32::abs(self.y - other.y)
    }
}

#[derive(serde::Serialize, Hash, PartialEq, Eq, Clone, Debug)]
pub enum Move {
    Up,
    Right,
    Down,
    Left,
}

impl Move {
    fn get_coord(&self, origin: &Coord) -> Coord {
        if self == &Move::Up {
            Coord {
                x: origin.x,
                y: origin.y + 1,
            }
        } else if self == &Move::Down {
            Coord {
                x: origin.x,
                y: origin.y - 1,
            }
        } else if self == &Move::Left {
            Coord {
                x: origin.x - 1,
                y: origin.y,
            }
        } else {
            Coord {
                x: origin.x + 1,
                y: origin.y,
            }
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Move::Up => write!(f, "up"),
            Move::Right => write!(f, "right"),
            Move::Down => write!(f, "down"),
            Move::Left => write!(f, "left"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GameState {
    game: Game,
    turn: i32,
    board: Board,
    you: Battlesnake,
}

#[get("/")]
fn handle_index() -> Json<Value> {
    Json(logic::info())
}

#[post("/start", format = "json", data = "<start_req>")]
fn handle_start(start_req: Json<GameState>) -> Status {
    logic::start(
        &start_req.game,
        &start_req.turn,
        &start_req.board,
        &start_req.you,
    );

    Status::Ok
}

#[post("/move", format = "json", data = "<move_req>")]
fn handle_move(move_req: Json<GameState>) -> Json<Value> {
    let response = logic::get_move(
        &move_req.game,
        &move_req.turn,
        &move_req.board,
        &move_req.you,
    );

    info!("MOVE {}: {}", &move_req.turn, response);
    Json(json!({"move": response.to_string()}))
}

#[post("/end", format = "json", data = "<end_req>")]
fn handle_end(end_req: Json<GameState>) -> Status {
    logic::end(&end_req.game, &end_req.turn, &end_req.board, &end_req.you);

    Status::Ok
}

#[launch]
fn rocket() -> _ {
    // Lots of web hosting services expect you to bind to the port specified by the `PORT`
    // environment variable. However, Rocket looks at the `ROCKET_PORT` environment variable.
    // If we find a value for `PORT`, we set `ROCKET_PORT` to that value.
    env::set_var("ROCKET_PORT", "8000");

    // We default to 'info' level logging. But if the `RUST_LOG` environment variable is set,
    // we keep that value instead.
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    env_logger::init();

    info!("Starting Battlesnake Server...");

    rocket::build()
        .attach(AdHoc::on_response("Server ID Middleware", |_, res| {
            Box::pin(async move {
                res.set_raw_header("Server", "battlesnake/github/starter-snake-rust");
            })
        }))
        .mount(
            "/",
            routes![handle_index, handle_start, handle_move, handle_end],
        )
}
