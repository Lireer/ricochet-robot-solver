use std::vec;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use ricochet_board::{template, Game, RobotPositions, Round, Symbol, Target};
use ricochet_solver::{BreadthFirst, IterativeDeepening, Solver};

fn bench_solvers(c: &mut Criterion) {
    let (pos, game) = create_board();
    let targets = vec![
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
    ];

    let mut group = c.benchmark_group("Ricochet Solver");
    for target in targets {
        let round = Round::new(
            game.board().clone(),
            target.0,
            game.get_target_position(&target.0).unwrap(),
        );
        group.bench_function(BenchmarkId::new("Breadth-First", target.1), |b| {
            b.iter(|| BreadthFirst::new().solve(&round, pos.clone()))
        });
        group.bench_function(BenchmarkId::new("IDDFS", target.1), |b| {
            b.iter(|| IterativeDeepening::new().solve(&round, pos.clone()))
        });
        // group.bench_with_input(BenchmarkId::new("Iterative", i), i, |b, i| {
        //     b.iter(|| fibonacci_fast(*i))
        // });
    }
    group.finish();
}

criterion_group!(benches, bench_solvers);
criterion_main!(benches);

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
