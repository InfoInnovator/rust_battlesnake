#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use std::fmt::Write;

use crate::Move;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: i32,
    pub current_move: Option<Move>,
    pub wins: i32,
    pub simulations: i32,
    pub children: Vec<Node>,
}

impl Node {
    pub fn new(id: i32, current_move: Option<Move>) -> Node {
        Node {
            id,
            current_move,
            wins: 0,
            simulations: 0,
            children: Vec::new(),
        }
    }

    pub fn get_ucb(&self, total_sims: i32) -> f64 {
        if self.simulations == 0 {
            return f64::INFINITY;
        }
        let exploitation = f64::from(self.wins) / f64::from(self.simulations);
        let exploration = (1.3 * f64::from(total_sims).ln() / f64::from(self.simulations)).sqrt();
        exploitation + exploration
    }

    pub fn add_child(&mut self, new_node: Node) -> Node {
        self.children.push(new_node);
        self.clone()
    }

    pub fn save_graph(&self, _filename: &str) {
        let mut result = String::new();
        result.push_str("digraph G {\n");

        let root_move = match &self.current_move {
            Some(m) => m.to_string(),
            None => "None".to_string(),
        };
        write!(
            &mut result,
            "  {} [label=\"{}\ncurrent_move: {}\nwins: {}\nsimulations: {}\"];\n",
            self.id, self.id, root_move, self.wins, self.simulations
        )
        .unwrap();
        result.push_str(&self.export());

        result.push_str("}\n");

        // std::fs::write(format!("{}.dot", filename), result).expect("Unable to write file");
        // std::process::Command::new("dot")
        //     .arg("-Tpdf")
        //     .arg(format!("{}.dot", filename))
        //     .arg("-o")
        //     .arg(format!("{}.pdf", filename))
        //     .output()
        //     .expect("failed to execute process");
    }

    fn export(&self) -> String {
        let mut result = String::new();

        for child in &self.children {
            let child_move = match &child.current_move {
                Some(m) => m.to_string(),
                None => "None".to_string(),
            };
            write!(
                &mut result,
                "  {} [label=\"{}\ncurrent_move: {}\nwins: {}\nsimulations: {}\nucb: {}\"];\n",
                child.id,
                child.id,
                child_move,
                child.wins,
                child.simulations,
                child.get_ucb(self.simulations)
            )
            .unwrap();
            writeln!(&mut result, "  {} -> {};", self.id, child.id).unwrap();
            result.push_str(&child.export());
        }

        result
    }
}
