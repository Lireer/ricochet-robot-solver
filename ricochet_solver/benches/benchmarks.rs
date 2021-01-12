use std::vec;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use ricochet_board::{template, Color, Game, RobotPositions, Round, Symbol, Target};
use ricochet_solver::util::LeastMovesBoard;
use ricochet_solver::{BreadthFirst, IterativeDeepening, Solver};

fn bench_bfs(c: &mut Criterion) {
    let (pos, bench_data) = solver_bench_setup();

    let mut group = c.benchmark_group("Ricochet Solver");
    for (round, moves) in bench_data {
        group.bench_function(BenchmarkId::new("Breadth-First", moves), |b| {
            b.iter(|| BreadthFirst::new().solve(&round, pos.clone()))
        });
        group.bench_function(BenchmarkId::new("IDDFS", moves), |b| {
            b.iter(|| IterativeDeepening::new().solve(&round, pos.clone()))
        });
    }
    group.finish();
}

fn bench_util(c: &mut Criterion) {
    let (pos, game) = create_board();
    let target_position = pos[Color::Red];

    let mut group = c.benchmark_group("Ricochet Solver Utils");
    group.bench_function(BenchmarkId::new("LeastMovesBoard", ""), |b| {
        b.iter(|| LeastMovesBoard::new(game.board(), target_position))
    });

    group.finish();
}

criterion_group!(benches, bench_bfs, bench_util);
criterion_main!(benches);

fn solver_bench_setup() -> (RobotPositions, Vec<(Round, usize)>) {
    let (pos, game) = create_board();

    let data = vec![
        (Target::Blue(Symbol::Triangle), 2),
        (Target::Yellow(Symbol::Circle), 3),
        (Target::Red(Symbol::Triangle), 4),
        (Target::Red(Symbol::Hexagon), 5),
        (Target::Spiral, 6),
        (Target::Green(Symbol::Triangle), 7),
        (Target::Red(Symbol::Square), 8),
        (Target::Green(Symbol::Hexagon), 9),
        (Target::Yellow(Symbol::Hexagon), 11),
        (Target::Yellow(Symbol::Triangle), 12),
        (Target::Yellow(Symbol::Square), 13),
    ]
    .iter_mut()
    .map(|(target, moves)| {
        let round = Round::new(
            game.board().clone(),
            *target,
            game.get_target_position(&target).unwrap(),
        );
        (round, *moves)
    })
    .collect();

    (pos, data)
}

fn create_board() -> (RobotPositions, Game) {
    const ORIENTATIONS: [template::Orientation; 4] = [
        template::Orientation::UpperLeft,
        template::Orientation::UpperRight,
        template::Orientation::BottomRight,
        template::Orientation::BottomLeft,
    ];

    let templates = template::gen_templates()
        .iter()
        .step_by(3)
        .cloned()
        .enumerate()
        .map(|(i, mut temp)| {
            temp.rotate_to(ORIENTATIONS[i]);
            temp
        })
        .collect::<Vec<template::BoardTemplate>>();

    let pos = RobotPositions::from_tuples(&[(15, 15), (15, 0), (0, 15), (0, 0)]);
    (pos, Game::from_templates(&templates))
}
