#[macro_use]
extern crate rocket;

use core::fmt;
use log::info;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::serde::{json::Json, Deserialize};
use serde::Serialize;
use serde_json::{json, Value};
use core::fmt;
use std::collections::HashMap;
use std::{env, vec};

use std::hash::Hash;

mod logic;

/*
TODO:
- [ ] draw other snakes in gameboard display
*/

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
    height: u32,
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
impl Board {
    fn display_board(&self, me: &Battlesnake, path: &Vec<Coord>) {
        print!("  |");
        for x in 0..self.width {
            print!("{}|", x);
        }
        println!("");

        for y in (0..self.height).rev() {
            print!("{:2}", y);
            print!("|");

            for x in 0..self.width {
                let current = &Coord { x: x, y: y as i32 };

                if self.food.contains(current) {
                    // draw food
                    print!("a");
                } else if me.body.contains(current) && !me.head.eq(current) {
                    // draw body
                    print!("O");
                } else if me.head.eq(current) {
                    // draw head
                    print!("h");
                } else if path.contains(current) {
                    // draw path
                    print!("+");
                } else {
                    // draw empty field
                    print!("_");
                }

                print!("|");
            }

            println!("");
        }
    }
}

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

#[derive(Deserialize, Serialize, Debug, Eq, Hash, Clone)]
pub struct Coord {
    x: i32,
    y: i32,
}

#[derive(serde::Serialize)]
pub enum Move {
    Up,
    Right,
    Down,
    Left,
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

impl PartialEq for Coord {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Coord {
    fn distance(&self, other: &Coord) -> u32 {
        let diff_x = (self.x - other.x) as f32;
        let diff_y = (self.y - other.y) as f32;
        (f32::powi(f32::abs(diff_x), 2) + f32::powi(f32::abs(diff_y), 2).ceil()) as u32
    }

    fn successors(&self, my_body: &Vec<Coord>, other_snakes: &Vec<Battlesnake>, field_dim: (i32, u32)) -> Vec<(Coord, u32)> {
        let mut start = vec![
            Coord{x: self.x+1, y: self.y},
            Coord{x: self.x, y: self.y-1},
            Coord{x: self.x-1, y: self.y},
            Coord{x: self.x, y: self.y+1},
        ];

        // obstacle: body
        for body_part in my_body {
            start.retain(|val| body_part != val);
        }

        // obstacle: boundary
        start.retain(|val| val.x >= 0 && val.y >= 0 && val.x <= field_dim.0 && val.y <= (field_dim.1) as i32);

        // obstacle: other snakes
        for other_snake in other_snakes {
            for body_part in &other_snake.body {
                start.retain(|val| body_part != val);
            }
        }
        
        start.into_iter().map(|coord| (coord, 1)).collect()
    }

    fn check_left(&self, other: &Coord) -> bool {
        if self.x - 1 == other.x { return true }
        false
    }

    fn check_right(&self, other: &Coord) -> bool {
        if self.x + 1 == other.x { return true }
        false
    }

    fn check_up(&self, other: &Coord) -> bool {
        if self.y + 1 == other.y { return true }
        false
    }

    fn check_down(&self, other: &Coord) -> bool {
        if self.y - 1 == other.y { return true }
        false
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
    if let Ok(port) = env::var("PORT") {
        env::set_var("ROCKET_PORT", &port);
    }

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
