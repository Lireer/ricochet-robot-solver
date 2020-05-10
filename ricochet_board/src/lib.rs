#![feature(const_int_pow)]

pub mod template;

use std::collections::BTreeMap;
use std::{fmt, mem};
use template::{BoardTemplate, Orientation, WallDirection};

pub const BOARDSIZE: usize = 16;

#[derive(RustcDecodable, RustcEncodable, Copy, Clone)]
pub struct Field {
    pub down: bool,
    pub right: bool,
}

impl Default for Field {
    fn default() -> Self {
        Field {
            down: false,
            right: false,
        }
    }
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct Board {
    pub fields: [[Field; BOARDSIZE]; BOARDSIZE],
    pub targets: BTreeMap<Target, (usize, usize)>,
}

// using an u64 we could encode a 256x256 board and a 65536x65536 board using an u128
pub type PositionEncoding = u8;

/// A position on the board.
///
/// |x   |y   |
/// |0000|0000|
#[derive(RustcDecodable, RustcEncodable, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Position {
    encoded_position: PositionEncoding,
}

impl Position {
    /// Size of the encoded value in bits.
    const SIZE: PositionEncoding = mem::size_of::<PositionEncoding>() as PositionEncoding * 8;
    /// Bitflag used to extract the row information of a position by removing the column bits.
    /// The first half of the bits is `0` the rest `1`. This would be `0000_1111` for `u8`
    /// or `0000_0000_1111_1111` for `u16`.
    const ROW_FLAG: PositionEncoding = (2 as PositionEncoding).pow((Position::SIZE / 2) as u32) - 1;

    /// Creates a new position.
    ///
    /// The caller has to make sure, that the given coordinates are within the bounds of the board.
    pub fn new(column: PositionEncoding, row: PositionEncoding) -> Self {
        Position {
            encoded_position: (column << (Self::SIZE / 2)) ^ row,
        }
    }

    /// Wrapper for `Position::new()` to construct a position from a tuple.
    pub fn from_tuple((column, row): (PositionEncoding, PositionEncoding)) -> Self {
        Position::new(column, row)
    }

    #[inline(always)]
    pub fn column(&self) -> usize {
        (self.encoded_position >> (Self::SIZE / 2)) as usize
    }

    #[inline(always)]
    pub fn row(&self) -> usize {
        (self.encoded_position & Self::ROW_FLAG) as usize
    }

    /// Creates a new `Position` in the given direction.
    /// Wraps around at the edge of the board.
    fn to_direction(&self, direction: Direction) -> Position {
        let (col, row) = match direction {
            Direction::Right => ((self.column() + 1) % BOARDSIZE, self.row()),
            Direction::Left => ((self.column() + BOARDSIZE - 1) % BOARDSIZE, self.row()),
            Direction::Up => (self.column(), (self.row() + BOARDSIZE - 1) % BOARDSIZE),
            Direction::Down => (self.column(), (self.row() + 1) % BOARDSIZE),
        };
        Position::new(col as PositionEncoding, row as PositionEncoding)
    }
}

#[derive(RustcDecodable, RustcEncodable, Copy, Clone, PartialEq, Eq)]
pub struct RobotPositions {
    red: Position,
    blue: Position,
    green: Position,
    yellow: Position,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Color {
    Red = 0,
    Blue = 1,
    Green = 2,
    Yellow = 3,
}

#[derive(Debug, RustcDecodable, RustcEncodable, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Target {
    Red(Symbol),
    Blue(Symbol),
    Green(Symbol),
    Yellow(Symbol),
    Spiral,
}

#[derive(Debug, RustcDecodable, RustcEncodable, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symbol {
    Circle,
    Triangle,
    Square,
    Hexagon,
}

#[derive(Debug, RustcDecodable, RustcEncodable, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    Up,
    Down,
    Right,
    Left,
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

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = format!("{:?}", &self);
        f.pad(&string)
    }
}

impl Default for Board {
    fn default() -> Self {
        let board = Board {
            fields: [[Field {
                down: false,
                right: false,
            }; BOARDSIZE]; BOARDSIZE],
            targets: Default::default(),
        };
        board
            .wall_enclosure() // Set outer walls
            .set_center_walls() // Set walls around the four center fields
    }
}

impl Board {
    pub fn from_templates(temps: &[BoardTemplate]) -> Self {
        let mut board = Board::default();
        for temp in temps {
            board.add_template(temp);
        }
        board
    }

    fn add_template(&mut self, temp: &BoardTemplate) {
        // get the needed offset
        let (col_add, row_add) = match temp.orientation() {
            Orientation::UpperLeft => (0, 0),
            Orientation::UpperRight => (8, 0),
            Orientation::BottomRight => (8, 8),
            Orientation::BottomLeft => (0, 8),
        };

        // set the walls
        for ((c, r), dir) in temp.walls() {
            let c = (c + col_add) as usize;
            let r = (r + row_add) as usize;

            match dir {
                WallDirection::Down => self.fields[c][r].down = true,
                WallDirection::Right => self.fields[c][r].right = true,
            }
        }

        // set the targets
        for ((c, r), target) in temp.targets() {
            let c = (c + col_add) as usize;
            let r = (r + row_add) as usize;
            self.targets.insert(*target, (c, r));
        }
    }

    pub fn wall_enclosure(self) -> Self {
        self.enclose_lengths(0, 0, BOARDSIZE, BOARDSIZE)
    }

    // only useful for 16x16 board
    pub fn set_center_walls(self) -> Self {
        self.enclose_lengths(7, 7, 2, 2)
    }

    /// Encloses a rectangle defined by the left upper corner and its width and length.
    /// The field [col, row] is inside the enclosure. Wraps around at the end of the board.
    /// # Panics
    /// - Panics if [col, row] is out of bounds.
    pub fn enclose_lengths(self, col: usize, row: usize, len: usize, width: usize) -> Self {
        let top_row = if row == 0 { BOARDSIZE - 1 } else { row - 1 };
        let bottom_row = if row + len > BOARDSIZE {
            BOARDSIZE - 1
        } else {
            row + len - 1
        };

        let left_col = if col == 0 { BOARDSIZE - 1 } else { col - 1 };
        let right_col = if col + width > BOARDSIZE {
            BOARDSIZE - 1
        } else {
            col + width - 1
        };

        self.set_horizontal_line(col, top_row, width)
            .set_horizontal_line(col, bottom_row, width)
            .set_vertical_line(left_col, row, len)
            .set_vertical_line(right_col, row, len)
    }

    /// Starts from `[col, row]` and sets `len` fields below to have a wall on the right side
    #[inline]
    fn set_vertical_line(mut self, col: usize, row: usize, len: usize) -> Self {
        for row in row..(row + len) {
            self.fields[col][row].right = true;
        }
        self
    }

    /// Starts from `[col, row]` and sets `len` fields to the right to have a wall on the bottom side
    #[inline]
    fn set_horizontal_line(mut self, col: usize, row: usize, width: usize) -> Self {
        for col in col..(col + width) {
            self.fields[col][row].down = true;
        }
        self
    }

    pub fn adjacent_to_wall(&self, pos: Position, direction: Direction) -> bool {
        match direction {
            Direction::Right => self.fields[pos.column()][pos.row()].right,
            Direction::Down => self.fields[pos.column()][pos.row()].down,
            Direction::Left => {
                let pos = pos.to_direction(Direction::Left);
                self.fields[pos.column()][pos.row()].right
            }
            Direction::Up => {
                let pos = pos.to_direction(Direction::Up);
                self.fields[pos.column()][pos.row()].down
            }
        }
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let to_print: Vec<Vec<Field>> = self.fields.iter().map(|&a| a.to_vec()).collect();
        write!(fmt, "{}", board_string(to_print))
    }
}

impl RobotPositions {
    #[inline(always)]
    pub fn contains_any_robot(self, pos: Position) -> bool {
        pos == self.red || pos == self.blue || pos == self.green || pos == self.yellow
    }

    #[inline(always)]
    pub fn contains_colored_robot(self, color: Color, pos: Position) -> bool {
        match color {
            Color::Red => pos == self.red,
            Color::Blue => pos == self.blue,
            Color::Green => pos == self.green,
            Color::Yellow => pos == self.yellow,
        }
    }

    /// Checks if the adjacent field in the direction is reachable, i.e. no wall
    /// inbetween and not already occupied.
    fn adjacent_reachable(self, board: &Board, pos: Position, direction: Direction) -> bool {
        !board.adjacent_to_wall(pos, direction)
            && !self.contains_any_robot(pos.to_direction(direction))
    }
}

impl RobotPositions {
    /// Move `robot` as far in the given `direction` as possible.
    pub fn move_in_direction(&mut self, board: &Board, robot: Color, direction: Direction) {
        // start form the current position
        let mut temp_pos = self.get_robot(robot);

        // check if the next position is reachable from the temporary position
        while self.adjacent_reachable(board, temp_pos, direction) {
            temp_pos = temp_pos.to_direction(direction);
        }

        // return the last possible position
        self.set_robot(robot, temp_pos);
    }
}

impl RobotPositions {
    pub fn from_array(positions: [(PositionEncoding, PositionEncoding); 4]) -> Self {
        RobotPositions {
            red: Position::from_tuple(positions[0]),
            blue: Position::from_tuple(positions[1]),
            green: Position::from_tuple(positions[2]),
            yellow: Position::from_tuple(positions[3]),
        }
    }

    /// Updates self with `new_positions`.
    fn set_robot(&mut self, robot: Color, new_position: Position) {
        *match robot {
            Color::Red => &mut self.red,
            Color::Blue => &mut self.blue,
            Color::Green => &mut self.green,
            Color::Yellow => &mut self.yellow,
        } = new_position;
    }

    fn get_robot(self, color: Color) -> Position {
        match color {
            Color::Red => self.red,
            Color::Blue => self.blue,
            Color::Green => self.green,
            Color::Yellow => self.yellow,
        }
    }

    pub fn red(self) -> Position {
        self.red
    }

    pub fn blue(self) -> Position {
        self.blue
    }

    pub fn green(self) -> Position {
        self.green
    }

    pub fn yellow(self) -> Position {
        self.yellow
    }
}

impl fmt::Debug for RobotPositions {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "[{:?}, {:?}, {:?}, {:?}]",
            self.red(),
            self.blue(),
            self.green(),
            self.yellow()
        )
    }
}

impl fmt::Display for RobotPositions {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "Red: {}\nBlue: {}\nGreen: {}\nYellow: {}",
            format!("{},{}", self.red.column(), self.red.row()),
            format!("{},{}", self.blue.column(), self.blue.row()),
            format!("{},{}", self.green.column(), self.green.row()),
            format!("{},{}", self.yellow.column(), self.yellow.row()),
        )
    }
}

pub fn board_string(board: Vec<Vec<Field>>) -> String {
    let mut print = "".to_owned();
    for row in 0..board.len() {
        #[allow(clippy::needless_range_loop)]
        for col in 0..board[row].len() {
            if board[col][row].down {
                print += "__"
            } else {
                print += "▆▆"
            }
            if board[col][row].right {
                print += "|"
            } else {
                print += " "
            }
        }
        print += "\n";
    }
    print
}
