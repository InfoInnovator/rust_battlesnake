use core::fmt;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug)]
pub struct Game {
    id: String,
    ruleset: HashMap<String, Value>,
    timeout: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Board {
    pub height: i32,
    pub width: i32,
    food: Vec<Coord>,
    pub snakes: Vec<Battlesnake>,
    hazards: Vec<Coord>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Battlesnake {
    pub id: String,
    pub name: String,
    pub health: i32,
    pub body: Vec<Coord>,
    pub head: Coord,
    pub length: i32,
    pub latency: String,
    pub shout: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

#[derive(serde::Serialize, Hash, PartialEq, Eq, Clone, Debug)]
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

#[derive(Deserialize, Serialize, Debug)]
pub struct GameState {
    pub game: Game,
    pub turn: i32,
    pub board: Board,
    pub you: Battlesnake,
}
