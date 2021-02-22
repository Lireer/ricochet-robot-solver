#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

//! Basic components to play Ricochet Robots and a reinforcement learning environment.
//!
//! The board game [Ricochet Robots](https://en.wikipedia.org/wiki/Ricochet_Robot) is played on a
//! 16x16 board containing some walls, 4 robots and 17 targets. The game is played in multiple
//! rounds in which a random target is chosen until each target has been chosen once. The goal of
//! a round is to reach the chosen target with the robot of the same color. The robots can each move
//! in all four directions but only stop when they hit a wall or another robot. This is counted as
//! one move and all robots can be moved in arbitrary order. Each player only looks at the board and
//! tries to think of a way to get the robot to the target with as few moves as possible.
//!
//! The main components needed to play the game are the [`Board`](Board), [`Round`](Round), and
//! [`Game`](Game). A `Board` stores all information regarding the walls. A `Round` contains a board
//! and the target on that board. This is the main struct to use, if you don't plan on playing a
//! whole game. A `Game` consist of everything needed to define a complete game. Like a round, it
//! holds a board but also has a set of targets.
//!
//! The physical board is made up of four parts, each of which is assigned a color. There are four
//! colors and multiple board parts per color. To build a complete board one part of each color is
//! needed. The crate provides these parts to make board creation easier, see the
//! [`template`](template) module for more information.

mod draw;
pub mod generator;
mod positions;
pub mod template;

use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};
use std::{fmt, ops};

pub use crate::draw::draw_board;
pub use crate::positions::{Position, PositionEncoding, RobotPositions};
use crate::template::{BoardTemplate, Orientation, WallDirection};

/// The type used to store the walls on a board.
pub type Walls = Vec<Vec<Field>>;

/// All `Direction`s a robot can move in.
pub const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Right,
    Direction::Left,
];

/// All robots defined by their color.
pub const ROBOTS: [Robot; 4] = [Robot::Red, Robot::Blue, Robot::Green, Robot::Yellow];

/// All targets in the game.
pub const TARGETS: [Target; 17] = {
    let mut targets = [Target::Spiral; 17];
    let symbols = &[
        Symbol::Circle,
        Symbol::Triangle,
        Symbol::Square,
        Symbol::Hexagon,
    ];
    let mut target_idx = 1;
    let mut symbol_idx = 0;
    while symbol_idx < 4 {
        targets[target_idx] = Target::Red(symbols[symbol_idx]);
        targets[target_idx + 1] = Target::Blue(symbols[symbol_idx]);
        targets[target_idx + 2] = Target::Green(symbols[symbol_idx]);
        targets[target_idx + 3] = Target::Yellow(symbols[symbol_idx]);
        target_idx += 4;
        symbol_idx += 1;
    }
    targets
};

/// A field on the board.
///
/// Contains information regarding walls to the right and bottom of the field.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct Field {
    /// Returns `true` if the wall in the down direction is set.
    pub down: bool,
    /// Returns `true` if the wall in the right direction is set.
    pub right: bool,
}

/// A game of ricochet on one board with a set of targets.
#[derive(Clone, PartialEq, Eq)]
pub struct Game {
    board: Board,
    targets: BTreeMap<Target, Position>,
}

/// One round of a ricochet game.
///
/// Represents the problem of finding a path from a starting position on a board to a given target.
#[derive(Clone, PartialEq, Eq)]
pub struct Round {
    board: Board,
    target: Target,
    target_position: Position,
}

/// A ricochet robots board containing walls, but no targets.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Board {
    walls: Walls,
}

/// The robots identified by their color.
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Robot {
    Red,
    Blue,
    Green,
    Yellow,
}

/// The different targets to reach.
///
/// The spiral can be reached by any robot, the others have to be reached by the robot of the
/// respective color. Different targets of the same color can be differentiated by looking at the
/// contained [Symbol].
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Target {
    Red(Symbol),
    Blue(Symbol),
    Green(Symbol),
    Yellow(Symbol),
    Spiral,
}

/// Symbols used with colored targets to differentiate between targets of the same color.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symbol {
    Circle,
    Triangle,
    Square,
    Hexagon,
}

/// The directions a robot can be moved in.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    Up,
    Down,
    Right,
    Left,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = format!("{:?}", &self);
        f.pad(&string)
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match *self {
            Target::Red(symb) => format!("Red {:?}", symb),
            Target::Blue(symb) => format!("Blue {:?}", symb),
            Target::Green(symb) => format!("Green {:?}", symb),
            Target::Yellow(symb) => format!("Yellow {:?}", symb),
            Target::Spiral => "Spiral".to_string(),
        };
        f.pad(&string)
    }
}

impl TryFrom<Target> for Robot {
    type Error = &'static str;

    fn try_from(value: Target) -> Result<Self, Self::Error> {
        match value {
            Target::Red(_) => Ok(Robot::Red),
            Target::Blue(_) => Ok(Robot::Blue),
            Target::Green(_) => Ok(Robot::Green),
            Target::Yellow(_) => Ok(Robot::Yellow),
            Target::Spiral => Err("Conversion of spiral target to robot color is not possible"),
        }
    }
}

impl fmt::Display for Robot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = format!("{:?}", &self);
        f.pad(&string)
    }
}

/// Board impl containing code to create or change a board.
impl Board {
    /// Create a new board with the given `walls`.
    ///
    /// # Panics
    /// Panics if not all vecs in `walls` are the same length.
    pub fn new(walls: Walls) -> Self {
        let board_size = walls.len();

        if walls.iter().any(|v| v.len() != board_size) {
            panic!("Tried to create a non-square board.")
        }

        Self { walls }
    }

    /// Create a new empty board with no walls with `side_length`.
    pub fn new_empty(side_length: PositionEncoding) -> Self {
        Self {
            walls: vec![vec![Field::default(); side_length as usize]; side_length as usize],
        }
    }

    /// Returns the side length of the board.
    pub fn side_length(&self) -> PositionEncoding {
        self.walls.len() as PositionEncoding
    }

    /// Encloses the board with walls.
    pub fn wall_enclosure(self) -> Self {
        let side_length = self.side_length();
        self.enclose_lengths(0, 0, side_length, side_length)
    }

    /// Creates a 2x2 block enclosed by walls in the center of the board.
    pub fn set_center_walls(self) -> Self {
        let point = self.side_length() / 2 - 1;
        self.enclose_lengths(point, point, 2, 2)
    }

    /// Encloses a rectangle defined by the left upper corner and its width and height.
    /// The field (col, row) is inside the enclosure. Wraps around at the edge of the board.
    ///
    /// # Panics
    /// Panics if (col, row) is out of bounds.
    pub fn enclose_lengths(
        self,
        col: PositionEncoding,
        row: PositionEncoding,
        len: PositionEncoding,
        width: PositionEncoding,
    ) -> Self {
        let board_size = self.side_length();

        let top_row = if row == 0 { board_size - 1 } else { row - 1 };
        let bottom_row = if row + len > board_size {
            board_size - 1
        } else {
            row + len - 1
        };

        let left_col = if col == 0 { board_size - 1 } else { col - 1 };
        let right_col = if col + width > board_size {
            board_size - 1
        } else {
            col + width - 1
        };

        self.set_horizontal_line(col, top_row, width)
            .set_horizontal_line(col, bottom_row, width)
            .set_vertical_line(left_col, row, len)
            .set_vertical_line(right_col, row, len)
    }

    /// Starting from `[col, row]` sets `len` fields downwards to have a wall on the right side.
    #[inline]
    pub fn set_vertical_line(
        mut self,
        col: PositionEncoding,
        row: PositionEncoding,
        len: PositionEncoding,
    ) -> Self {
        for row in row..(row + len) {
            self.walls[col as usize][row as usize].right = true;
        }
        self
    }

    /// Starting from `[col, row]` sets `len` fields to the right to have a wall on the bottom side.
    #[inline]
    pub fn set_horizontal_line(
        mut self,
        col: PositionEncoding,
        row: PositionEncoding,
        width: PositionEncoding,
    ) -> Self {
        for col in col..(col + width) {
            self.walls[col as usize][row as usize].down = true;
        }
        self
    }
}

/// Board impl containing code to interact with a board.
impl Board {
    /// Returns a reference to the walls of the board.
    pub fn get_walls(&self) -> &Walls {
        &self.walls
    }

    /// Returns a mutable reference to the walls of the board.
    pub fn get_mut_walls(&mut self) -> &mut Walls {
        &mut self.walls
    }

    /// Checks if a wall is next to `pos` in the given `direction`.
    pub fn is_adjacent_to_wall(&self, pos: Position, direction: Direction) -> bool {
        match direction {
            Direction::Right => self.walls[pos.column() as usize][pos.row() as usize].right,
            Direction::Down => self.walls[pos.column() as usize][pos.row() as usize].down,
            Direction::Left => {
                let pos = pos.to_direction(Direction::Left, self.side_length());
                self.walls[pos.column() as usize][pos.row() as usize].right
            }
            Direction::Up => {
                let pos = pos.to_direction(Direction::Up, self.side_length());
                self.walls[pos.column() as usize][pos.row() as usize].down
            }
        }
    }
}

impl ops::Index<Position> for Board {
    type Output = Field;

    fn index(&self, index: Position) -> &Self::Output {
        &self.walls[index.column() as usize][index.row() as usize]
    }
}

impl ops::IndexMut<Position> for Board {
    fn index_mut(&mut self, index: Position) -> &mut Self::Output {
        &mut self.walls[index.column() as usize][index.row() as usize]
    }
}

impl Round {
    /// Creates a new ricochet robots round.
    pub fn new(board: Board, target: Target, target_position: Position) -> Self {
        Self {
            board,
            target,
            target_position,
        }
    }

    /// Returns the `Board` the robots move on.
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Returns the `Target` to be reached.
    pub fn target(&self) -> Target {
        self.target
    }

    /// Returns the targets position.
    pub fn target_position(&self) -> Position {
        self.target_position
    }

    /// Checks if the target has been reached.
    pub fn target_reached(&self, positions: &RobotPositions) -> bool {
        match self.target {
            Target::Spiral => positions.contains_any_robot(self.target_position),
            _ => positions.contains_colored_robot(
                self.target
                    .try_into()
                    .expect("Failed to extract the robot corresponding to the target"),
                self.target_position,
            ),
        }
    }
}

impl Game {
    /// Creates a new game with the given board and targets.
    pub fn new(board: Board, targets: BTreeMap<Target, Position>) -> Self {
        Self { board, targets }
    }

    /// Creates a new game with an empty square board.
    ///
    /// No walls or targets are set.
    pub fn new_empty(side_length: PositionEncoding) -> Self {
        Game {
            board: Board::new_empty(side_length),
            targets: Default::default(),
        }
    }

    /// Creates a new game with an enclosed board with a enclosed 2x2 block in the center.
    pub fn new_enclosed(side_length: PositionEncoding) -> Self {
        let board = Board::new_empty(side_length)
            .wall_enclosure() // Set outer walls
            .set_center_walls(); // Set walls around the four center fields

        Game {
            board,
            targets: Default::default(),
        }
    }

    /// Returns the board the game is being played on.
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Returns the targets on the board.
    pub fn targets(&self) -> &BTreeMap<Target, Position> {
        &self.targets
    }

    /// Returns the position of a target if it exists on the board.
    pub fn get_target_position(&self, target: &Target) -> Option<Position> {
        self.targets.get(target).cloned()
    }
}

impl Game {
    /// Creates a 16x16 game board from a list of templates for board quarters.
    pub fn from_templates(temps: &[BoardTemplate]) -> Self {
        let mut game = Game::new_enclosed(template::STANDARD_BOARD_SIZE);
        for temp in temps {
            game.add_template(temp);
        }
        game
    }

    /// Adds a template for a board quarter to the board.
    ///
    /// Panics if `self.side_length() != 16`.
    fn add_template(&mut self, temp: &BoardTemplate) {
        // get the needed offset
        let (col_add, row_add) = match temp.orientation() {
            Orientation::UpperLeft => (0, 0),
            Orientation::UpperRight => (8, 0),
            Orientation::BottomRight => (8, 8),
            Orientation::BottomLeft => (0, 8),
        };

        // set the walls
        let walls: &mut Walls = &mut self.board.walls;
        for ((c, r), dir) in temp.walls() {
            let c = (c + col_add) as usize;
            let r = (r + row_add) as usize;

            match dir {
                WallDirection::Down => walls[c][r].down = true,
                WallDirection::Right => walls[c][r].right = true,
            }
        }

        // set the targets
        for ((c, r), target) in temp.targets() {
            let c = (c + col_add) as PositionEncoding;
            let r = (r + row_add) as PositionEncoding;
            self.targets.insert(*target, Position::new(c, r));
        }
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", draw_board(&self.walls))
    }
}

impl fmt::Debug for Round {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", draw_board(&self.board.walls))
    }
}

impl fmt::Debug for Game {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", draw_board(&self.board.walls))
    }
}

#[cfg(test)]
mod tests {
    use crate::{template, Board, Direction, Game, Position, Robot, RobotPositions};

    fn create_board() -> (RobotPositions, Board) {
        let templates = template::gen_templates()
            .iter()
            .step_by(3)
            .cloned()
            .enumerate()
            .map(|(i, mut temp)| {
                temp.rotate_to(template::ORIENTATIONS[i]);
                temp
            })
            .collect::<Vec<template::BoardTemplate>>();

        let pos = RobotPositions::from_tuples(&[(0, 1), (5, 4), (7, 1), (7, 15)]);
        let board = Game::from_templates(&templates).board;
        (pos, board)
    }

    #[test]
    fn board_creation() {
        create_board();
    }

    #[test]
    fn move_right() {
        let (mut positions, board) = create_board();
        assert_eq!(positions[Robot::Green], Position::from((7, 1)));
        positions = positions.move_in_direction(&board, Robot::Green, Direction::Right);
        assert_eq!(positions[Robot::Green], Position::from((15, 1)));
    }

    #[test]
    fn move_left() {
        let (mut positions, board) = create_board();
        assert_eq!(positions[Robot::Green], Position::from((7, 1)));
        positions = positions.move_in_direction(&board, Robot::Green, Direction::Left);
        assert_eq!(positions[Robot::Green], Position::from((5, 1)));
    }

    #[test]
    fn move_up() {
        let (mut positions, board) = create_board();
        assert_eq!(positions[Robot::Green], Position::from((7, 1)));
        positions = positions.move_in_direction(&board, Robot::Green, Direction::Up);
        assert_eq!(positions[Robot::Green], Position::from((7, 0)));
    }

    #[test]
    fn move_down() {
        let (mut positions, board) = create_board();
        assert_eq!(positions[Robot::Green], Position::from((7, 1)));
        positions = positions.move_in_direction(&board, Robot::Green, Direction::Down);
        assert_eq!(positions[Robot::Green], Position::from((7, 6)));
    }
}
