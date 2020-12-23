use fnv::FnvHashMap;
use std::collections::hash_map::Entry;

use ricochet_board::{Board, Color, Direction, RobotPositions, Target};

struct VisitInformation {
    steps_needed: usize,
    previous_position: RobotPositions,
    robot: Color,
    direction: Direction,
}

impl VisitInformation {
    pub fn new(
        steps: usize,
        previous_position: RobotPositions,
        robot: Color,
        direction: Direction,
    ) -> Self {
        VisitInformation {
            steps_needed: steps,
            previous_position,
            robot,
            direction,
        }
    }

    pub fn steps(&self) -> usize {
        self.steps_needed
    }

    pub fn previous_position(&self) -> &RobotPositions {
        &self.previous_position
    }

    pub fn from_path(&self) -> (Color, Direction) {
        (self.robot, self.direction)
    }
}

pub fn solve(
    board: &Board,
    positions: RobotPositions,
    target: Target,
) -> (RobotPositions, Vec<(Color, Direction)>) {
    // Check if the robot has already reached the target
    if board.target_reached(target, &positions) {
        return (positions, vec![]);
    }

    mem_solve(board, positions, target)
}

fn mem_solve(
    board: &Board,
    start_pos: RobotPositions,
    target: Target,
) -> (RobotPositions, Vec<(Color, Direction)>) {
    // contains all positions from which the positions in
    let mut current_step_positions: Vec<RobotPositions> = Vec::with_capacity(256);
    current_step_positions.push(start_pos.clone());
    let mut next_step_positions: Vec<RobotPositions> = Vec::with_capacity(256);

    // store information regarding the
    let mut position_store = FnvHashMap::with_capacity_and_hasher(1024, Default::default());

    // initialize the positions which will store the solution with the starting position
    let mut solution = start_pos;

    // Forward pathing to the target.
    // Computes the min. number of steps to the target and creates a tree of reachable positions
    // in `position_store`, which is used in the backwards path creation.
    'outer: for step in 0.. {
        for pos in &current_step_positions {
            if let Some(reached) = eval(
                board,
                target,
                pos,
                step,
                &mut position_store,
                &mut |pos: &RobotPositions| next_step_positions.push(pos.clone()),
            ) {
                solution = reached;
                break 'outer;
            };
        }
        current_step_positions.clear();
        std::mem::swap(&mut current_step_positions, &mut next_step_positions)
    }

    // Backwards path creation from the final positions to the starting position.
    let mut path = Vec::with_capacity(32);
    let mut current_pos = solution.clone();

    loop {
        // This should never panic since the position should be in `position_store`.
        let current_info = position_store
            .get(&current_pos)
            .expect("Failed to find a supposed source position");
        path.push(current_info.from_path());
        if current_info.steps() == 1 {
            break;
        }
        current_pos = current_info.previous_position().clone();
    }

    path.reverse();
    (solution, path)
}

const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Right,
    Direction::Left,
];

const ROBOTS: [Color; 4] = [Color::Red, Color::Blue, Color::Green, Color::Yellow];

/// Calculates all new possible positions starting from `initial_pos`.
/// `steps` is the number of steps needed to reach `initial_pos`.
/// The calculated postions are inserted into `visited_pos` and `pos_store`.
fn eval<F: FnMut(&RobotPositions)>(
    board: &Board,
    target: Target,
    initial_pos: &RobotPositions,
    steps: usize,
    pos_store: &mut FnvHashMap<RobotPositions, VisitInformation>,
    add_pos: &mut F,
) -> Option<RobotPositions> {
    for &robot in ROBOTS.iter() {
        for &dir in DIRECTIONS.iter() {
            // create a position starting from the initial position
            let mut new_pos = initial_pos.clone();
            new_pos.move_in_direction(board, robot, dir);

            let entry = pos_store.entry(new_pos.clone());
            match entry {
                // This position has already been reached
                Entry::Occupied(_) => continue,
                // First time this position has been reached
                Entry::Vacant(entry) => entry.insert(VisitInformation::new(
                    steps + 1,
                    initial_pos.clone(),
                    robot,
                    dir,
                )),
            };

            // Check if the target has been reached
            if board.target_reached(target, &new_pos) {
                return Some(new_pos);
            }

            // Add new_pos to the already visited positions
            add_pos(&new_pos);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use chrono::prelude::*;
    use itertools::Itertools;
    use rand::distributions::{Distribution, Uniform};
    use rand::{Rng, SeedableRng};
    use rayon::prelude::*;
    use ricochet_board::*;
    use std::convert::TryInto;
    use std::fmt;

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
            super::solve(&board, start, Target::Green(Symbol::Triangle)),
            (end, vec![])
        );
    }

    // Test short path
    #[test]
    fn solve() {
        let (pos, board) = create_board();

        assert_eq!(
            super::solve(&board, pos, Target::Yellow(Symbol::Hexagon)),
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
                    super::solve(&board, pos.clone(), target),
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
}
