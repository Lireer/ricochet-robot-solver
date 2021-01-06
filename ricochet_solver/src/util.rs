use std::collections::hash_map::Entry;
use std::convert::TryInto;
use std::ops;

use fnv::FnvHashMap;
use getset::{CopyGetters, Getters};
use ricochet_board::{
    Board, Color, Direction, Position, PositionEncoding, RobotPositions, Target, DIRECTIONS, ROBOTS,
};

use crate::Solution;

#[derive(Debug, Clone)]
pub(crate) struct VisitedNodes {
    nodes: FnvHashMap<RobotPositions, VisitedNode>,
}

impl VisitedNodes {
    /// Creates a new `VisitedNodes` with the given `capacity`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: FnvHashMap::with_capacity_and_hasher(capacity, Default::default()),
        }
    }

    /// Removes all stored nodes.
    pub fn clear(&mut self) {
        self.nodes.clear()
    }

    /// Returns the visit information of a node.
    pub fn get_node(&self, positions: &RobotPositions) -> Option<&VisitedNode> {
        self.nodes.get(positions)
    }

    pub fn add_node(
        &mut self,
        positions: RobotPositions,
        from: &RobotPositions,
        steps: usize,
        moved: (Color, Direction),
    ) -> bool {
        match self.nodes.entry(positions) {
            Entry::Occupied(occupied) if occupied.get().steps_to_reach() <= steps => {
                // Ignore positions if `occupied` has less or equal steps
                false
            }
            Entry::Occupied(mut occupied) => {
                // A shorter path has been found, insert the new path.
                let visited = VisitedNode::new(steps, from.clone(), moved.0, moved.1);
                occupied.insert(visited);
                true
            }
            Entry::Vacant(vacant) => {
                let visited = VisitedNode::new(steps, from.clone(), moved.0, moved.1);
                vacant.insert(visited);
                true
            }
        }
    }

    /// Returns the shortest known path to `positions`.
    ///
    /// # Panics
    /// Panics if `positions` has yet to be visited.
    pub fn path_to(&self, positions: &RobotPositions) -> Solution {
        let mut path = Vec::with_capacity(32);
        let mut current_pos = positions.clone();

        // Create the path by following the nodes previous positions.
        loop {
            let current_node = self
                .get_node(&current_pos)
                .expect("Failed to find a supposed source position");
            path.push(current_node.reached_with());
            current_pos = current_node.previous_position().clone();
            if current_node.steps_to_reach() == 1 {
                // current_pos is now the start of the path
                break;
            }
        }

        path.reverse();
        Solution::new(current_pos, positions.clone(), path)
    }
}

#[derive(Debug, Clone, CopyGetters, Getters)]
pub(crate) struct VisitedNode {
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

#[derive(Debug, Clone)]
pub struct LeastMovesBoard {
    board: Vec<Vec<usize>>,
}

impl LeastMovesBoard {
    pub fn new(board: &Board, target_position: Position) -> Self {
        let len = board.side_length() as usize;
        let mut move_board = vec![vec![len * len; len]; len];

        let mut current_steps = Vec::with_capacity(256);
        let mut next_steps = current_steps.clone();

        move_board[target_position.column() as usize][target_position.row() as usize] = 0;
        current_steps.push(target_position);

        for step in 1usize.. {
            for &pos in &current_steps {
                for &dir in DIRECTIONS.iter() {
                    // Start from pos for each direction.
                    let mut check_pos = pos;
                    loop {
                        if board.is_adjacent_to_wall(check_pos, dir) {
                            break;
                        }
                        check_pos = check_pos.to_direction(dir, len as PositionEncoding);
                        let current_min =
                            &mut move_board[check_pos.column() as usize][check_pos.row() as usize];
                        if step < *current_min {
                            // new position found
                            *current_min = step;
                            next_steps.push(check_pos);
                        }
                    }
                }
            }

            if next_steps.is_empty() {
                break;
            }
            current_steps.clear();
            std::mem::swap(&mut current_steps, &mut next_steps);
        }

        Self { board: move_board }
    }

    pub fn min_steps(&self, robots: &RobotPositions, target: Target) -> usize {
        match target.try_into() {
            Ok(color) => self[robots[color]],
            Err(_) => {
                // The spiral is the target.
                ROBOTS
                    .iter()
                    .map(|&color| self[robots[color]])
                    .min()
                    .expect("Failed to find minimum number of steps to the target.")
            }
        }
    }
}

impl ops::Index<Position> for LeastMovesBoard {
    type Output = usize;

    fn index(&self, index: Position) -> &Self::Output {
        &self.board[index.column() as usize][index.row() as usize]
    }
}

#[cfg(test)]
mod tests {
    use ricochet_board::{Board, Position};

    use super::LeastMovesBoard;

    #[test]
    fn empty_move_board() {
        let board = Board::new_empty(2).wall_enclosure();
        let target = Position::new(0, 0);
        assert_eq!(
            LeastMovesBoard::new(&board, target).board,
            vec![vec![0, 1], vec![1, 2]]
        );
    }

    #[test]
    fn walled_move_board() {
        let board = Board::new_empty(3)
            .wall_enclosure()
            .set_horizontal_line(0, 0, 1)
            .set_horizontal_line(1, 1, 1)
            .set_vertical_line(1, 1, 1);
        let target = Position::new(0, 0);

        assert_eq!(
            LeastMovesBoard::new(&board, target).board,
            vec![vec![0, 3, 3], vec![1, 2, 3], vec![1, 2, 2]]
        );
    }
}
