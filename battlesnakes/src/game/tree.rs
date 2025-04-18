#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use std::fmt::Write;

use crate::Move;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: u32,
    pub current_move: Option<Move>,
    pub reward: f64,
    pub simulations: i32,
    pub children: Vec<Node>,
}

impl Node {
    /// Create a new node with the given id and current move.
    ///
    /// The id is only used to identify the node inside of a generated graph.
    #[must_use]
    pub fn new(id: u32, current_move: Option<Move>) -> Node {
        Node {
            id,
            current_move,
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
    pub fn save_graph(&self, filename: &str) {
        let mut result = String::new();
        result.push_str("digraph G {\n");

        let root_move = match &self.current_move {
            Some(m) => m.to_string(),
            None => "None".to_string(),
        };
        write!(
            &mut result,
            "  {} [label=\"{}\ncurrent_move: {}\nreward: {}\nsimulations: {}\"];\n",
            self.id, self.id, root_move, self.reward, self.simulations
        )
        .unwrap();

        let depth = 3;
        result.push_str(&self.export(f64::from(self.simulations), depth));

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
    fn export(&self, total_sims: f64, depth: i32) -> String {
        if depth <= 0 {
            return String::new();
        }

        let mut result = String::new();

        for child in &self.children {
            let child_move = match &child.current_move {
                Some(m) => m.to_string(),
                None => "None".to_string(),
            };
            write!(
                &mut result,
                "  {} [label=\"{}\ncurrent_move: {}\nreward: {}\nsimulations: {}\nucb: {}\"];\n",
                child.id,
                child.id,
                child_move,
                child.reward,
                child.simulations,
                child.get_ucb(total_sims)
            )
            .unwrap();
            writeln!(&mut result, "  {} -> {};", self.id, child.id).unwrap();
            result.push_str(&child.export(total_sims, depth - 1));
        }

        result
    }
}
