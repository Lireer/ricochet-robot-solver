use std::{fmt, mem, ops};

use crate::{Board, Color, Direction};

/// The type a position is encoded as.
///
/// Depending on the number of bits in a value, different positions on a board can be encoded. A u8
/// is sufficient to encode any position on the standard board. Using u64 would allow encoding
/// positions on a 2^32x2^32 board, see [Position] for more information.
pub type PositionEncoding = u16;

/// A position on the board.
///
/// ```txt
/// |x   |y   |
/// |0000|0000|
/// ```
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Position {
    encoded_position: PositionEncoding,
}

/// Positions of all robots on the board.
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct RobotPositions {
    red: Position,
    blue: Position,
    green: Position,
    yellow: Position,
}

impl Position {
    /// Number of bits used for the encoding.
    const BIT_COUNT: PositionEncoding = mem::size_of::<PositionEncoding>() as PositionEncoding * 8;

    /// Bitflag used to extract the row information of a position by removing the column bits.
    ///
    /// The first half of the bits is `0` the rest `1`. This would be `0000_1111` for `u8`
    /// or `0000_0000_1111_1111` for `u16`.
    const ROW_FLAG: PositionEncoding = {
        // When 1.50 is stablized, this will be possible.
        // Currently requires the `const_int_pow` feature.
        // (2 as PositionEncoding).pow((Position::BIT_COUNT / 2) as u32) - 1

        let mut flag: PositionEncoding = 1;
        // Add more ones until half the bits are ones.
        while flag.count_ones() < mem::size_of::<PositionEncoding>() as u32 * 8 / 2 {
            flag = (flag << 1) + 1;
        }
        flag
    };

    /// Bitflag used to extract the column information of a position by removing the row bits.
    ///
    /// The first half of the bits is `1` the rest `0`. This would be `1111_0000` for `u8`
    /// or `1111_1111_0000_0000` for `u16`.
    const COLUMN_FLAG: PositionEncoding = Self::ROW_FLAG ^ PositionEncoding::MAX;

    /// Creates a new position.
    ///
    /// The caller has to make sure, that the given coordinates are within the bounds of the board.
    pub fn new(column: PositionEncoding, row: PositionEncoding) -> Self {
        Position {
            encoded_position: (column << (Self::BIT_COUNT / 2)) ^ row,
        }
    }

    /// Wrapper for `Position::new()` to construct a position from a tuple.
    pub fn from_tuple((column, row): (PositionEncoding, PositionEncoding)) -> Self {
        Position::new(column, row)
    }

    /// Returns the column the robot is in.
    #[inline(always)]
    pub fn column(&self) -> PositionEncoding {
        self.encoded_position >> (Self::BIT_COUNT / 2)
    }

    /// Returns the row the robot is in.
    #[inline(always)]
    pub fn row(&self) -> PositionEncoding {
        self.encoded_position & Self::ROW_FLAG
    }

    /// Sets `column` as the new column value.
    fn set_column(&mut self, column: PositionEncoding) {
        self.encoded_position = (column << (Self::BIT_COUNT / 2)) ^ self.row() as PositionEncoding;
    }

    /// Sets `row` as the new row value.
    fn set_row(&mut self, row: PositionEncoding) {
        // get the column of the current position and add the new row information
        self.encoded_position = (self.encoded_position & Self::COLUMN_FLAG) ^ row;
    }

    /// Moves the Position one field to `direction`.
    ///
    /// Wraps around at the edge of the board given by `board_size`.
    pub fn to_direction(mut self, direction: Direction, side_length: PositionEncoding) -> Self {
        match direction {
            Direction::Right => self.set_column((self.column() + 1) % side_length),
            Direction::Left => self.set_column((self.column() + side_length - 1) % side_length),
            Direction::Up => self.set_row((self.row() + side_length - 1) % side_length),
            Direction::Down => self.set_row((self.row() + 1) % side_length),
        };
        self
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.column(), self.row())
    }
}

impl Into<(PositionEncoding, PositionEncoding)> for Position {
    fn into(self) -> (PositionEncoding, PositionEncoding) {
        (self.column(), self.row())
    }
}

impl RobotPositions {
    /// Creates a board from a slice of position tuples.
    ///
    /// The values in `positions` are used in the order red, blue, green, yellow.
    pub fn from_tuples(positions: &[(PositionEncoding, PositionEncoding); 4]) -> Self {
        RobotPositions {
            red: Position::from_tuple(positions[0]),
            blue: Position::from_tuple(positions[1]),
            green: Position::from_tuple(positions[2]),
            yellow: Position::from_tuple(positions[3]),
        }
    }

    /// Sets the robot with `color` to `new_position`.
    fn set_robot(&mut self, robot: Color, new_position: Position) {
        *match robot {
            Color::Red => &mut self.red,
            Color::Blue => &mut self.blue,
            Color::Green => &mut self.green,
            Color::Yellow => &mut self.yellow,
        } = new_position;
    }

    /// Checks if `pos` has any robot on it.
    #[inline(always)]
    pub fn contains_any_robot(&self, pos: Position) -> bool {
        pos == self.red || pos == self.blue || pos == self.green || pos == self.yellow
    }

    /// Checks if the robot with `color` is on `pos`.
    #[inline(always)]
    pub fn contains_colored_robot(&self, color: Color, pos: Position) -> bool {
        match color {
            Color::Red => pos == self.red,
            Color::Blue => pos == self.blue,
            Color::Green => pos == self.green,
            Color::Yellow => pos == self.yellow,
        }
    }

    /// Checks if the adjacent field in the direction is reachable, i.e. no wall inbetween and not
    /// already occupied.
    fn adjacent_reachable(&self, board: &Board, pos: Position, direction: Direction) -> bool {
        !board.is_adjacent_to_wall(pos, direction)
            && !self.contains_any_robot(pos.to_direction(direction, board.side_length()))
    }

    /// Moves `robot` as far in the given `direction` as possible.
    pub fn move_in_direction(mut self, board: &Board, robot: Color, direction: Direction) -> Self {
        // start form the current position
        let mut temp_pos = self[robot];

        // check if the next position is reachable from the temporary position
        while self.adjacent_reachable(board, temp_pos, direction) {
            temp_pos = temp_pos.to_direction(direction, board.side_length());
        }

        // set the robot to the last possible position
        self.set_robot(robot, temp_pos);

        self
    }
}

impl ops::Index<Color> for RobotPositions {
    type Output = Position;

    fn index(&self, index: Color) -> &Self::Output {
        match index {
            Color::Red => &self.red,
            Color::Blue => &self.blue,
            Color::Green => &self.green,
            Color::Yellow => &self.yellow,
        }
    }
}

impl fmt::Debug for RobotPositions {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "[{:?} | {:?} | {:?} | {:?}]",
            self.red, self.blue, self.green, self.yellow
        )
    }
}

impl fmt::Display for RobotPositions {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "Red: {}\nBlue: {}\nGreen: {}\nYellow: {}",
            format!("{},{}", self.red.column() + 1, self.red.row() + 1),
            format!("{},{}", self.blue.column() + 1, self.blue.row() + 1),
            format!("{},{}", self.green.column() + 1, self.green.row() + 1),
            format!("{},{}", self.yellow.column() + 1, self.yellow.row() + 1),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Position;

    #[test]
    fn check_flags() {
        let row_flag = 2u16.pow((Position::BIT_COUNT / 2) as u32) - 1;
        assert_eq!(row_flag, Position::ROW_FLAG);
        assert_eq!(!row_flag, Position::COLUMN_FLAG);
    }
}