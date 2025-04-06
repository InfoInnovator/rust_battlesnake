use core::fmt;
use std::collections::HashMap;

use log::warn;
use rand::seq::SliceRandom;
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
    height: i32,
    width: i32,
    food: Vec<Coord>,
    snakes: Vec<Battlesnake>,
    hazards: Vec<Coord>,
}

/*
Snake:
    + Head: h
    + Body: O
Apple: a
*/
impl Board {
    fn print(&self) {
        print!("  |");
        for x in 0..self.width {
            print!("{}|", x);
        }
        println!();

        for y in (0..self.height).rev() {
            print!("{:2}", y);
            print!("|");

            for x in 0..self.width {
                let current = &Coord { x, y };

                let mut contains_snake = false;
                for snake in &self.snakes {
                    for part in &snake.body {
                        if part == current {
                            if current == &snake.body[0] {
                                // draw snake head
                                print!("h");
                            } else {
                                // draw snake body
                                print!("O");
                            }
                            contains_snake = true;
                            break;
                        }
                    }
                    if contains_snake {
                        break;
                    }
                }

                if !contains_snake {
                    // draw empty field
                    print!("_");
                }

                print!("|");
            }

            println!();
        }
    }
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

impl Battlesnake {
    fn equals(&self, other: &Battlesnake) -> bool {
        self.body == other.body
    }

    fn get_future_snake(&mut self, next_move: &Move) {
        let new_head_pos = next_move.get_coord(&self.head);
        let mut new_body = vec![new_head_pos.clone()];
        self.body.truncate(self.body.len() - 1);
        new_body.append(&mut self.body);

        self.head = new_head_pos;
        self.body = new_body;
    }
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

    pub fn check_collisions(&self, board: &Board, input_moves: &mut HashMap<Move, bool>) {
        self.check_out_of_bounds(board, input_moves);
        self.check_body_collision(board, input_moves);
        self.check_head_to_head_collisions(board, input_moves);
    }

    pub fn floodfill(&self, board: &Board) -> i32 {
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
}

#[derive(serde::Serialize, Hash, PartialEq, Eq, Clone, Debug)]
pub enum Move {
    Up,
    Right,
    Down,
    Left,
}

impl Move {
    fn simulate_step(&self, you: &Battlesnake, board: &mut Board) {
        /*
        TODO
            + optimize performance
         */

        // perform predefined step for own snake
        board.snakes.iter_mut().for_each(|snake| {
            if snake.id == you.id {
                snake.get_future_snake(self);
            }
        });

        // simulate one random move for each snake
        for i in 0..board.snakes.len() {
            let mut is_move_safe: HashMap<_, _> = vec![
                (Move::Up, true),
                (Move::Down, true),
                (Move::Left, true),
                (Move::Right, true),
            ]
            .into_iter()
            .collect();

            board.snakes[i]
                .head
                .check_collisions(board, &mut is_move_safe);

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

            board.snakes[i].get_future_snake(chosen);
        }
    }

    pub fn get_coord(&self, origin: &Coord) -> Coord {
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
    pub game: Game,
    pub turn: i32,
    pub board: Board,
    pub you: Battlesnake,
}
