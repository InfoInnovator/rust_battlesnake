use crate::game::game_types::Coord;
use std::collections::HashMap;

use log::{info, warn};
use rand::seq::SliceRandom;
use serde_json::{Value, json};

use crate::{
    BattlesnakeFactory,
    game::game_types::{Battlesnake, Board, Game, Move},
};

pub struct BattleratFactory;

impl BattlesnakeFactory for BattleratFactory {
    fn name(&self) -> String {
        "Battlerat".to_string()
    }

    fn info(&self) -> Value {
        info!("INFO");

        json!({
            "apiversion": "1",
            "author": "Malte",
            "color": "#879c6b",
            "head": "missile",
            "tail": "block-bum",
        })
    }

    fn handle_move(&self, _game: &Game, _turn: &i32, board: &Board, you: &Battlesnake) -> Move {
        let mut is_move_safe: HashMap<_, _> = vec![
            (Move::Up, true),
            (Move::Down, true),
            (Move::Left, true),
            (Move::Right, true),
        ]
        .into_iter()
        .collect();

        you.head.check_collisions(board, &mut is_move_safe);

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
                return max_area.0.clone();
            }
        }

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

        chosen.clone()
    }
}

#[derive(PartialEq)]
enum FieldType {
    Discovered,
    Free,
    Blocked,
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
impl Move {
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
