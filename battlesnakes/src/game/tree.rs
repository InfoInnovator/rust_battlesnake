#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use std::{collections::HashMap, fmt::Write};

use crate::Move;

use super::game_types::GameState;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: u32,
    pub current_moves: HashMap<String, Option<Move>>,
    pub reward: f64,
    pub simulations: i32,
    pub children: Vec<Node>,
}

impl Node {
    /// Create a new node with the given id and current move.
    ///
    /// The id is only used to identify the node inside of a generated graph.
    #[must_use]
    pub fn new(id: u32, current_moves: HashMap<String, Option<Move>>) -> Node {
        Node {
            id,
            current_moves,
            reward: 0.0,
            simulations: 0,
            children: Vec::new(),
        }
    }

    /// This calculates the Upper Confidence Bound (UCB) for the node.
    #[must_use]
    pub fn get_ucb(&self, total_sims: f64) -> f64 {
        if self.simulations == 0 {
            return f64::INFINITY;
        }
        let exploitation = self.reward / f64::from(self.simulations);
        let exploration = 4.0 * (total_sims.ln() / f64::from(self.simulations)).sqrt();
        exploitation + exploration
    }

    /// Add a child node to the current node.
    pub fn add_child(&mut self, new_node: Node) {
        self.children.push(new_node);
    }

    /// This functions saves a graph representation of the tree to a file
    /// with a fixed depth of 3. The actual tree might be bigger.
    ///
    /// # Arguments
    ///
    /// * `filename` - The name of the file to save the graph to.
    pub fn save_graph(&self, filename: &str, game_state: &GameState) {
        let mut result = String::new();
        result.push_str("digraph G {\n");

        let root_moves = self
            .current_moves
            .iter()
            .map(|(k, v)| {
                format!(
                    "\n{}: {}",
                    game_state
                        .board
                        .snakes
                        .iter()
                        .find(|s| s.id == *k)
                        .unwrap()
                        .name,
                    match v {
                        Some(m) => m.to_string(),
                        None => "None".to_string(),
                    }
                )
            })
            .collect::<Vec<String>>()
            .join(", ");
        write!(
            &mut result,
            "  {} [label=\"{}\ncurrent_moves: {}\n\nreward: {}\nsimulations: {}\"];\n",
            self.id, self.id, root_moves, self.reward, self.simulations
        )
        .unwrap();

        let depth = -1;
        result.push_str(&self.export(f64::from(self.simulations), depth, game_state));

        result.push_str("}\n");

        std::fs::write(format!("{filename}.dot"), result).expect("Unable to write file");
    }

    /// This function is used to export the tree to a string in DOT format.
    ///
    /// This is called recursively for each child node. The tree can be much bigger than `depth`,
    /// but only the first `depth` levels are exported in order to keep the file usable.
    ///
    /// # Arguments
    ///
    /// * `total_sims` - The total number of simulations performed.
    /// * `depth`- The total depth of the tree to export.
    fn export(&self, total_sims: f64, depth: i32, game_state: &GameState) -> String {
        if depth == 0 {
            return String::new();
        }

        let mut result = String::new();

        for child in &self.children {
            let child_moves = child
                .current_moves
                .iter()
                .map(|(k, v)| {
                    format!(
                        "\n{}: {}",
                        game_state
                            .board
                            .snakes
                            .iter()
                            .find(|s| s.id == *k)
                            .unwrap()
                            .name,
                        match v {
                            Some(m) => m.to_string(),
                            None => "None".to_string(),
                        }
                    )
                })
                .collect::<Vec<String>>()
                .join(", ");

            write!(
                &mut result,
                "  {} [label=\"{}\ncurrent_moves: {}\n\nreward: {}\nsimulations: {}\nucb: {}\"];\n",
                child.id,
                child.id,
                child_moves,
                child.reward,
                child.simulations,
                child.get_ucb(total_sims)
            )
            .unwrap();
            writeln!(&mut result, "  {} -> {};", self.id, child.id).unwrap();
            result.push_str(&child.export(total_sims, depth - 1, game_state));
        }

        result
    }
}
