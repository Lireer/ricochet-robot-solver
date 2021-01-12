use std::collections::hash_map::Entry;
use std::convert::TryInto;
use std::ops;

use fnv::FnvHashMap;
use ricochet_board::{
    Board, Direction, Position, PositionEncoding, Robot, RobotPositions, Target, DIRECTIONS, ROBOTS,
};

use crate::Path;

#[derive(Debug, Clone)]
pub(crate) struct VisitedNodes<N: VisitedNode> {
    nodes: FnvHashMap<RobotPositions, N>,
}

impl<N: VisitedNode> VisitedNodes<N> {
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
    pub fn get_node(&self, positions: &RobotPositions) -> Option<&N> {
        self.nodes.get(positions)
    }

    pub fn add_node<F>(
        &mut self,
        positions: RobotPositions,
        from: &RobotPositions,
        moves: usize,
        moved: (Robot, Direction),
        create_node: F,
    ) -> bool
    where
        F: FnOnce(usize, RobotPositions, (Robot, Direction)) -> N,
    {
        match self.nodes.entry(positions) {
            Entry::Occupied(occupied) if occupied.get().moves_to_reach() <= moves => {
                // Ignore `positions` if `occupied` has less or equal moves.
                false
            }
            Entry::Occupied(mut occupied) => {
                // A shorter path has been found, insert the new node.
                let visited = create_node(moves, from.clone(), moved);
                occupied.insert(visited);
                true
            }
            Entry::Vacant(vacant) => {
                let visited = create_node(moves, from.clone(), moved);
                vacant.insert(visited);
                true
            }
        }
    }

    /// Returns the shortest known path to `positions`.
    ///
    /// # Panics
    /// Panics if `positions` has yet to be visited.
    pub fn path_to(&self, positions: &RobotPositions) -> Path {
        let mut path = Vec::with_capacity(32);
        let mut current_pos = positions.clone();

        // Create the path by following the nodes previous positions.
        loop {
            let current_node = self
                .get_node(&current_pos)
                .expect("Failed to find a supposed source position");
            path.push(current_node.reached_with());
            current_pos = current_node.previous_position().clone();
            if current_node.moves_to_reach() == 1 {
                // current_pos is now the start of the path
                break;
            }
        }

        path.reverse();
        Path::new(current_pos, positions.clone(), path)
    }
}

/// Defines the functionality and information a visited node has to provide.
///
/// This makes it possible to have differently optimized implementations depending on the algorithm.
pub(crate) trait VisitedNode {
    /// Returns the number of moves needed to reach this node.
    fn moves_to_reach(&self) -> usize;

    /// Returns the `RobotPositions` this node was reached from.
    fn previous_position(&self) -> &RobotPositions;

    /// Returns the robot and the direction it has to be moved in to reach `self` from the previous
    /// position.
    fn reached_with(&self) -> (Robot, Direction);
}

#[derive(Debug, Clone)]
pub(crate) struct BasicVisitedNode {
    moves_to_reach: usize,
    previous_position: RobotPositions,
    robot: Robot,
    direction: Direction,
}

impl BasicVisitedNode {
    pub fn new(
        moves: usize,
        previous_position: RobotPositions,
        movement: (Robot, Direction),
    ) -> Self {
        BasicVisitedNode {
            moves_to_reach: moves,
            previous_position,
            robot: movement.0,
            direction: movement.1,
        }
    }
}

impl VisitedNode for BasicVisitedNode {
    fn moves_to_reach(&self) -> usize {
        self.moves_to_reach
    }

    fn previous_position(&self) -> &RobotPositions {
        &self.previous_position
    }

    fn reached_with(&self) -> (Robot, Direction) {
        (self.robot, self.direction)
    }
}

/// This board contains the minimum number of moves to reach the target for each field.
///
/// This minimum is a lower bound and may be impossible to reach even if all other robots are
/// positioned perfectly. If the lower bound of a position is the square of the side_length of the
/// board or the number of fields plus one, then the target is impossible to reach from that field.
///
/// `LeastMovesBoard` implements `Index<Position>` which makes getting the calculated minimum of a
/// positon easy.
#[derive(Debug, Clone, Default)]
pub struct LeastMovesBoard {
    board: Vec<Vec<usize>>,
    target_position: Position,
}

impl LeastMovesBoard {
    /// Creates a new board and calculates the minimum number of moves needed to reach the target
    /// from each field.
    ///
    /// The board is created by starting from the target position and going through all fields from
    /// which the target can be reached in one move. These fields are assigned a lower bound of 1
    /// and are added to the list of next positons to be expanded. This repeats until only a subset
    /// of the positions from which the target can never be reached are left. Those positions are
    /// marked with a lower bound of `board.side_length().pow(2)`, a bound longer than possible on a
    /// square board.
    pub fn new(board: &Board, target_position: Position) -> Self {
        let len = board.side_length() as usize;
        let mut move_board = vec![vec![len * len; len]; len];

        let mut current_moves = Vec::with_capacity(256);
        let mut next_moves = current_moves.clone();

        move_board[target_position.column() as usize][target_position.row() as usize] = 0;
        current_moves.push(target_position);

        for move_n in 1usize.. {
            for &pos in &current_moves {
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
                        if move_n < *current_min {
                            // new position found
                            *current_min = move_n;
                            next_moves.push(check_pos);
                        }
                    }
                }
            }

            if next_moves.is_empty() {
                break;
            }
            current_moves.clear();
            std::mem::swap(&mut current_moves, &mut next_moves);
        }

        Self {
            board: move_board,
            target_position,
        }
    }

    /// Returns the lower bound of the number of moves needed to reach the `target` with `robots`.
    ///
    /// The lower bound is chosen depending on the robot and in case of the spiral target the
    /// minimum of any of the four robots is returned.
    pub fn min_moves(&self, robots: &RobotPositions, target: Target) -> usize {
        match target.try_into() {
            Ok(color) => self[robots[color]],
            Err(_) => {
                // The spiral is the target.
                ROBOTS
                    .iter()
                    .map(|&color| self[robots[color]])
                    .min()
                    .expect("Failed to find minimum number of moves to the target.")
            }
        }
    }

    /// Checks whether the `target` is impossible to reach by checking if the lower bound returned
    /// by [`min_moves`](Self::min_moves) is greater than the number of fields on the board.
    pub fn is_unsolvable(&self, robots: &RobotPositions, target: Target) -> bool {
        self.min_moves(robots, target) > self.board.len().pow(2)
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
