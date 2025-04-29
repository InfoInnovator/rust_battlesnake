use battlesnakes::game::{game_types::GameState, simulator::Simulator};
use criterion::{Criterion, criterion_group, criterion_main};

fn criterion_benchmark(c: &mut Criterion) {
    let game_state = serde_json::from_str::<GameState>(
        r#"{
        "game": {
            "id": "2829de0c-d62d-4738-8dce-9818bada470c",
            "ruleset": {
                "name": "solo",
                "version": "cli",
                "settings": {
                    "foodSpawnChance": 15,
                    "hazardDamagePerTurn": 14,
                    "hazardMap": "",
                    "hazardMapAuthor": "",
                    "minimumFood": 1,
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
            "timeout": 500
        },
        "turn": 0,
        "board": {
            "height": 11,
            "width": 11,
            "food": [],
            "snakes": [
                {
                    "id": "my_snake_id",
                    "name": "my_snake",
                    "health": 100,
                    "body": [
                        {
                            "x": 5,
                            "y": 6
                        },
                        {
                            "x": 5,
                            "y": 5
                        },
                        {
                            "x": 5,
                            "y": 4
                        },
                        {
                            "x": 5,
                            "y": 3
                        }
                    ],
                    "head": {
                        "x": 5,
                        "y": 6
                    },
                    "length": 0,
                    "latency": "",
                    "shout": null
                }
            ],
            "hazards": []
        },
        "you": {
            "id": "my_snake_id",
            "name": "my_snake",
            "health": 100,
            "body": [
                {
                    "x": 5,
                    "y": 6
                },
                {
                    "x": 5,
                    "y": 5
                },
                {
                    "x": 5,
                    "y": 4
                },
                {
                    "x": 5,
                    "y": 3
                }
            ],
            "head": {
                "x": 5,
                "y": 6
            },
            "length": 0,
            "latency": "",
            "shout": null
        }
    }"#,
    )
    .unwrap();

    let mut simulator = Simulator::from_gamestate(&mut game_state.clone());
    c.bench_function("simulator 100 turns", |b| {
        b.iter(|| simulator.simulate_turns(100))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
