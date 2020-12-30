use fnv::FnvHashMap;
use getset::{CopyGetters, Getters};
use ricochet_board::{Color, Direction, RobotPositions, Round};
use std::collections::hash_map::Entry;
use std::todo;

use crate::{Solution, Solver};

/// Finds an optimal solution by visiting all possible game states in order of steps needed to
/// reach them.
#[derive(Debug, Clone)]
pub struct BreadthFirst {
    // /// Contains all visited nodes.
    // ///
    // /// All nodes in the vec at index `i` of the outer vec are reachable in `i` steps.
    // visited_nodes: Vec<Vec<VisitedNode>>,
    /// Contains all visited nodes as a set to make finding one known node easier.
    visited_nodes: FnvHashMap<RobotPositions, VisitedNode>,
}

#[derive(Debug, Clone, CopyGetters, Getters)]
struct VisitedNode {
    #[getset(get_copy = "pub")]
    steps_to_reach: usize,
    #[getset(get = "pub")]
    previous_position: RobotPositions,
    robot: Color,
    direction: Direction,
}

impl VisitedNode {
    pub fn new(
        steps: usize,
        previous_position: RobotPositions,
        robot: Color,
        direction: Direction,
    ) -> Self {
        VisitedNode {
            steps_to_reach: steps,
            previous_position,
            robot,
            direction,
        }
    }

    pub fn reach_with(&self) -> (Color, Direction) {
        (self.robot, self.direction)
    }
}

impl Solver for BreadthFirst {
    fn solve(&mut self, round: &Round, start_positions: RobotPositions) -> Solution {
        // Check if the robot has already reached the target
        if round.target_reached(&start_positions) {
            return Solution::new(start_positions.clone(), start_positions, vec![]);
        }

        self.mem_solve(round, start_positions)
    }
}

impl BreadthFirst {
    /// Create a new solver which uses a breadth first search to find an optimal solution.
    pub fn new() -> Self {
        Self {
            visited_nodes: Default::default(),
            // visited_nodes_set: Default::default(),
        }
    }

    /// Evaluates all postitions reachable in one step starting from `positions`.
    ///
    /// `eval_step` is the number of steps needed to reach these new positions, which is the number
    /// of steps needed to reach `positions` plus one.
    fn evaluate_state(&mut self, positions: &RobotPositions, eval_step: usize) {
        todo!()
    }

    fn mem_solve(&mut self, round: &Round, start_pos: RobotPositions) -> Solution {
        // contains all positions from which the positions in
        let mut current_step_positions: Vec<RobotPositions> = Vec::with_capacity(256);
        current_step_positions.push(start_pos.clone());
        let mut next_step_positions: Vec<RobotPositions> = Vec::with_capacity(1024);

        // initialize the positions which will store the solution with the starting position
        let mut solution = start_pos.clone();

        // Forward pathing to the target.
        // Computes the min. number of steps to the target and creates a tree of reachable positions
        // in `visited_nodes`, which is later used in the backwards path creation.
        'outer: for step in 0.. {
            for pos in &current_step_positions {
                if let Some(reached) = self.eval(round, pos, step, &mut |pos: &RobotPositions| {
                    next_step_positions.push(pos.clone())
                }) {
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
            // This should never panic since the position should be in `visited_nodes`.
            let current_info = self
                .visited_nodes
                .get(&current_pos)
                .expect("Failed to find a supposed source position");
            path.push(current_info.reach_with());
            if current_info.steps_to_reach() == 1 {
                break;
            }
            current_pos = current_info.previous_position().clone();
        }

        path.reverse();
        Solution::new(start_pos, solution, path)
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
    /// The calculated postions are inserted into `pos_store`.
    fn eval<F: FnMut(&RobotPositions)>(
        &mut self,
        round: &Round,
        initial_pos: &RobotPositions,
        steps: usize,
        add_pos: &mut F,
    ) -> Option<RobotPositions> {
        for &robot in Self::ROBOTS.iter() {
            for &dir in Self::DIRECTIONS.iter() {
                // create a position starting from the initial position
                let new_pos = initial_pos
                    .clone()
                    .move_in_direction(round.board(), robot, dir);

                // if nothing changed, do nothing
                if *initial_pos == new_pos {
                    continue;
                }

                let entry = self.visited_nodes.entry(new_pos.clone());
                match entry {
                    // This position has already been reached
                    Entry::Occupied(_) => continue,
                    // First time this position has been reached
                    Entry::Vacant(entry) => {
                        entry.insert(VisitedNode::new(steps + 1, initial_pos.clone(), robot, dir))
                    }
                };

                // Check if the target has been reached
                if round.target_reached(&new_pos) {
                    return Some(new_pos);
                }

                // Add new_pos to the already visited positions
                add_pos(&new_pos);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::BreadthFirst;
    use crate::{Solution, Solver};
    use chrono::prelude::*;
    use itertools::Itertools;
    use rand::distributions::{Distribution, Uniform};
    use rand::{Rng, SeedableRng};
    use rayon::prelude::*;
    use ricochet_board::*;
    use std::convert::TryInto;
    use std::{fmt, vec};

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

        let pos = RobotPositions::from_tuples(&[(0, 1), (5, 4), (7, 1), (7, 15)]);
        (pos, Game::from_templates(&templates))
    }

    #[test]
    fn board_creation() {
        create_board();
    }

    // Test robot already on target
    #[test]
    fn on_target() {
        let (_, game) = create_board();
        let target = Target::Green(Symbol::Triangle);
        let target_position = game.get_target_position(&target).unwrap();

        let start = RobotPositions::from_tuples(&[(0, 1), (5, 4), target_position.into(), (7, 15)]);
        let end = start.clone();

        let round = Round::new(game.board().clone(), target, target_position);

        let expected = Solution::new(start.clone(), end, vec![]);
        assert_eq!(BreadthFirst::new().solve(&round, start), expected);
    }

    // Test short path
    #[test]
    fn solve() {
        let (pos, game) = create_board();
        let target = Target::Yellow(Symbol::Hexagon);

        let round = Round::new(
            game.board().clone(),
            target,
            game.get_target_position(&target).unwrap(),
        );

        let expected = Solution::new(
            pos.clone(),
            RobotPositions::from_tuples(&[(10, 15), (9, 11), (7, 1), (9, 12)]),
            vec![
                (Color::Red, Direction::Right),
                (Color::Red, Direction::Down),
                (Color::Red, Direction::Right),
                (Color::Blue, Direction::Right),
                (Color::Blue, Direction::Down),
                (Color::Red, Direction::Left),
                (Color::Red, Direction::Down),
                (Color::Yellow, Direction::Right),
                (Color::Yellow, Direction::Up),
            ],
        );

        assert_eq!(BreadthFirst::new().solve(&round, pos), expected);
    }

    #[test]
    fn monte_carlo_solve() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(10);

        let (pos, game) = create_board();
        let target = Target::Red(Symbol::Triangle);
        let round = Round::new(
            game.board().clone(),
            target,
            game.get_target_position(&target).unwrap(),
        );

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
                let robot = ROBOTS[rng.gen_range(0..4)];
                let direction = DIRECTIONS[rng.gen_range(0..4)];
                path.push((robot, direction));

                total_steps += 1;
                current_pos = current_pos.move_in_direction(&round.board(), robot, direction);
                if round.target_reached(&current_pos) {
                    break;
                }
            }

            if path.len() <= 3 {
                break;
            }
        }

        assert_eq!(tries, 9179);
        assert_eq!(total_steps, 3689543);
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
        let (_, game) = create_board();

        let targets: Vec<_> = game.targets().iter().map(|(target, _)| target).collect();

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
                    Ok(a) => Some(RobotPositions::from_tuples(a)),
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
                let target_position = game.get_target_position(&target).expect("unknown target");
                let round = Round::new(game.board().clone(), target, target_position);
                let solution = BreadthFirst::new().solve(&round, pos.clone());
                PositionTest::new(pos.clone(), target, solution.end_pos, solution.path)
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
                RobotPositions::from_tuples(&[(3, 2), (4, 12), (14, 0), (12, 9)]),
                Target::Yellow(Symbol::Square),
                RobotPositions::from_tuples(&[(0, 6), (6, 7), (14, 0), (5, 5)]),
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
            solution: RobotPositions,
            path: Vec<(Color, Direction)>,
        ) -> Self {
            Self {
                start_pos,
                target,
                solution,
                length: path.len(),
                unique: path.iter().map(|&(c, _)| c).unique().count(),
                path,
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