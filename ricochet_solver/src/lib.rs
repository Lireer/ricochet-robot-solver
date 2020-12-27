mod breadth_first;

use getset::Getters;
use ricochet_board::{Color, Direction, RobotPositions, Round};

pub use breadth_first::BreadthFirst;

pub trait Solver {
    /// Find a solution to get from the `start_positions` to a target position.
    fn solve(&mut self, round: &Round, start_positions: RobotPositions) -> Solution;
}

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct Solution {
    start_pos: RobotPositions,
    end_pos: RobotPositions,
    path: Vec<(Color, Direction)>,
}

impl Solution {
    pub fn new(
        start_pos: RobotPositions,
        end_pos: RobotPositions,
        path: Vec<(Color, Direction)>,
    ) -> Self {
        Self {
            start_pos,
            end_pos,
            path,
        }
    }
}
