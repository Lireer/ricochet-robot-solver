mod breadth_first;
mod iterative_deepening;
pub mod util;

use getset::Getters;
use ricochet_board::{Direction, Robot, RobotPositions, Round};

pub use breadth_first::BreadthFirst;
pub use iterative_deepening::IterativeDeepening;

pub trait Solver {
    /// Find a solution to get from the `start_positions` to a target position.
    fn solve(&mut self, round: &Round, start_positions: RobotPositions) -> Path;
}

/// A path from a starting position to another position.
///
/// Contains the starting positions of the robots, their final positions and a path from the former
/// to the latter. The path consists of tuples of a robot and the direction it moved to.
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
        path: Vec<(Robot, Direction)>,
    ) -> Self {
        Self {
            start_pos,
            end_pos,
            movements: path,
        }
    }

    /// Creates a new path which ends on the starting position.
    pub fn new_start_on_target(start_pos: RobotPositions) -> Self {
        Self::new(start_pos.clone(), start_pos, Vec::new())
    }
}
