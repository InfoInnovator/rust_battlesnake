#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use std::path::PathBuf;

use battlesnakes::game::game_types::{Battlesnake, Board, Coord, Game, GameState, Move};
use crossterm::event::{self, KeyCode};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin},
    style::Stylize,
    widgets::{Block, Borders, Paragraph},
};
use serde::{Deserialize, Serialize};

pub struct Creator {
    cursor: (i32, i32),
    game_state: GameState,
    next_valid_moves: Vec<Move>,
    config: Config,
    description: String,
}

pub struct Config {
    field_size: i32,
    output_file: PathBuf,
}

impl Creator {
    pub fn new(field_size: i32, output_file: PathBuf, description: String) -> Self {
        let game: Game = serde_json::from_str::<Game>(
            r#"{
                "id": "2829de0c-d62d-4738-8dce-9818bada470c",
                "ruleset": {
                    "name": "solo",
                    "version": "cli",
                    "settings": {
                    "foodSpawnChance": 15,
                    "minimumFood": 1,
                    "hazardDamagePerTurn": 14,
                    "hazardMap": "",
                    "hazardMapAuthor": "",
                    "royale": {
                        "shrinkEveryNTurns": 25
                    },
                    "squad": {
                        "allowBodyCollisions": false,
                        "sharedElimination": false,
                        "sharedHealth": false,
                        "sharedLength": false
                    }
                    }
                },
                "map": "standard",
                "timeout": 500,
                "source": ""
            }"#,
        )
        .unwrap();

        let my_snake = Battlesnake {
            id: "my_snake_id".to_string(),
            name: "my_snake".to_string(),
            health: 100,
            body: Vec::new(),
            head: battlesnakes::game::game_types::Coord { x: 0, y: 0 },
            length: 0,
            latency: String::new(),
            shout: None,
        };

        let game_state = GameState {
            game,
            turn: 0,
            board: Board {
                height: field_size,
                width: field_size,
                food: Vec::new(),
                snakes: vec![my_snake.clone()],
                hazards: Vec::new(),
            },
            you: my_snake,
        };

        Self {
            cursor: (0, 0),
            game_state,
            next_valid_moves: Vec::new(),
            config: Config {
                field_size,
                output_file,
            },
            description,
        }
    }

    pub fn run(&mut self) {
        let mut terminal = ratatui::init();

        loop {
            terminal
                .draw(|frame| self.draw(frame))
                .expect("failed to draw frame");

            if event::poll(std::time::Duration::from_millis(250)).expect("failed to poll event") {
                if let event::Event::Key(key) = event::read().expect("failed to read event") {
                    match key.code {
                        KeyCode::Esc => break,
                        KeyCode::Char('q') => {
                            break;
                        }
                        KeyCode::Up => {
                            if self.cursor.1 > 0 {
                                self.cursor.1 -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if self.cursor.1 < self.config.field_size - 1 {
                                self.cursor.1 += 1;
                            }
                        }
                        KeyCode::Left => {
                            if self.cursor.0 > 0 {
                                self.cursor.0 -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if self.cursor.0 < self.config.field_size - 1 {
                                self.cursor.0 += 1;
                            }
                        }
                        KeyCode::Char('h') => {
                            self.add_head();
                        }
                        KeyCode::Char('b') => {
                            self.add_body_part();
                        }
                        KeyCode::Char('e') => {
                            self.export(&self.config.output_file);
                        }
                        KeyCode::Char('w') => {
                            self.next_valid_moves.push(Move::Up);
                        }
                        KeyCode::Char('d') => {
                            self.next_valid_moves.push(Move::Right);
                        }
                        KeyCode::Char('s') => {
                            self.next_valid_moves.push(Move::Down);
                        }
                        KeyCode::Char('a') => {
                            self.next_valid_moves.push(Move::Left);
                        }
                        _ => {}
                    }
                }
            }
        }

        ratatui::restore();
    }

    fn draw(&mut self, frame: &mut Frame) {
        let mut hor_constraints =
            vec![Constraint::Length(4); self.game_state.board.width.try_into().unwrap()];
        hor_constraints.push(Constraint::Percentage(100));

        let ver_constraints =
            vec![Constraint::Length(2); self.game_state.board.height.try_into().unwrap()];

        let outer = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(hor_constraints)
            .split(frame.area());

        let all_inner = outer
            .iter()
            .map(|o| {
                Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints(ver_constraints.clone())
                    .split(*o)
            })
            .collect::<Vec<_>>();

        // Board
        for (i, inner) in all_inner.iter().enumerate() {
            for (j, current) in inner.iter().enumerate() {
                let mut block = Block::default();

                if (j + i) % 2 == 0 {
                    block = block.on_green();
                } else {
                    block = block.on_red();
                }

                if self.cursor.0 == i32::try_from(i).unwrap()
                    && self.cursor.1 == i32::try_from(j).unwrap()
                {
                    block = block.borders(Borders::ALL);
                }

                if !self.game_state.board.snakes.is_empty()
                    && self.game_state.board.snakes[0].head.x == i32::try_from(i).unwrap()
                    && self.game_state.board.snakes[0].head.y == i32::try_from(j).unwrap()
                {
                    block = block.title("Head");
                }

                if self.game_state.board.snakes.iter().any(|s| {
                    s.body.iter().any(|b| {
                        b.x == i32::try_from(i).unwrap() && b.y == i32::try_from(j).unwrap()
                    })
                }) {
                    block = block.title("Body");
                }

                frame.render_widget(block, *current);
            }
        }

        frame.render_widget(
            Block::new().on_black(),
            outer[usize::try_from(self.game_state.board.width).unwrap()],
        );

        let snake_info_layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Fill(1)])
            .split(outer[usize::try_from(self.game_state.board.width).unwrap()]);

        if !self.game_state.board.snakes.is_empty() {
            // title
            let snake_text = format!("Snake {}", self.game_state.board.snakes[0].name);
            let block = Block::default().title(snake_text).borders(Borders::ALL);
            frame.render_widget(block, snake_info_layout[0]);

            // snake body
            let snake_body_layout = Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints(vec![
                    Constraint::Length(1);
                    self.game_state.board.snakes[0].body.len()
                ])
                .split(snake_info_layout[0].inner(Margin::new(1, 1)));

            for (j, body_part) in self.game_state.board.snakes[0].body.iter().enumerate() {
                let mut body_text = format!("{body_part:?}");

                if body_part.x == self.game_state.board.snakes[0].head.x
                    && body_part.y == self.game_state.board.snakes[0].head.y
                {
                    body_text = format!("{body_text} (Head)");
                }

                let body_block = Paragraph::new(body_text);

                frame.render_widget(body_block, snake_body_layout[j]);
            }
        }
    }

    fn export(&self, output_file: &PathBuf) {
        let mut game_state_wo_snakes = self.game_state.clone();
        game_state_wo_snakes.board.snakes.clear();

        let export_state = ExportedGameState {
            description: self.description.clone(),
            game_state: game_state_wo_snakes,
            next_valid_moves: self.next_valid_moves.clone(),
        };
        let game_state_json = serde_json::to_string(&export_state).unwrap();

        std::fs::write(output_file, game_state_json).unwrap();
    }

    /// Adds a head to the snake at the current cursor position
    ///
    /// This function adds a head to the board at the current cursor position.
    /// The head is also added to the snake's body.
    /// It removes all body parts that existed before.
    fn add_head(&mut self) {
        // set head position
        self.game_state.board.snakes.get_mut(0).unwrap().head = Coord {
            x: self.cursor.0,
            y: self.cursor.1,
        };

        // clear all previous body parts
        self.game_state
            .board
            .snakes
            .get_mut(0)
            .unwrap()
            .body
            .clear();

        // set body position
        self.game_state
            .board
            .snakes
            .get_mut(0)
            .unwrap()
            .body
            .push(Coord {
                x: self.cursor.0,
                y: self.cursor.1,
            });
    }

    /// Adds a body part to the snake at the current cursor position
    ///
    /// The body parts need to be added from head to tail.
    fn add_body_part(&mut self) {
        self.game_state
            .board
            .snakes
            .get_mut(0)
            .unwrap()
            .body
            .push(Coord {
                x: self.cursor.0,
                y: self.cursor.1,
            });
    }
}

#[derive(Serialize, Deserialize)]
pub struct ExportedGameState {
    pub description: String,
    pub game_state: GameState,
    pub next_valid_moves: Vec<Move>,
}
