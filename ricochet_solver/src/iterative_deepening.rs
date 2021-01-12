use ricochet_board::{RobotPositions, Round};

use crate::util::{BasicVisitedNode, LeastMovesBoard, VisitedNodes};
use crate::{Solution, Solver};

// Why it's good: https://cseweb.ucsd.edu/~elkan/130/itdeep.html
// Optimizations: https://speakerdeck.com/fogleman/ricochet-robots-solver-algorithms
#[derive(Debug)]
pub struct IterativeDeepening {
    /// Contains all visited robot positions and the number of moves in the shortest path found from
    /// the starting positions.
    visited_nodes: VisitedNodes<BasicVisitedNode>,
    /// This board contains the minimum number of moves to reach the target for each field.
    ///
    /// This minimum is a lower bound and may be impossible to reach even if all other robots are
    /// positioned perfectly.
    move_board: LeastMovesBoard,
}

impl Solver for IterativeDeepening {
    fn solve(&mut self, round: &Round, start_positions: RobotPositions) -> Solution {
        // Check if the robot has already reached the target
        if round.target_reached(&start_positions) {
            return Solution::new(start_positions.clone(), start_positions, vec![]);
        }

        self.move_board = LeastMovesBoard::new(round.board(), round.target_position());
        let start = self.move_board.min_moves(&start_positions, round.target());

        if self
            .move_board
            .is_unsolvable(&start_positions, round.target())
        {
            panic!("It's not possible to reach the target starting from this robot configuration");
        }

        for i in start.. {
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
            move_board: Default::default(),
        }
    }

    /// Performs a depth-limited DFS from `start_pos` up to a depth of `max_depth`.
    ///
    /// `at_move` is the number of moves needed to reach `start_pos` in the context of IDDFS.
    fn depth_limited_dfs(
        &mut self,
        round: &Round,
        start_pos: RobotPositions,
        at_move: usize,
        max_depth: usize,
    ) -> Option<RobotPositions> {
        // Return the final position if the target has been reached.
        if max_depth == 0 {
            if round.target_reached(&start_pos) {
                return Some(start_pos);
            }
            return None;
        }

        let calculating_move = at_move + 1;

        for (pos, (robot, dir)) in start_pos.reachable_positions(round.board()) {
            // Ignore the new positions if the target can't be reached within the limit of
            // max_depth - 1 moves.
            if max_depth - 1 < self.move_board.min_moves(&pos, round.target()) {
                continue;
            }

            if !self.visited_nodes.add_node(
                pos.clone(),
                &start_pos,
                calculating_move,
                (robot, dir),
                BasicVisitedNode::new,
            ) {
                continue;
            }

            if let Some(final_pos) =
                self.depth_limited_dfs(round, pos, calculating_move, max_depth - 1)
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
