mod a_star;
mod breadth_first;
mod iterative_deepening;
mod mcts;
pub mod util;

use getset::Getters;
use ricochet_board::{Direction, Robot, RobotPositions, Round};

pub use a_star::AStar;
pub use breadth_first::BreadthFirst;
pub use iterative_deepening::IdaStar;
pub use mcts::Mcts;

pub trait Solver {
    /// Find a solution to get from the `start_positions` to a target.
    fn solve(&mut self, round: &Round, start_positions: RobotPositions) -> Path;
}

/// A path from a starting position to another position.
///
/// Contains the starting positions of the robots, their final positions and a path from the former
/// to the latter. The path consists of tuples of a robot and the direction it moved in.
#[derive(Debug, Clone, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct Path {
    start_pos: RobotPositions,
    end_pos: RobotPositions,
    movements: Vec<(Robot, Direction)>,
}

impl Path {
    /// Creates a new path containing the starting and final positions of the robots and a path
    /// to reach the target.
    pub fn new(
        start_pos: RobotPositions,
        end_pos: RobotPositions,
        movements: Vec<(Robot, Direction)>,
    ) -> Self {
        debug_assert!(!movements.is_empty() || start_pos == end_pos);
        Self {
            start_pos,
            end_pos,
            movements,
        }
    }

    /// Creates a new path which ends on the starting position.
    pub fn new_start_on_target(start_pos: RobotPositions) -> Self {
        Self::new(start_pos.clone(), start_pos, Vec::new())
    }

    /// Returns the number of moves in the path.
    pub fn len(&self) -> usize {
        self.movements.len()
    }

    /// Checks if the path has a length of 0.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
