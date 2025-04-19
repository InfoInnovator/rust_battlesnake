#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use std::collections::HashMap;

use super::game_types::{Coord, GameState, Move};

pub struct Simulator {
    pub game_state: GameState,
}

impl Simulator {
    /// Creates a new `Simulator` with the given `board`.
    pub fn from_gamestate(game_state: &mut GameState) -> Simulator {
        // reset the turns to always simulate from 0
        game_state.turn = 0;

        Simulator {
            game_state: game_state.clone(),
        }
    }

    /// Gets the reasonable moves for the snake.
    ///
    /// The returned moves are the ones that do not collide with the snake's own body
    /// and do not move out of bounds.
    pub fn get_reasonable_moves(&mut self, snake_id: String) -> Vec<Move> {
        let mut is_move_safe: HashMap<_, _> = vec![
            (Move::Up, true),
            (Move::Down, true),
            (Move::Left, true),
            (Move::Right, true),
        ]
        .into_iter()
        .collect();

        let my_head = match &self
            .game_state
            .board
            .snakes
            .iter()
            .find(|s| s.id == snake_id)
        {
            Some(snake) => snake.head.clone(),
            None => return vec![],
        };
        let my_neck = &self
            .game_state
            .board
            .snakes
            .iter()
            .find(|s| s.id == snake_id)
            .unwrap()
            .body[1];

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
        if my_head.x == self.game_state.board.width - 1 {
            is_move_safe.insert(Move::Right, false);
        }
        if my_head.y == 0 {
            is_move_safe.insert(Move::Down, false);
        }
        if my_head.y == self.game_state.board.height - 1 {
            is_move_safe.insert(Move::Up, false);
        }

        // check for collisions with body parts
        for snake in &self.game_state.board.snakes {
            for body_part in &snake.body {
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
        }

        is_move_safe
            .iter()
            .filter(|(_, v)| **v)
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Makes the given move on the board for the snake.
    pub fn make_move(&mut self, next_move: &Move, snake_id: String) {
        let snake = self
            .game_state
            .board
            .snakes
            .iter_mut()
            .find(|s| s.id == snake_id)
            .unwrap();

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
        for food in &self.game_state.board.food {
            if snake.head.x == food.x && snake.head.y == food.y {
                snake.body.push(snake.body.last().unwrap().clone());
                snake.health = 100;
                break;
            }
        }

        // if the snake is also "you" then update "you" in GameState
        if self.game_state.you.id == snake.id {
            self.game_state.you = snake.clone();
        }
    }

    /// Checks for collisions for all snakes on the board.
    ///
    /// It return true if there was a collision involving 'you'.
    /// This currently does not check for correct head-to-head collisions
    /// of the snakes.
    #[must_use]
    fn check_snake_collisions(&mut self) -> bool {
        let snakes = self.game_state.board.snakes.clone();

        for snake in &snakes {
            for other_snake in &snakes {
                for body_part in other_snake.body.iter().skip(1) {
                    if snake.head.x == body_part.x && snake.head.y == body_part.y {
                        if snake.id != self.game_state.you.id {
                            // we did not collide, so remove the collided snakes from the board

                            // remove the shorter snake
                            if snake.length > other_snake.length {
                                self.game_state
                                    .board
                                    .snakes
                                    .retain(|s| s.id != other_snake.id);
                            } else if snake.length < other_snake.length {
                                self.game_state.board.snakes.retain(|s| s.id != snake.id);
                            } else {
                                // if they are the same length, remove both
                                self.game_state
                                    .board
                                    .snakes
                                    .retain(|s| s.id != other_snake.id || s.id != snake.id);
                            }
                        } else {
                            // check if 'you' is bigger
                            if self.game_state.you.length > other_snake.length {
                                self.game_state
                                    .board
                                    .snakes
                                    .retain(|s| s.id != other_snake.id);
                            } else {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    /// Checks if any snake has eaten food.
    ///
    /// This resets the snake's health to 100, adds a new body part
    /// and removes the taken food from the board.
    fn check_eaten_food(&mut self) {
        for snake in &mut self.game_state.board.snakes {
            for food in &self.game_state.board.food.clone() {
                if snake.head.x == food.x && snake.head.y == food.y {
                    // snake ate food
                    snake.body.push(snake.body.last().unwrap().clone());
                    snake.health = 100;
                    snake.length += 1;
                    self.game_state.board.food.retain(|f| f != food);

                    // update you if id's match
                    if snake.id == self.game_state.you.id {
                        self.game_state.you = snake.clone();
                    }

                    break;
                }
            }
        }
    }

    /// Checks if any snake is out of bounds.
    ///
    /// If 'you' goes out of bounds, the function returns true and the game is over.
    /// If any other snake goes out of bounds, it is removed from the board.
    #[must_use]
    fn check_out_of_bounds(&mut self) -> bool {
        let snakes = self.game_state.board.snakes.clone();
        for snake in &snakes {
            if snake.head.x < 0
                || snake.head.x >= self.game_state.board.width
                || snake.head.y < 0
                || snake.head.y >= self.game_state.board.height
            {
                if snake.id == self.game_state.you.id {
                    return true;
                } else {
                    self.game_state.board.snakes.retain(|s| s.id != snake.id);
                }
            }
        }

        false
    }

    /// Simulates a number of turns.
    pub fn simulate_turns(&mut self, turns: u32) -> i32 {
        for _ in 0..turns {
            // generate moves for all snakes
            let mut possible_moves = HashMap::new();
            let snakes = self.game_state.board.snakes.clone();
            for snake in &snakes {
                possible_moves.insert(
                    snake.id.clone(),
                    self.get_reasonable_moves(snake.id.clone()),
                );
            }

            let mut snakes_to_remove = vec![];
            for snake in &mut self.game_state.board.snakes {
                let possible_moves_vec = possible_moves.get(&snake.id.clone()).unwrap().clone();

                if possible_moves_vec.is_empty() && snake.id == self.game_state.you.id {
                    return self.game_state.turn;
                } else if possible_moves_vec.is_empty() {
                    snakes_to_remove.push(snake.id.clone());
                    continue;
                }

                let random_index = rand::random_range(0..possible_moves_vec.len());
                let next_move = possible_moves_vec[random_index].clone();

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

            for snake_id in snakes_to_remove {
                self.game_state.board.snakes.retain(|s| s.id != snake_id);
            }

            // check for collisions
            if self.check_snake_collisions() {
                return self.game_state.turn;
            }

            // check for food
            self.check_eaten_food();

            // check for out of bounds
            if self.check_out_of_bounds() {
                return self.game_state.turn;
            }

            // decrease health with every turn
            for snake in &mut self.game_state.board.snakes {
                snake.health -= 1;

                if snake.health <= 0 {
                    return self.game_state.turn;
                }
            }

            self.game_state.turn += 1;
        }

        self.game_state.turn
    }
}

#[cfg(test)]
mod tests {
    use crate::game::game_types::{Battlesnake, Board, Game};

    use super::*;

    #[test]
    fn make_move() {
        let mut snake = Battlesnake {
            id: "abc".to_string(),
            name: "CarloConstrictor".to_string(),
            health: 75,
            body: vec![Coord::new(1, 2), Coord::new(1, 1), Coord::new(1, 0)],
            head: Coord::new(1, 2),
            length: 3,
            latency: "0".to_string(),
            shout: None,
        };

        let mut game_state = GameState {
            game: Game::new(),
            turn: 0,
            board: Board {
                width: 11,
                height: 11,
                food: vec![],
                snakes: vec![snake.clone()],
                hazards: vec![],
            },
            you: snake.clone(),
        };

        let mut simulator = Simulator::from_gamestate(&mut game_state);

        simulator.make_move(&Move::Up, snake.id.clone());
        assert_eq!(simulator.game_state.you.head, Coord::new(1, 3));
        assert_eq!(simulator.game_state.you.body[0], Coord::new(1, 3));
        assert_eq!(simulator.game_state.you.body[1], Coord::new(1, 2));
        assert_eq!(simulator.game_state.you.body[2], Coord::new(1, 1));
        assert_eq!(
            simulator.game_state.you,
            simulator.game_state.board.snakes.get(0).unwrap().clone()
        );

        simulator.make_move(&Move::Right, snake.id.clone());
        assert_eq!(simulator.game_state.you.head, Coord::new(2, 3));
        assert_eq!(simulator.game_state.you.body[0], Coord::new(2, 3));
        assert_eq!(simulator.game_state.you.body[1], Coord::new(1, 3));
        assert_eq!(simulator.game_state.you.body[2], Coord::new(1, 2));
        assert_eq!(
            simulator.game_state.you,
            simulator.game_state.board.snakes.get(0).unwrap().clone()
        );

        simulator.make_move(&Move::Down, snake.id.clone());
        assert_eq!(simulator.game_state.you.head, Coord::new(2, 2));
        assert_eq!(simulator.game_state.you.body[0], Coord::new(2, 2));
        assert_eq!(simulator.game_state.you.body[1], Coord::new(2, 3));
        assert_eq!(simulator.game_state.you.body[2], Coord::new(1, 3));
        assert_eq!(
            simulator.game_state.you,
            simulator.game_state.board.snakes.get(0).unwrap().clone()
        );

        snake.body = vec![Coord::new(1, 2), Coord::new(1, 1), Coord::new(1, 0)];
        snake.head = Coord::new(1, 2);
        game_state.you = snake.clone();
        game_state.board.snakes = vec![snake.clone()];
        simulator.game_state = game_state.clone();

        simulator.make_move(&Move::Left, snake.id.clone());
        assert_eq!(simulator.game_state.you.head, Coord::new(0, 2));
        assert_eq!(simulator.game_state.you.body[0], Coord::new(0, 2));
        assert_eq!(simulator.game_state.you.body[1], Coord::new(1, 2));
        assert_eq!(simulator.game_state.you.body[2], Coord::new(1, 1));
        assert_eq!(
            simulator.game_state.you,
            simulator.game_state.board.snakes.get(0).unwrap().clone()
        );

        let enemy_snake = Battlesnake {
            id: "def".to_string(),
            name: "EnemySnake".to_string(),
            health: 75,
            body: vec![Coord::new(5, 2), Coord::new(5, 1), Coord::new(5, 0)],
            head: Coord::new(5, 2),
            length: 3,
            latency: "0".to_string(),
            shout: None,
        };

        snake.body = vec![Coord::new(0, 2), Coord::new(0, 1), Coord::new(1, 0)];
        snake.head = Coord::new(0, 2);
        game_state.you = snake.clone();
        game_state.board.snakes = vec![snake.clone(), enemy_snake.clone()];
        simulator.game_state = game_state.clone();

        simulator.make_move(&Move::Up, enemy_snake.id.clone());

        assert_eq!(
            simulator.game_state.board.snakes.get(1).unwrap().head,
            Coord::new(5, 3)
        );
        assert_ne!(
            simulator.game_state.you,
            simulator.game_state.board.snakes.get(1).unwrap().clone()
        );
    }

    #[test]
    fn check_snake_collisions() {
        let snake = Battlesnake {
            id: "abc".to_string(),
            name: "CarloConstrictor".to_string(),
            health: 75,
            body: vec![Coord::new(0, 2), Coord::new(0, 1), Coord::new(0, 0)],
            head: Coord::new(0, 2),
            length: 3,
            latency: "0".to_string(),
            shout: None,
        };

        let enemy_snake = Battlesnake {
            id: "def".to_string(),
            name: "EnemySnake".to_string(),
            health: 75,
            body: vec![Coord::new(0, 2), Coord::new(0, 2), Coord::new(0, 2)],
            head: Coord::new(0, 2),
            length: 3,
            latency: "0".to_string(),
            shout: None,
        };

        let mut game_state = GameState {
            game: Game::new(),
            turn: 0,
            board: Board {
                width: 11,
                height: 11,
                food: vec![],
                snakes: vec![snake.clone(), enemy_snake.clone()],
                hazards: vec![],
            },
            you: snake.clone(),
        };

        let mut simulator = Simulator::from_gamestate(&mut game_state);
        assert_eq!(simulator.check_snake_collisions(), true);

        let snake = Battlesnake {
            id: "abc".to_string(),
            name: "CarloConstrictor".to_string(),
            health: 75,
            body: vec![Coord::new(0, 2), Coord::new(0, 1), Coord::new(0, 0)],
            head: Coord::new(0, 2),
            length: 3,
            latency: "0".to_string(),
            shout: None,
        };

        let enemy_snake1 = Battlesnake {
            id: "def".to_string(),
            name: "EnemySnake1".to_string(),
            health: 75,
            body: vec![Coord::new(1, 2), Coord::new(1, 1), Coord::new(1, 0)],
            head: Coord::new(1, 2),
            length: 3,
            latency: "0".to_string(),
            shout: None,
        };

        let enemy_snake2 = Battlesnake {
            id: "ghi".to_string(),
            name: "EnemySnake2".to_string(),
            health: 75,
            body: vec![Coord::new(1, 1), Coord::new(2, 1), Coord::new(2, 0)],
            head: Coord::new(1, 1),
            length: 3,
            latency: "0".to_string(),
            shout: None,
        };

        game_state.board.snakes = vec![snake.clone(), enemy_snake1.clone(), enemy_snake2.clone()];
        game_state.you = snake.clone();
        simulator.game_state = game_state.clone();

        assert_eq!(
            simulator.check_snake_collisions(),
            false,
            "you didnt collide with an enemy"
        );
        assert_eq!(
            simulator.game_state.board.snakes.len(),
            2,
            "Snakes with collisions didnt got removed"
        );
    }

    #[test]
    fn check_out_of_bounds() {
        let mut snake = Battlesnake {
            id: "abc".to_string(),
            name: "CarloConstrictor".to_string(),
            health: 75,
            body: vec![Coord::new(0, 0), Coord::new(0, 0), Coord::new(0, 0)],
            head: Coord::new(0, 0),
            length: 3,
            latency: "0".to_string(),
            shout: None,
        };

        let mut game_state = GameState {
            game: Game::new(),
            turn: 0,
            board: Board {
                width: 11,
                height: 11,
                food: vec![],
                snakes: vec![snake.clone()],
                hazards: vec![],
            },
            you: snake.clone(),
        };

        let mut simulator = Simulator::from_gamestate(&mut game_state);

        assert_eq!(simulator.check_out_of_bounds(), false);

        snake.body = vec![Coord::new(-1, 0), Coord::new(0, 0), Coord::new(0, 0)];
        snake.head = Coord::new(-1, 0);
        game_state.you = snake.clone();
        game_state.board.snakes = vec![snake.clone()];
        simulator.game_state = game_state.clone();

        assert_eq!(simulator.check_out_of_bounds(), true);

        snake.body = vec![Coord::new(5, 11), Coord::new(0, 0), Coord::new(0, 0)];
        snake.head = Coord::new(5, 11);
        game_state.you = snake.clone();
        game_state.board.snakes = vec![snake.clone()];
        simulator.game_state = game_state.clone();

        assert_eq!(simulator.check_out_of_bounds(), true);
    }

    #[test]
    fn check_eaten_food() {
        let snake = Battlesnake {
            id: "abc".to_string(),
            name: "CarloConstrictor".to_string(),
            health: 75,
            body: vec![Coord::new(0, 1), Coord::new(0, 0), Coord::new(0, 0)],
            head: Coord::new(0, 1),
            length: 3,
            latency: "0".to_string(),
            shout: None,
        };

        let mut game_state = GameState {
            game: Game::new(),
            turn: 0,
            board: Board {
                width: 11,
                height: 11,
                food: vec![Coord::new(0, 1)],
                snakes: vec![snake.clone()],
                hazards: vec![],
            },
            you: snake.clone(),
        };

        let mut simulator = Simulator::from_gamestate(&mut game_state);
        simulator.check_eaten_food();

        // check for extended body
        assert_eq!(
            simulator.game_state.you.body.len(),
            4,
            "(you) Body does not have the right length"
        );
        assert_eq!(
            simulator.game_state.board.snakes.get(0).unwrap().body.len(),
            4,
            "Body does not have the right length"
        );
        assert_eq!(
            simulator.game_state.you.length, 4,
            "(you) Snake property length is not correct"
        );
        assert_eq!(
            simulator.game_state.board.snakes.get(0).unwrap().length,
            4,
            "Snake property 'length' is not correct"
        );

        // check for resetted health
        assert_eq!(
            simulator.game_state.you.health, 100,
            "(you) Health is not correct"
        );
        assert_eq!(
            simulator.game_state.board.snakes.get(0).unwrap().health,
            100,
            "Health is not correct"
        );

        // check for removed food
        assert_eq!(
            simulator.game_state.board.food.len(),
            0,
            "Food is not removed from the board"
        );
    }

    #[test]
    fn from_gamestate() {
        let you = Battlesnake {
            id: "abc".to_string(),
            name: "CarloConstrictor".to_string(),
            health: 100,
            body: vec![Coord::new(0, 0), Coord::new(0, 0), Coord::new(0, 0)],
            head: Coord::new(0, 0),
            length: 3,
            latency: "0".to_string(),
            shout: None,
        };

        let mut game_state = GameState {
            game: Game::new(),
            turn: 0,
            board: Board {
                width: 11,
                height: 11,
                food: vec![],
                snakes: vec![you.clone()],
                hazards: vec![],
            },
            you,
        };

        let simulator = Simulator::from_gamestate(&mut game_state);
        assert_eq!(simulator.game_state.board.width, 11);
        assert_eq!(simulator.game_state.board.height, 11);
        assert_eq!(simulator.game_state.board.food.len(), 0);
        assert_eq!(simulator.game_state.board.snakes.len(), 1);
        assert_eq!(simulator.game_state.board.hazards.len(), 0);
        assert_eq!(simulator.game_state.turn, 0);
        assert_eq!(simulator.game_state.you.id, "abc");
        assert_eq!(simulator.game_state.you.name, "CarloConstrictor");
        assert_eq!(simulator.game_state.you.health, 100);
        assert_eq!(simulator.game_state.you.body.len(), 3);
    }

    #[test]
    fn get_reasonable_moves_mid() {
        let main_snake = Battlesnake {
            id: "abc".to_string(),
            name: "CarloConstrictor".to_string(),
            health: 100,
            body: vec![Coord::new(5, 5), Coord::new(5, 4), Coord::new(5, 3)],
            head: Coord::new(5, 5),
            length: 3,
            latency: "0".to_string(),
            shout: None,
        };

        let mut game_state = GameState {
            game: Game::new(),
            turn: 0,
            board: Board {
                width: 11,
                height: 11,
                food: vec![],
                snakes: vec![main_snake.clone()],
                hazards: vec![],
            },
            you: main_snake.clone(),
        };

        let mut simulator = Simulator::from_gamestate(&mut game_state);
        let moves = simulator.get_reasonable_moves("abc".to_string());
        assert_eq!(moves.len(), 3);
        assert!(moves.contains(&Move::Up));
        assert!(moves.contains(&Move::Left));
        assert!(moves.contains(&Move::Right));
    }

    #[test]
    fn get_reasonable_moves_mid_one_enemy() {
        let main_snake = Battlesnake {
            id: "abc".to_string(),
            name: "CarloConstrictor".to_string(),
            health: 100,
            body: vec![Coord::new(5, 5), Coord::new(5, 4), Coord::new(5, 3)],
            head: Coord::new(5, 5),
            length: 3,
            latency: "0".to_string(),
            shout: None,
        };

        let enemy_snake = Battlesnake {
            id: "def".to_string(),
            name: "EnemySnake".to_string(),
            health: 100,
            body: vec![Coord::new(6, 5), Coord::new(6, 4), Coord::new(6, 3)],
            head: Coord::new(6, 5),
            length: 3,
            latency: "0".to_string(),
            shout: None,
        };

        let mut game_state = GameState {
            game: Game::new(),
            turn: 0,
            board: Board {
                width: 11,
                height: 11,
                food: vec![],
                snakes: vec![main_snake.clone(), enemy_snake.clone()],
                hazards: vec![],
            },
            you: main_snake.clone(),
        };

        let mut simulator = Simulator::from_gamestate(&mut game_state);
        let moves = simulator.get_reasonable_moves("abc".to_string());
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Move::Up));
        assert!(moves.contains(&Move::Left));
    }

    #[test]
    fn get_reasonable_moves_4_corners() {
        let mut main_snake = Battlesnake {
            id: "abc".to_string(),
            name: "CarloConstrictor".to_string(),
            health: 100,
            body: vec![Coord::new(0, 0), Coord::new(0, 0), Coord::new(0, 0)],
            head: Coord::new(0, 0),
            length: 3,
            latency: "0".to_string(),
            shout: None,
        };

        // bottom-left
        let mut game_state = GameState {
            game: Game::new(),
            turn: 0,
            board: Board {
                width: 11,
                height: 11,
                food: vec![],
                snakes: vec![main_snake.clone()],
                hazards: vec![],
            },
            you: main_snake.clone(),
        };

        let mut simulator = Simulator::from_gamestate(&mut game_state);
        let moves = simulator.get_reasonable_moves("abc".to_string());
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Move::Up));
        assert!(moves.contains(&Move::Right));

        // top-left
        main_snake.head = Coord { x: 0, y: 10 };
        main_snake.body = vec![
            Coord { x: 0, y: 10 },
            Coord { x: 0, y: 10 },
            Coord { x: 0, y: 10 },
        ];

        let mut game_state = GameState {
            game: Game::new(),
            turn: 0,
            board: Board {
                width: 11,
                height: 11,
                food: vec![],
                snakes: vec![main_snake.clone()],
                hazards: vec![],
            },
            you: main_snake.clone(),
        };

        let mut simulator = Simulator::from_gamestate(&mut game_state);
        let moves = simulator.get_reasonable_moves("abc".to_string());
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Move::Right));
        assert!(moves.contains(&Move::Down));

        // top-right
        main_snake.head = Coord { x: 10, y: 10 };
        main_snake.body = vec![
            Coord { x: 10, y: 10 },
            Coord { x: 10, y: 10 },
            Coord { x: 10, y: 10 },
        ];

        let mut game_state = GameState {
            game: Game::new(),
            turn: 0,
            board: Board {
                width: 11,
                height: 11,
                food: vec![],
                snakes: vec![main_snake.clone()],
                hazards: vec![],
            },
            you: main_snake.clone(),
        };

        let mut simulator = Simulator::from_gamestate(&mut game_state);
        let moves = simulator.get_reasonable_moves("abc".to_string());
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Move::Left));
        assert!(moves.contains(&Move::Down));

        // bottom-right
        main_snake.head = Coord { x: 10, y: 0 };
        main_snake.body = vec![
            Coord { x: 10, y: 0 },
            Coord { x: 10, y: 0 },
            Coord { x: 10, y: 0 },
        ];

        let mut game_state = GameState {
            game: Game::new(),
            turn: 0,
            board: Board {
                width: 11,
                height: 11,
                food: vec![],
                snakes: vec![main_snake.clone()],
                hazards: vec![],
            },
            you: main_snake.clone(),
        };

        let mut simulator = Simulator::from_gamestate(&mut game_state);
        let moves = simulator.get_reasonable_moves("abc".to_string());
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Move::Left));
        assert!(moves.contains(&Move::Up));
    }

    #[test]
    fn get_reasonable_moves_one_enemy() {
        let main_snake = Battlesnake {
            id: "abc".to_string(),
            name: "CarloConstrictor".to_string(),
            health: 100,
            body: vec![Coord::new(0, 0), Coord::new(0, 0), Coord::new(0, 0)],
            head: Coord::new(0, 0),
            length: 3,
            latency: "0".to_string(),
            shout: None,
        };

        let enemy_snake = Battlesnake {
            id: "def".to_string(),
            name: "EnemySnake".to_string(),
            health: 100,
            body: vec![Coord::new(1, 1), Coord::new(1, 1), Coord::new(1, 1)],
            head: Coord::new(1, 1),
            length: 3,
            latency: "0".to_string(),
            shout: None,
        };

        let mut game_state = GameState {
            game: Game::new(),
            turn: 0,
            board: Board {
                width: 11,
                height: 11,
                food: vec![],
                snakes: vec![main_snake.clone(), enemy_snake.clone()],
                hazards: vec![],
            },
            you: main_snake.clone(),
        };

        let mut simulator = Simulator::from_gamestate(&mut game_state);
        let moves = simulator.get_reasonable_moves("abc".to_string());
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Move::Up));
        assert!(moves.contains(&Move::Right));
    }

    #[test]
    fn get_reasonable_moves_two_enemies() {
        let main_snake = Battlesnake {
            id: "abc".to_string(),
            name: "CarloConstrictor".to_string(),
            health: 100,
            body: vec![Coord::new(0, 0), Coord::new(0, 0), Coord::new(0, 0)],
            head: Coord::new(0, 0),
            length: 3,
            latency: "0".to_string(),
            shout: None,
        };

        let enemy_snake_1 = Battlesnake {
            id: "def".to_string(),
            name: "EnemySnake1".to_string(),
            health: 100,
            body: vec![Coord::new(1, 1), Coord::new(1, 1), Coord::new(1, 1)],
            head: Coord::new(1, 1),
            length: 3,
            latency: "0".to_string(),
            shout: None,
        };

        let enemy_snake_2 = Battlesnake {
            id: "ghi".to_string(),
            name: "EnemySnake2".to_string(),
            health: 100,
            body: vec![Coord::new(1, 0), Coord::new(1, 0), Coord::new(1, 0)],
            head: Coord::new(1, 0),
            length: 3,
            latency: "0".to_string(),
            shout: None,
        };

        let mut game_state = GameState {
            game: Game::new(),
            turn: 0,
            board: Board {
                width: 11,
                height: 11,
                food: vec![],
                snakes: vec![
                    main_snake.clone(),
                    enemy_snake_1.clone(),
                    enemy_snake_2.clone(),
                ],
                hazards: vec![],
            },
            you: main_snake.clone(),
        };

        let mut simulator = Simulator::from_gamestate(&mut game_state);
        let moves = simulator.get_reasonable_moves("abc".to_string());
        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&Move::Up));
    }
}
