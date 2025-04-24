use std::{
    process::{Child, Command},
    time::Duration,
};

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
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    };
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
    let args = vec!["build", "-p", "rocket-server", "--release"];
    let _ = Command::new("cargo")
        .args(args)
        .spawn()
        .expect("Failed to start cargo")
        .wait();

    // run battlesnake
    let args = vec!["run", "-p", "rocket-server", "--release"];
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
