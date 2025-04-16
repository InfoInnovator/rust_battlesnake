#![warn(clippy::all)]
#![warn(clippy::pedantic)]

#[macro_use]
extern crate rocket;

use std::env;

use battlesnakes::game::game_types::GameState;
use battlesnakes::{BoxedBattlesnakeFactory, add_all_factories};
use log::info;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Config, State};
use serde_json::{Value, json};

#[get("/")]
fn handle_index() -> Json<Value> {
    json!({
        "apiversion": "1",
        "author": "Malte",
        "color": "#879c6b",
        "head": "missile",
        "tail": "block-bum",
    })
    .into()
}

#[post("/start", format = "json", data = "<_start_req>")]
fn handle_start(_start_req: Json<GameState>) -> Status {
    Status::Ok
}

#[allow(clippy::needless_pass_by_value)]
#[post("/move", format = "json", data = "<move_req>")]
fn handle_move(
    move_req: Json<GameState>,
    factories: &State<Vec<BoxedBattlesnakeFactory>>,
) -> Json<Value> {
    let (game, turn, board, you) = (
        &move_req.game,
        &move_req.turn,
        &move_req.board,
        &move_req.you,
    );
    let next_move = factories
        .iter()
        .find(|snake| snake.name() == you.name)
        .unwrap()
        .handle_move(game, turn, board, you);

    info!("[{}] {}: {:?}", turn, you.name, next_move);

    Json(json!({"move": next_move.to_string()}))
}

#[post("/end", format = "json", data = "<_end_req>")]
fn handle_end(_end_req: Json<GameState>) -> Status {
    Status::Ok
}

#[launch]
fn rocket() -> _ {
    if env::var("RUST_LOG").is_err() {
        unsafe {
            env::set_var("RUST_LOG", "info");
        }
    }

    env_logger::init();

    info!("Starting Battlesnake Server...");

    let config = Config {
        address: std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
        port: 8000,
        ..Default::default()
    };

    rocket::build()
        .configure(config)
        .manage(add_all_factories())
        .attach(AdHoc::on_response("Server ID Middleware", |_, res| {
            Box::pin(async move {
                res.set_raw_header("Server", "battlesnake-rust");
            })
        }))
        .mount(
            "/",
            routes![handle_index, handle_start, handle_move, handle_end],
        )
}
