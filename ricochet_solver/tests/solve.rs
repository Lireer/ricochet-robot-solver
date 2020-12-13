use chrono::prelude::*;
use itertools::Itertools;
use rand::distributions::{Distribution, Uniform};
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::convert::TryInto;
use std::fmt;

use ricochet_board::*;

fn create_board() -> (RobotPositions, Board) {
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

    let pos = RobotPositions::from_array(&[(0, 1), (5, 4), (7, 1), (7, 15)]);
    (pos, Board::from_templates(&templates))
}

#[test]
fn board_creation() {
    create_board();
}

// Test robot already on target
#[test]
fn on_target() {
    let (_, board) = create_board();

    let start = RobotPositions::from_array(&[(0, 1), (5, 4), (12, 14), (7, 15)]);
    let end = start.clone();

    assert_eq!(
        ricochet_solver::solve(&board, start, Target::Green(Symbol::Triangle)),
        (end, vec![])
    );
}

// Test short path
#[test]
fn solve() {
    let (pos, board) = create_board();

    assert_eq!(
        ricochet_solver::solve(&board, pos, Target::Yellow(Symbol::Hexagon)),
        (
            RobotPositions::from_array(&[(10, 15), (9, 11), (7, 1), (9, 12)]),
            vec![
                (Color::Red, Direction::Right),
                (Color::Red, Direction::Down),
                (Color::Red, Direction::Right),
                (Color::Blue, Direction::Right),
                (Color::Blue, Direction::Down),
                (Color::Red, Direction::Left),
                (Color::Red, Direction::Down),
                (Color::Yellow, Direction::Right),
                (Color::Yellow, Direction::Up)
            ]
        )
    );
}

#[test]
fn monte_carlo_solve() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(10);

    let (pos, board) = create_board();
    let target = Target::Red(Symbol::Triangle);

    const ROBOTS: [Color; 4] = [Color::Blue, Color::Red, Color::Green, Color::Yellow];
    const DIRECTIONS: [Direction; 4] = [
        Direction::Up,
        Direction::Right,
        Direction::Down,
        Direction::Left,
    ];

    let mut tries = 0;
    let mut total_steps: u64 = 0;
    let mut path;
    loop {
        path = Vec::new();
        let mut current_pos = pos.clone();
        tries += 1;

        loop {
            let robot = ROBOTS[rng.gen_range(0, 4)];
            let direction = DIRECTIONS[rng.gen_range(0, 4)];
            path.push((robot, direction));

            total_steps += 1;
            current_pos.move_in_direction(&board, robot, direction);
            if board.target_reached(target, &current_pos) {
                break;
            }
        }

        if path.len() <= 3 {
            break;
        }
    }
    
    assert_eq!(tries, 124);
    assert_eq!(total_steps, 49036);
    assert_eq!(
        path,
        vec![
            (Color::Red, Direction::Up),
            (Color::Red, Direction::Right),
            (Color::Red, Direction::Down)
        ]
    );
}

#[test]
#[ignore]
fn solve_many() {
    let (_, board) = create_board();

    let targets: Vec<_> = board.targets.iter().map(|(target, _)| target).collect();

    let uniform = Uniform::from(0..16);
    let rng = rand::rngs::StdRng::seed_from_u64(1);

    let n_starting_positions = 500;

    println!("Starting operations at {}", Local::now());
    println!("{}> Generating starting positions", Local::now());

    let samples = uniform
        .sample_iter(rng)
        .tuples()
        .filter(|(c, r)| !((7..=8).contains(c) && (7..=8).contains(r)))
        .take(4 * n_starting_positions)
        .batching(|it| {
            let vec = it
                .take(4)
                .collect::<Vec<(PositionEncoding, PositionEncoding)>>();
            if vec.len() < 4 {
                return None;
            }
            match vec[..4].try_into() {
                Ok(a) => Some(RobotPositions::from_array(a)),
                Err(_) => None,
            }
        })
        .cartesian_product(targets)
        .collect::<Vec<_>>();

    println!(
        "{}> Generated {} starting positions",
        Local::now(),
        n_starting_positions
    );
    println!(
        "{}> Calculating {} solutions...",
        Local::now(),
        samples.len()
    );

    let mut tests = samples
        .par_iter()
        .map(|(pos, &target)| {
            // .map(|pos| {
            PositionTest::new(
                pos.clone(),
                target,
                ricochet_solver::solve(&board, pos.clone(), target),
            )
        })
        .collect::<Vec<_>>();

    println!("{}> Finished calculations", Local::now());
    println!("{}> Sorting solutions...", Local::now());

    tests.sort_unstable_by_key(|test| (test.length, test.unique));
    tests.reverse();

    println!("{}>", Local::now());
    println!("{:#?}", &tests[..3]);
    println!("{}>", Local::now());
    println!(
        "{:?}",
        tests
            .iter()
            .filter(|t| t.unique == 4)
            .take(3)
            .collect::<Vec<_>>()
    );

    assert_eq!(
        tests[0],
        PositionTest::new(
            RobotPositions::from_array(&[(3, 2), (4, 12), (14, 0), (12, 9)]),
            Target::Yellow(Symbol::Square),
            (
                RobotPositions::from_array(&[(0, 6), (6, 7), (14, 0), (5, 5)]),
                vec![
                    (Color::Red, Direction::Down),
                    (Color::Red, Direction::Left),
                    (Color::Blue, Direction::Left),
                    (Color::Blue, Direction::Up),
                    (Color::Blue, Direction::Right),
                    (Color::Yellow, Direction::Right),
                    (Color::Yellow, Direction::Down),
                    (Color::Yellow, Direction::Left),
                    (Color::Yellow, Direction::Up),
                    (Color::Yellow, Direction::Left),
                    (Color::Yellow, Direction::Up),
                    (Color::Yellow, Direction::Right),
                    (Color::Yellow, Direction::Up),
                ]
            ),
        )
    )
}

#[derive(PartialEq)]
struct PositionTest {
    pub start_pos: RobotPositions,
    pub target: Target,
    pub solution: RobotPositions,
    pub length: usize,
    pub unique: usize,
    pub path: Vec<(Color, Direction)>,
}

impl PositionTest {
    pub fn new(
        start_pos: RobotPositions,
        target: Target,
        (solution, path): (RobotPositions, Vec<(Color, Direction)>),
    ) -> Self {
        Self {
            start_pos,
            target,
            solution,
            length: path.len(),
            unique: path.iter().map(|&(c, _)| c).unique().count(),
            path: path,
        }
    }
}

impl fmt::Debug for PositionTest {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            r#"PositionTest {{
                start_pos: {:?},
                solution:  {:?},
                target: {},
                length: {},
                unique: {},
                path: {:?},
            }}"#,
            self.start_pos, self.solution, self.target, self.length, self.unique, self.path,
        )
    }
}
