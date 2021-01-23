use std::vec;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use ricochet_board::{template, Game, Robot, RobotPositions, Round, Symbol, Target};
use ricochet_solver::util::LeastMovesBoard;
use ricochet_solver::{AStar, BreadthFirst, IdaStar, Solver};

fn bench_solvers(c: &mut Criterion) {
    let (pos, bench_data) = solver_bench_setup();

    let mut group = c.benchmark_group("Ricochet Solver");
    for (round, moves) in bench_data {
        group.bench_function(BenchmarkId::new("Breadth-First", moves), |b| {
            b.iter(|| BreadthFirst::new().solve(&round, pos.clone()))
        });
        group.bench_function(BenchmarkId::new("IDA*", moves), |b| {
            b.iter(|| IdaStar::new().solve(&round, pos.clone()))
        });
        group.bench_function(BenchmarkId::new("A*", moves), |b| {
            b.iter(|| AStar::new().solve(&round, pos.clone()))
        });
    }
    group.finish();
}

fn bench_util(c: &mut Criterion) {
    let (pos, game) = create_board();
    let target_position = pos[Robot::Red];

    let mut group = c.benchmark_group("Ricochet Solver Utils");
    group.bench_function(BenchmarkId::new("LeastMovesBoard", ""), |b| {
        b.iter(|| LeastMovesBoard::new(game.board(), target_position))
    });

    group.finish();
}

/// Needs more than 20 minutes on a Ryzen 3600
fn bench_22_move_problem(c: &mut Criterion) {
    let (pos, round) = create_22_move_problem();

    let mut group = c.benchmark_group("22 move problem");
    group.sample_size(10);
    group.bench_function(BenchmarkId::new("A*", 22), |b| {
        b.iter(|| AStar::new().solve(&round, pos.clone()))
    });
    group.bench_function(BenchmarkId::new("IDA*", 22), |b| {
        b.iter(|| IdaStar::new().solve(&round, pos.clone()))
    });
    group.bench_function(BenchmarkId::new("Breadth-First", 22), |b| {
        b.iter(|| BreadthFirst::new().solve(&round, pos.clone()))
    });

    group.finish();
}

criterion_group!(benches, bench_solvers, bench_util, bench_22_move_problem);
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

fn create_22_move_problem() -> (RobotPositions, Round) {
    const ORIENTATIONS: [template::Orientation; 4] = [
        template::Orientation::UpperLeft,
        template::Orientation::UpperRight,
        template::Orientation::BottomRight,
        template::Orientation::BottomLeft,
    ];

    let templates = template::gen_templates();
    let templates = [
        templates[11].clone(),
        templates[1].clone(),
        templates[5].clone(),
        templates[7].clone(),
    ]
    .iter()
    .cloned()
    .enumerate()
    .map(|(i, mut temp)| {
        temp.rotate_to(ORIENTATIONS[i]);
        temp
    })
    .collect::<Vec<template::BoardTemplate>>();

    let pos = RobotPositions::from_tuples(&[(15, 6), (14, 0), (13, 0), (0, 14)]);
    let target = Target::Blue(Symbol::Triangle);
    let game = Game::from_templates(&templates);
    let round = Round::new(
        game.board().clone(),
        target,
        game.get_target_position(&target).unwrap(),
    );
    (pos, round)
}
