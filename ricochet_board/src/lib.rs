use std::collections::BTreeSet;
use std::fmt;

pub const BOARDSIZE: usize = 16;

// using an u64 we could encode a 256x256 board and a 65536x65536 board using an u128
type PositionEncoding = u32;

#[derive(RustcDecodable, RustcEncodable, Copy, Clone)]
pub struct Field {
    pub bottom: bool,
    pub right: bool,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct Board {
    pub fields: [[Field; BOARDSIZE]; BOARDSIZE],
    pub targets: BTreeSet<(Target, (usize, usize))>,
}

#[derive(RustcDecodable, RustcEncodable, Copy, Clone, PartialEq, Eq)]
pub struct RobotPosition(pub PositionEncoding);

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Robot {
    Red = 0,
    Green = 1,
    Blue = 2,
    Yellow = 3,
}

#[derive(Debug, RustcDecodable, RustcEncodable, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Target {
    Red(Symbol),
    Green(Symbol),
    Blue(Symbol),
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

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match *self {
            Target::Red(symb) => format!("Red {:?}", symb),
            Target::Green(symb) => format!("Green {:?}", symb),
            Target::Blue(symb) => format!("Blue {:?}", symb),
            Target::Yellow(symb) => format!("Yellow {:?}", symb),
            Target::Spiral => "Spiral".to_string(),
        };
        f.pad(&string)
    }
}

impl fmt::Display for Robot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = format!("{:?}", &self);
        f.pad(&string)
    }
}

impl Board {
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
            self.fields[col][row].bottom = true;
        }
        self
    }

    #[inline]
    pub fn wall_right(&self, col: usize, row: usize) -> bool {
        self.fields[col][row].right
    }

    #[inline]
    pub fn wall_bottom(&self, col: usize, row: usize) -> bool {
        self.fields[col][row].bottom
    }

    #[inline]
    pub fn wall_left(&self, col: usize, row: usize) -> bool {
        if col == 0 {
            self.wall_right(BOARDSIZE - 1, row)
        } else {
            self.wall_right(col - 1, row)
        }
    }

    #[inline]
    pub fn wall_top(&self, col: usize, row: usize) -> bool {
        if row == 0 {
            self.wall_bottom(col, BOARDSIZE - 1)
        } else {
            self.wall_bottom(col, row - 1)
        }
    }
}

impl RobotPosition {
    pub fn contains_robot(self, col: usize, row: usize) -> bool {
        let byte = ((col << 4) | row) as PositionEncoding;
        ((self.0 & 0xFF) == byte)
            || (((self.0 >> 8) & 0xFF) == byte)
            || (((self.0 >> 16) & 0xFF) == byte)
            || (((self.0 >> 24) & 0xFF) == byte)
    }

    pub fn contains_red(self, col: usize, row: usize) -> bool {
        let byte = ((col << 4) | row) as PositionEncoding;
        (((self.0 >> 24) & 0xFF) == byte)
    }

    pub fn contains_green(self, col: usize, row: usize) -> bool {
        let byte = ((col << 4) | row) as PositionEncoding;
        (((self.0 >> 16) & 0xFF) == byte)
    }

    pub fn contains_blue(self, col: usize, row: usize) -> bool {
        let byte = ((col << 4) | row) as PositionEncoding;
        (((self.0 >> 8) & 0xFF) == byte)
    }

    pub fn contains_yellow(self, col: usize, row: usize) -> bool {
        let byte = ((col << 4) | row) as PositionEncoding;
        ((self.0 & 0xFF) == byte)
    }

    fn can_move_right(self, board: &Board, col: usize, row: usize) -> bool {
        !board.wall_right(col, row) && !self.contains_robot((col + 1) % BOARDSIZE, row)
    }

    fn can_move_down(self, board: &Board, col: usize, row: usize) -> bool {
        !board.wall_bottom(col, row) && !self.contains_robot(col, (row + 1) % BOARDSIZE)
    }

    fn can_move_left(self, board: &Board, col: usize, row: usize) -> bool {
        !board.wall_left(col, row) && !self.contains_robot((col + BOARDSIZE - 1) % BOARDSIZE, row)
    }

    fn can_move_up(self, board: &Board, col: usize, row: usize) -> bool {
        !board.wall_top(col, row) && !self.contains_robot(col, (row + BOARDSIZE - 1) % BOARDSIZE)
    }
}

impl RobotPosition {
    pub fn move_right(&mut self, robot: Robot, board: &Board) {
        let (col, row) = self.robot(robot);
        for col_tmp in col.. {
            let col_tmp = col_tmp % BOARDSIZE;
            if !self.can_move_right(board, col_tmp, row) {
                if col != col_tmp {
                    self.set_robot(robot, (col_tmp, row));
                }
                break;
            }
        }
    }

    pub fn move_down(&mut self, robot: Robot, board: &Board) {
        let (col, row) = self.robot(robot);
        for row_tmp in row.. {
            let row_tmp = row_tmp % BOARDSIZE;
            if !self.can_move_down(board, col, row_tmp) {
                if row != row_tmp {
                    self.set_robot(robot, (col, row_tmp));
                }
                break;
            }
        }
    }

    pub fn move_left(&mut self, robot: Robot, board: &Board) {
        let (col, row) = self.robot(robot);
        for i in 0.. {
            let col = (col + BOARDSIZE - i) % BOARDSIZE;
            if !self.can_move_left(board, col, row) {
                if i != 0 {
                    self.set_robot(robot, (col, row));
                }
                break;
            }
        }
    }

    pub fn move_up(&mut self, robot: Robot, board: &Board) {
        let (col, row) = self.robot(robot);
        for i in 0.. {
            let row = (row + BOARDSIZE - i) % BOARDSIZE;
            if !self.can_move_up(board, col, row) {
                if i != 0 {
                    self.set_robot(robot, (col, row));
                }
                break;
            }
        }
    }
}

impl RobotPosition {
    pub fn from_array(pos: [(u8, u8); 4]) -> Self {
        RobotPosition(
            (PositionEncoding::from(pos[0].0) << 28)
                | (PositionEncoding::from(pos[0].1) << 24)
                | (PositionEncoding::from(pos[1].0) << 20)
                | (PositionEncoding::from(pos[1].1) << 16)
                | (PositionEncoding::from(pos[2].0) << 12)
                | (PositionEncoding::from(pos[2].1) << 8)
                | (PositionEncoding::from(pos[3].0) << 4)
                | PositionEncoding::from(pos[3].1),
        )
    }
    pub fn set_robot(&mut self, rob: Robot, (col, row): (usize, usize)) {
        let pos = ((col as PositionEncoding) << 4) | (row as PositionEncoding);
        let rob = rob as usize;
        self.0 &= !(0xFF << (8 * (3 - rob)));
        self.0 |= pos << (8 * (3 - rob));
    }
    pub fn robot(self, rob: Robot) -> (usize, usize) {
        let rob = rob as usize;
        let pos = self.0 >> (8 * (3 - rob));
        (((pos >> 4) & 0xF) as usize, (pos & 0xF) as usize)
    }
    pub fn red(self) -> (usize, usize) {
        ((self.0 >> 28) as usize, ((self.0 >> 24) & 0xF) as usize)
    }
    pub fn green(self) -> (usize, usize) {
        (
            ((self.0 >> 20) & 0xF) as usize,
            ((self.0 >> 16) & 0xF) as usize,
        )
    }
    pub fn blue(self) -> (usize, usize) {
        (
            ((self.0 >> 12) & 0xF) as usize,
            ((self.0 >> 8) & 0xF) as usize,
        )
    }
    pub fn yellow(self) -> (usize, usize) {
        (((self.0 >> 4) & 0xF) as usize, (self.0 & 0xF) as usize)
    }

    pub fn red_display(self) -> String {
        format!("{},{}", self.red().0, self.red().1)
    }
    pub fn green_display(self) -> String {
        format!("{},{}", self.green().0, self.green().1)
    }
    pub fn blue_display(self) -> String {
        format!("{},{}", self.blue().0, self.blue().1)
    }
    pub fn yellow_display(self) -> String {
        format!("{},{}", self.yellow().0, self.yellow().1)
    }
}

impl fmt::Debug for RobotPosition {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "[{:?}, {:?}, {:?}, {:?}]",
            self.red(),
            self.green(),
            self.blue(),
            self.yellow()
        )
    }
}

impl fmt::Display for RobotPosition {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "Red: {}\nGreen: {}\nBlue: {}\nYellow: {}",
            self.red_display(),
            self.green_display(),
            self.blue_display(),
            self.yellow_display()
        )
    }
}
