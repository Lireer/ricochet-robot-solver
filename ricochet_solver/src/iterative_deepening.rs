use std::collections::hash_map::Entry;

use fnv::FnvHashMap;
use ricochet_board::{RobotPositions, Round};

use crate::{Solution, Solver, VisitedNode};

// Why it is good: https://cseweb.ucsd.edu/~elkan/130/itdeep.html
pub struct IterativeDeepening {
    /// Contains all visited robot positions and the number of steps in the shortest path found from
    /// the starting positions.
    visited_nodes: FnvHashMap<RobotPositions, VisitedNode>,
}

impl Solver for IterativeDeepening {
    fn solve(&mut self, round: &Round, start_positions: RobotPositions) -> Solution {
        // Check if the robot has already reached the target
        if round.target_reached(&start_positions) {
            return Solution::new(start_positions.clone(), start_positions, vec![]);
        }

        for i in 0.. {
            let maybe = self.depth_limited_dfs(round, start_positions.clone(), 0, i);
            if let Some(final_pos) = maybe {
                return self.backwards_path_creation(final_pos); // Solution::new(start_positions, final_pos, self.current_path.clone());
            }
            self.visited_nodes.clear();
        }
        unreachable!();
    }
}

impl IterativeDeepening {
    pub fn new() -> Self {
        Self {
            visited_nodes: FnvHashMap::with_capacity_and_hasher(65536, Default::default()),
        }
    }

    /// Performs a depth-limited DFS from the root node up to a depth of
    fn depth_limited_dfs(
        &mut self,
        round: &Round,
        start_pos: RobotPositions,
        at_step: usize,
        max_depth: usize,
    ) -> Option<RobotPositions> {
        // TODO: Try non-recursive version using a vec (Vec<(Robot, Direction); max_depth>)
        //       which always contains the current path and a vec with the positions reached
        //       in the path.

        // Return the final position if the target has been reached.
        if max_depth == 0 {
            if round.target_reached(&start_pos) {
                return Some(start_pos);
            }
            return None;
        }

        let calculating_step = at_step + 1;

        for (pos, (robot, dir)) in start_pos.reachable_positions(round.board()) {
            match self.visited_nodes.entry(pos.clone()) {
                Entry::Occupied(mut occupied) => {
                    // Ignore positions if entry has less or equal steps
                    if occupied.get().steps_to_reach() <= calculating_step {
                        continue;
                    }

                    // A shorter path has been found, insert the new path.
                    let visited = VisitedNode::new(calculating_step, start_pos.clone(), robot, dir);
                    occupied.insert(visited);
                }
                Entry::Vacant(vacant) => {
                    let visited = VisitedNode::new(calculating_step, start_pos.clone(), robot, dir);
                    vacant.insert(visited);
                }
            }

            if let Some(final_pos) =
                self.depth_limited_dfs(round, pos, calculating_step, max_depth - 1)
            {
                return Some(final_pos);
            }
        }
        None
    }

    /// Backwards path creation from the final positions to the starting position.
    fn backwards_path_creation(&self, final_position: RobotPositions) -> Solution {
        let mut path = Vec::with_capacity(32);
        let mut current_pos = final_position.clone();

        loop {
            // This should never panic since the position should be in `visited_nodes`.
            let current_info = self
                .visited_nodes
                .get(&current_pos)
                .expect("Failed to find a supposed source position");
            path.push(current_info.reached_with());
            current_pos = current_info.previous_position().clone();
            if current_info.steps_to_reach() == 1 {
                // current_pos has to be the starting position
                break;
            }
        }

        path.reverse();
        Solution::new(current_pos, final_position, path)
    }
}

impl Default for IterativeDeepening {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use ricochet_board::{template, Color, Direction, Game, RobotPositions, Round, Symbol, Target};

    use crate::{IterativeDeepening, Solution, Solver};

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
        assert_eq!(IterativeDeepening::new().solve(&round, start), expected);
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

        assert_eq!(IterativeDeepening::new().solve(&round, pos), expected);
    }
}
