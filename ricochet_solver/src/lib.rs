mod breadth_first;
mod iterative_deepening;

use getset::{CopyGetters, Getters};
use ricochet_board::{Color, Direction, RobotPositions, Round};

pub use breadth_first::BreadthFirst;
pub use iterative_deepening::IterativeDeepening;

pub trait Solver {
    /// Find a solution to get from the `start_positions` to a target position.
    fn solve(&mut self, round: &Round, start_positions: RobotPositions) -> Solution;
}

/// A solution to a ricochet robots problem.
///
/// Contains the starting positions of the robots, their final positions and a path from the former
/// to the latter. The path consists of tuples of the robot color and the direction it moved to.
#[derive(Debug, Clone, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct Solution {
    start_pos: RobotPositions,
    end_pos: RobotPositions,
    path: Vec<(Color, Direction)>,
}

impl Solution {
    /// Creates a new solution containing the starting and final positions of the robots and a path
    /// to reach the target.
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

    /// Creates a new solution in which the robot reaches the target without moving.
    pub fn new_start_on_target(start_pos: RobotPositions) -> Self {
        Self::new(start_pos.clone(), start_pos, Vec::new())
    }
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

    pub fn reached_with(&self) -> (Color, Direction) {
        (self.robot, self.direction)
    }
}
