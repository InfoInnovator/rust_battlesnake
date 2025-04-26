use std::{
    collections::VecDeque,
    io::Write,
    process::{Child, Command},
    time::Duration,
};

use battlesnakes::game::game_types::GameState;
use runa::creator::simple_creator::ExportedGameState;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    if args.len() < 2 {
        eprintln!("Usage: {} <command>", args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "solo" => {
            let snake_name = if args.len() > 2 {
                args[2].clone()
            } else {
                println!("No snake name provided, using default: 'CarloConstrictor'");
                "CarloConstrictor".to_string()
            };

            start_snake_solo(snake_name);
        }
        "export" => {
            let filename = if args.len() > 2 {
                args[2].clone()
            } else {
                eprintln!("No filename provided");
                std::process::exit(1);
            };

            export_graph_to_pdf(filename);
        }
        "import" => {
            let turn = if args.len() > 2 {
                args[2].clone()
            } else {
                eprintln!("No turn provided");
                std::process::exit(1);
            };

            export_game_state(turn);
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    };
}

fn export_game_state(turn: String) {
    // read content from file
    let filename = "game_state.json";
    let file = std::fs::File::open(filename).expect("Unable to open file");
    let reader = std::io::BufReader::new(file);
    let states: VecDeque<GameState> =
        serde_json::from_reader(reader).expect("Unable to parse JSON");

    // find the state for the given turn
    let state = states
        .iter()
        .find(|state| state.turn == turn.parse::<i32>().unwrap())
        .expect("Unable to find state for given turn");

    let eported_game_state = ExportedGameState {
        description: "".to_string(),
        game_state: state.clone(),
        next_valid_moves: vec![],
    };

    // write the game state to a file
    let file = std::fs::File::create("exported_game_state.json").expect("Unable to create file");
    let mut writer = std::io::BufWriter::new(file);
    let json = serde_json::to_string(&eported_game_state).expect("Unable to serialize");
    writer
        .write_all(json.as_bytes())
        .expect("Unable to write data");
}

fn export_graph_to_pdf(filename: String) {
    let args = vec!["-Tpdf", &filename, "-o", "out.pdf"];
    let mut dot_cmd = Command::new("dot")
        .args(args)
        .spawn()
        .expect("Failed to start dot");

    dot_cmd.wait().expect("Failed to wait on dot");

    let args = vec!["out.pdf"];
    let _ = Command::new("firefox")
        .args(args)
        .spawn()
        .expect("Failed to open PDF")
        .wait();
}

fn start_snake_solo(snake_name: String) {
    let mut children: Vec<Child> = vec![];

    // build battlesnake and wait for it to finish
    let args = vec!["build", "-p", "rocket-server"];
    let _ = Command::new("cargo")
        .args(args)
        .spawn()
        .expect("Failed to start cargo")
        .wait();

    // run battlesnake
    let args = vec!["run", "-p", "rocket-server"];
    let snake_cmd = Command::new("cargo")
        .args(args)
        .spawn()
        .expect("Failed to start cargo");
    children.push(snake_cmd);

    // wait for the server to start
    std::thread::sleep(Duration::from_secs(1));

    // start battlesnake cli
    let args = vec![
        "play",
        "-W",
        "11",
        "-H",
        "11",
        "--name",
        &snake_name,
        "--url",
        "http://localhost:8000",
        "-g",
        "solo",
        "--browser",
    ];
    let cli_cmd = Command::new("./battlesnake")
        .current_dir("/home/malte/go/bin")
        .args(args)
        .spawn()
        .expect("Failed to start battlesnake cli");
    children.push(cli_cmd);

    // wait for both commands to finish
    for mut child in children {
        let _ = child.wait().expect("Failed to wait on child");
    }
}
