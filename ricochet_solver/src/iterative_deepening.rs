use ricochet_board::{RobotPositions, Round};

use crate::util::VisitedNodes;
use crate::{Solution, Solver};

// Why it is good: https://cseweb.ucsd.edu/~elkan/130/itdeep.html
// Optimizations: https://speakerdeck.com/fogleman/ricochet-robots-solver-algorithms
pub struct IterativeDeepening {
    /// Contains all visited robot positions and the number of steps in the shortest path found from
    /// the starting positions.
    visited_nodes: VisitedNodes,
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
                return self.visited_nodes.path_to(&final_pos);
            }
            self.visited_nodes.clear();
        }
        unreachable!();
    }
}

impl IterativeDeepening {
    pub fn new() -> Self {
        Self {
            visited_nodes: VisitedNodes::with_capacity(65536),
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
            if !self
                .visited_nodes
                .add_node(pos.clone(), &start_pos, calculating_step, (robot, dir))
            {
                continue;
            }

            if let Some(final_pos) =
                self.depth_limited_dfs(round, pos, calculating_step, max_depth - 1)
            {
                return Some(final_pos);
            }
        }
        None
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
