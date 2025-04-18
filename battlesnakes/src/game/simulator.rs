#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use std::collections::HashMap;

use super::game_types::{Board, Coord, Move};

pub struct Simulator {
    pub board: Board,
    pub turn: u32,
}

impl Simulator {
    /// Creates a new `Simulator` with the given `board`.
    pub fn from_board(board: Board) -> Simulator {
        Simulator { board, turn: 0 }
    }

    /// Gets the reasonable moves for the snake.
    ///
    /// The returned moves are the ones that do not collide with the snake's own body
    /// and do not move out of bounds.
    pub fn get_reasonable_moves(&mut self) -> Vec<Move> {
        let mut is_move_safe: HashMap<_, _> = vec![
            (Move::Up, true),
            (Move::Down, true),
            (Move::Left, true),
            (Move::Right, true),
        ]
        .into_iter()
        .collect();

        let my_head = &self.board.snakes[0].head;
        let my_neck = &self.board.snakes[0].body[1];

        // check for own neck
        if my_neck.x < my_head.x {
            is_move_safe.insert(Move::Left, false);
        } else if my_neck.x > my_head.x {
            is_move_safe.insert(Move::Right, false);
        } else if my_neck.y < my_head.y {
            is_move_safe.insert(Move::Down, false);
        } else if my_neck.y > my_head.y {
            is_move_safe.insert(Move::Up, false);
        }

        // check for out of bounds
        if my_head.x == 0 {
            is_move_safe.insert(Move::Left, false);
        }
        if my_head.x == self.board.width - 1 {
            is_move_safe.insert(Move::Right, false);
        }
        if my_head.y == 0 {
            is_move_safe.insert(Move::Down, false);
        }
        if my_head.y == self.board.height - 1 {
            is_move_safe.insert(Move::Up, false);
        }

        // check for collisions with own body
        for body_part in self.board.snakes[0].body.iter().skip(1) {
            if body_part.x == my_head.x - 1 && body_part.y == my_head.y {
                is_move_safe.insert(Move::Left, false);
            }
            if body_part.x == my_head.x + 1 && body_part.y == my_head.y {
                is_move_safe.insert(Move::Right, false);
            }
            if body_part.x == my_head.x && body_part.y == my_head.y - 1 {
                is_move_safe.insert(Move::Down, false);
            }
            if body_part.x == my_head.x && body_part.y == my_head.y + 1 {
                is_move_safe.insert(Move::Up, false);
            }
        }

        is_move_safe
            .iter()
            .filter(|(_, v)| **v)
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Makes the given move for the snake.
    ///
    /// This only works for a single snake. (you)
    pub fn make_move(&mut self, next_move: &Move) {
        let snake = self.board.snakes.first_mut().unwrap();

        let new_head = match next_move {
            Move::Up => Coord {
                x: snake.head.x,
                y: snake.head.y + 1,
            },
            Move::Down => Coord {
                x: snake.head.x,
                y: snake.head.y - 1,
            },
            Move::Left => Coord {
                x: snake.head.x - 1,
                y: snake.head.y,
            },
            Move::Right => Coord {
                x: snake.head.x + 1,
                y: snake.head.y,
            },
        };

        if snake.body.len() > 3 {
            snake.body.pop();
            snake.body.insert(0, new_head.clone());
            snake.head = new_head.clone();
        } else {
            snake.body.insert(0, new_head.clone());
            snake.head = new_head.clone();
        }

        // check for eaten food
        for food in &self.board.food {
            if snake.head.x == food.x && snake.head.y == food.y {
                snake.body.push(snake.body.last().unwrap().clone());
                snake.health = 100;
                break;
            }
        }
    }

    /// Checks for collisions for all snakes on the board.
    #[must_use]
    fn check_snake_collisions(&mut self) -> bool {
        for snake in &self.board.snakes {
            for other_snake in &self.board.snakes {
                for body_part in other_snake.body.iter().skip(1) {
                    if snake.head.x == body_part.x && snake.head.y == body_part.y {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Checks if any snake has eaten food.
    ///
    /// This resets the snake's health to 100 and adds a new body part.
    fn check_food(&mut self) {
        for snake in &mut self.board.snakes {
            for food in &self.board.food.clone() {
                if snake.head.x == food.x && snake.head.y == food.y {
                    // snake ate food
                    snake.body.push(snake.body.last().unwrap().clone());
                    snake.health = 100;
                    self.board.food.retain(|f| f != food);
                }
            }
        }
    }

    /// Checks if any snake is out of bounds.
    #[must_use]
    fn check_out_of_bounds(&mut self) -> bool {
        for snake in &self.board.snakes {
            if snake.head.x < 0
                || snake.head.x >= self.board.width
                || snake.head.y < 0
                || snake.head.y >= self.board.height
            {
                return true;
            }
        }

        false
    }

    /// Simulates a number of turns.
    pub fn simulate_turns(&mut self, turns: u32) -> u32 {
        for _ in 0..turns {
            let possible_moves = self.get_reasonable_moves();
            if possible_moves.is_empty() {
                return self.turn;
            }

            for snake in &mut self.board.snakes {
                let random_index = rand::random_range(0..possible_moves.len());
                let next_move = possible_moves[random_index].clone();

                let new_head = match next_move {
                    Move::Up => Coord {
                        x: snake.head.x,
                        y: snake.head.y + 1,
                    },
                    Move::Down => Coord {
                        x: snake.head.x,
                        y: snake.head.y - 1,
                    },
                    Move::Left => Coord {
                        x: snake.head.x - 1,
                        y: snake.head.y,
                    },
                    Move::Right => Coord {
                        x: snake.head.x + 1,
                        y: snake.head.y,
                    },
                };

                if snake.body.len() > 3 {
                    snake.body.pop();
                    snake.body.insert(0, new_head.clone());
                    snake.head = new_head.clone();
                } else {
                    snake.body.insert(0, new_head.clone());
                    snake.head = new_head.clone();
                }
            }

            // check for collisions
            if self.check_snake_collisions() {
                return self.turn;
            }

            // check for food
            self.check_food();

            // check for out of bounds
            if self.check_out_of_bounds() {
                return self.turn;
            }

            // decrease health with every turn
            for snake in &mut self.board.snakes {
                snake.health -= 1;

                if snake.health <= 0 {
                    return self.turn;
                }
            }

            self.turn += 1;
        }

        self.turn
    }
}
