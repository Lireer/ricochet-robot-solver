use ricochet_board::{RobotPositions, Round};

use crate::util::VisitedNodes;
use crate::{Solution, Solver};

/// Finds an optimal solution by visiting all possible game states in order of moves needed to
/// reach them.
#[derive(Debug, Clone)]
pub struct BreadthFirst {
    /// Manages knowledge of visited nodes.
    visited_nodes: VisitedNodes,
}

impl Solver for BreadthFirst {
    fn solve(&mut self, round: &Round, start_positions: RobotPositions) -> Solution {
        // Check if the robot has already reached the target
        if round.target_reached(&start_positions) {
            return Solution::new(start_positions.clone(), start_positions, vec![]);
        }

        self.start(round, start_positions)
    }
}

impl BreadthFirst {
    /// Create a new solver which uses a breadth first search to find an optimal solution.
    pub fn new() -> Self {
        Self {
            visited_nodes: VisitedNodes::with_capacity(65536),
        }
    }

    fn start(&mut self, round: &Round, start_pos: RobotPositions) -> Solution {
        // contains all positions from which the positions in
        let mut current_move_positions: Vec<RobotPositions> = Vec::with_capacity(16usize.pow(3));
        current_move_positions.push(start_pos.clone());
        let mut next_move_positions: Vec<RobotPositions> = Vec::with_capacity(16usize.pow(4));

        // initialize the positions which will store the solution with the starting position
        let mut solution = start_pos;

        // Forward pathing to the target.
        // Computes the min. number of moves to the target and creates a tree of reachable positions
        // in `visited_nodes`, which is later used in the path creation.
        'outer: for move_n in 0.. {
            for pos in &current_move_positions {
                if let Some(reached) =
                    self.eval_robot_state(round, pos, move_n, &mut next_move_positions)
                {
                    solution = reached;
                    break 'outer;
                };
            }
            current_move_positions.clear();
            std::mem::swap(&mut current_move_positions, &mut next_move_positions)
        }

        self.visited_nodes.path_to(&solution)
    }

    /// Calculates all unseen reachable positions starting from `initial_pos` and adds them to
    /// `self.visited_nodes`.
    ///
    /// `moves` is the number of moves needed to reach `initial_pos`.
    /// The calculated postions are inserted into `pos_store`.
    fn eval_robot_state(
        &mut self,
        round: &Round,
        initial_pos: &RobotPositions,
        moves: usize,
        next_positions: &mut Vec<RobotPositions>,
    ) -> Option<RobotPositions> {
        for (new_pos, (robot, dir)) in initial_pos.reachable_positions(round.board()) {
            // Mark the new positions as visited and continue with the next one, if a better path
            // already exists.
            if !self
                .visited_nodes
                .add_node(new_pos.clone(), &initial_pos, moves + 1, (robot, dir))
            {
                continue;
            }

            // Check if the target has been reached.
            if round.target_reached(&new_pos) {
                return Some(new_pos);
            }

            // Add new_pos to the positions to be checked
            next_positions.push(new_pos);
        }

        None
    }
}

impl Default for BreadthFirst {
    fn default() -> Self {
        Self::new()
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

        let mut tries = 0;
        let mut total_moves: u64 = 0;
        let mut path;
        loop {
            path = Vec::new();
            let mut current_pos = pos.clone();
            tries += 1;

            loop {
                let robot = ROBOTS[rng.gen_range(0..4)];
                let direction = DIRECTIONS[rng.gen_range(0..4)];
                let new_pos =
                    current_pos
                        .clone()
                        .move_in_direction(&round.board(), robot, direction);
                if new_pos == current_pos {
                    continue;
                }
                current_pos = new_pos;
                path.push((robot, direction));

                total_moves += 1;
                if round.target_reached(&current_pos) {
                    break;
                }
            }

            if path.len() <= 3 {
                break;
            }
        }

        assert_eq!(tries, 2781);
        assert_eq!(total_moves, 596132);
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
                RobotPositions::from_tuples(&[(1, 10), (4, 1), (3, 15), (13, 2)]),
                Target::Yellow(Symbol::Hexagon),
                RobotPositions::from_tuples(&[(14, 11), (13, 11), (3, 15), (9, 12)]),
                vec![
                    (Color::Red, Direction::Up),
                    (Color::Red, Direction::Right),
                    (Color::Red, Direction::Down),
                    (Color::Red, Direction::Right),
                    (Color::Blue, Direction::Down),
                    (Color::Blue, Direction::Right),
                    (Color::Yellow, Direction::Right),
                    (Color::Yellow, Direction::Down),
                    (Color::Yellow, Direction::Left),
                    (Color::Yellow, Direction::Down),
                    (Color::Yellow, Direction::Left),
                    (Color::Yellow, Direction::Down),
                    (Color::Yellow, Direction::Right),
                    (Color::Yellow, Direction::Up),
                    (Color::Yellow, Direction::Left),
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
