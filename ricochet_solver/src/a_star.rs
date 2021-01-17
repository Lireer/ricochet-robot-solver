use fxhash::FxBuildHasher;
use priority_queue::PriorityQueue;
use ricochet_board::{RobotPositions, Round};
use std::cmp::Reverse;
use std::usize;

use crate::util::{BasicVisitedNode, LeastMovesBoard, VisitedNodes};
use crate::{Path, Solver};

#[derive(Debug)]
pub struct AStar {
    visited_nodes: VisitedNodes<BasicVisitedNode>,
    move_board: LeastMovesBoard,
}

impl AStar {
    pub fn new() -> Self {
        Self {
            visited_nodes: VisitedNodes::with_capacity(65536),
            move_board: Default::default(),
        }
    }
}

impl Solver for AStar {
    fn solve(&mut self, round: &Round, start_positions: RobotPositions) -> Path {
        // Check if the target has already been reached.
        if round.target_reached(&start_positions) {
            return Path::new_start_on_target(start_positions);
        }

        // Check if the problem may be impossible to solve.
        self.move_board = LeastMovesBoard::new(round.board(), round.target_position());
        if self
            .move_board
            .is_unsolvable(&start_positions, round.target())
        {
            panic!("It's not possible to reach the target starting from this robot configuration");
        }

        // Use the least moves board as an admissable heuristic (never overestimates the moves needed).
        let move_board = &self.move_board;
        let moves_to_target = |pos: &RobotPositions| move_board.min_moves(pos, round.target());

        // Create a queue holding the not yet expanded nodes.
        let mut open_list =
            PriorityQueue::<RobotPositions, MoveCounter, FxBuildHasher>::with_capacity_and_hasher(
                65536,
                Default::default(),
            );

        // Add starting positions to the open list.
        open_list.push(
            start_positions.clone(),
            MoveCounter::new(0, moves_to_target(&start_positions)),
        );

        let mut found_minimum = usize::MAX;
        let mut found_final_position = start_positions;

        // Expand the search tree.
        while let Some((from_pos, prio)) = open_list.pop() {
            if prio.total() >= found_minimum {
                // The shortest path has been found.
                break;
            }

            for (pos, movement) in from_pos.reachable_positions(round.board()) {
                let moves_from_start = prio.from_start() + 1;
                let moves_to_target = moves_to_target(&pos);

                if !self.visited_nodes.add_node(
                    pos.clone(),
                    &from_pos,
                    moves_from_start,
                    movement,
                    BasicVisitedNode::new,
                ) {
                    // This position has already been found with a shorter path.
                    continue;
                }

                if round.target_reached(&pos) {
                    // A better solution has been found.
                    if moves_to_target < found_minimum {
                        found_minimum = moves_from_start;
                        found_final_position = pos.clone();
                    }
                    continue;
                }

                open_list.push_increase(pos, MoveCounter::new(moves_from_start, moves_to_target));
            }
        }

        self.visited_nodes.path_to(&found_final_position)
    }
}

impl Default for AStar {
    fn default() -> Self {
        AStar::new()
    }
}

/// Used to hold the moves needed to reach a robot position and the estimated number of moves to the
/// target.
///
/// `MoveCounter`s are ordered from high to low by the estimated total number of moves from the
/// start to the target. If the totals are the same, the counter with a lower `from_start` value is
///  considered higher in the ordering.
///
/// ```txt
/// MoveCounter(total, from_start)
///
/// MoveCounter(10, 5) < MoveCounter(10, 3) = MoveCounter(10, 3) < MoveCounter(5, 2)
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct MoveCounter {
    // Reordering these fields changes the derived `Ord` and `PartialOrd` implementations.
    total: Reverse<usize>,
    from_start: Reverse<usize>,
}

impl MoveCounter {
    pub fn new(from_start: usize, to_target: usize) -> Self {
        Self {
            total: Reverse(from_start + to_target),
            from_start: Reverse(from_start),
        }
    }

    pub fn from_start(&self) -> usize {
        self.from_start.0
    }

    pub fn total(&self) -> usize {
        self.total.0
    }
}

#[cfg(test)]
mod tests {
    use priority_queue::PriorityQueue;
    use ricochet_board::{template, Direction, Game, Robot, RobotPositions, Round, Symbol, Target};

    use super::{AStar, MoveCounter, Path, Solver};

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

    #[test]
    fn move_counter_ordering() {
        // naming scheme: total_fromStart
        let ten_five = MoveCounter::new(5, 5);
        let ten_three_1 = MoveCounter::new(3, 7);
        let ten_three_2 = MoveCounter::new(3, 7);
        let five_two = MoveCounter::new(2, 3);
        let mut sorted = vec![
            ten_three_1.clone(),
            five_two.clone(),
            ten_five.clone(),
            ten_three_2.clone(),
        ];
        sorted.sort();

        assert_eq!(vec![ten_five, ten_three_1, ten_three_2, five_two], sorted)
    }

    #[test]
    fn move_counter_priority_queue() {
        let mut queue = PriorityQueue::new();
        queue.push("first", MoveCounter::new(3, 7));
        queue.push("second", MoveCounter::new(2, 3));
        queue.push("third", MoveCounter::new(5, 5));
        queue.push("fourth", MoveCounter::new(3, 7));

        let expected = queue.into_sorted_vec();
        assert_eq!(vec!["second", "fourth", "first", "third"], expected)
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

        let expected = Path::new(start.clone(), end, vec![]);
        assert_eq!(AStar::new().solve(&round, start), expected);
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

        let expected = Path::new(
            pos.clone(),
            RobotPositions::from_tuples(&[(10, 15), (9, 11), (7, 1), (9, 12)]),
            vec![
                (Robot::Red, Direction::Right),
                (Robot::Red, Direction::Down),
                (Robot::Red, Direction::Right),
                (Robot::Blue, Direction::Right),
                (Robot::Blue, Direction::Down),
                (Robot::Red, Direction::Left),
                (Robot::Red, Direction::Down),
                (Robot::Yellow, Direction::Right),
                (Robot::Yellow, Direction::Up),
            ],
        );

        assert_eq!(AStar::new().solve(&round, pos), expected);
    }
}
