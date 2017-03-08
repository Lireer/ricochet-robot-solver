extern crate rustc_serialize;

use std::collections::BTreeSet;
use std::fmt;

#[derive(RustcDecodable, RustcEncodable, Copy, Clone)]
pub struct Field {
    pub bottom: bool,
    pub right: bool,
}

pub const BOARDSIZE: usize = 16;

#[derive(RustcDecodable, RustcEncodable)]
pub struct Board {
    pub fields: [[Field; BOARDSIZE]; BOARDSIZE],
    pub targets: BTreeSet<(Target, (usize, usize))>,
}

#[derive(RustcDecodable, RustcEncodable, Copy, Clone, PartialEq, Eq)]
pub struct RobotPositions(pub u32);

#[derive(Debug,PartialEq,Copy, Clone)]
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
    pub fn wall_enclosure(&mut self) {
        for i in 0..BOARDSIZE {
            self.fields[i][BOARDSIZE - 1].bottom = true;
        }

        for i in 0..BOARDSIZE {
            self.fields[BOARDSIZE - 1][i].right = true;
        }
    }

    // only useful for 16x16 board
    pub fn set_center_walls(&mut self) {
        self.fields[6][7].right = true;
        self.fields[6][8].right = true;
        self.fields[7][6].bottom = true;
        self.fields[7][8].bottom = true;
        self.fields[8][6].bottom = true;
        self.fields[8][7].right = true;
        self.fields[8][8].right = true;
        self.fields[8][8].bottom = true;
    }

    pub fn wall_right(&self, x: usize, y: usize) -> bool {
        self.fields[x][y].right
    }

    pub fn wall_bottom(&self, x: usize, y: usize) -> bool {
        self.fields[x][y].bottom
    }

    pub fn wall_left(&self, x: usize, y: usize) -> bool {
        if x == 0 {
            self.wall_right(BOARDSIZE - 1, y)
        } else {
            self.wall_right(x - 1, y)
        }
    }

    pub fn wall_top(&self, x: usize, y: usize) -> bool {
        if y == 0 {
            self.wall_bottom(x, BOARDSIZE - 1)
        } else {
            self.wall_bottom(x, y - 1)
        }
    }
}

impl RobotPositions {
    pub fn contains_robot(&self, x: usize, y: usize) -> bool {
        let byte = ((x << 4) | y) as u32;
        ((self.0 & 0xFF) == byte) || (((self.0 >> 8) & 0xFF) == byte) ||
        (((self.0 >> 16) & 0xFF) == byte) || (((self.0 >> 24) & 0xFF) == byte)
    }

    pub fn contains_red(&self, x: usize, y: usize) -> bool {
        let byte = ((x << 4) | y) as u32;
        (((self.0 >> 24) & 0xFF) == byte)
    }

    pub fn contains_green(&self, x: usize, y: usize) -> bool {
        let byte = ((x << 4) | y) as u32;
        (((self.0 >> 16) & 0xFF) == byte)
    }

    pub fn contains_blue(&self, x: usize, y: usize) -> bool {
        let byte = ((x << 4) | y) as u32;
        (((self.0 >> 8) & 0xFF) == byte)
    }

    pub fn contains_yellow(&self, x: usize, y: usize) -> bool {
        let byte = ((x << 4) | y) as u32;
        ((self.0 & 0xFF) == byte)
    }

    fn can_move_right(&self, board: &Board, x: usize, y: usize) -> bool {
        !board.wall_right(x, y) && !self.contains_robot((x + 1) % BOARDSIZE, y)
    }

    fn can_move_down(&self, board: &Board, x: usize, y: usize) -> bool {
        !board.wall_bottom(x, y) && !self.contains_robot(x, (y + 1) % BOARDSIZE)
    }

    fn can_move_left(&self, board: &Board, x: usize, y: usize) -> bool {
        !board.wall_left(x, y) && !self.contains_robot((x + BOARDSIZE - 1) % BOARDSIZE, y)
    }

    fn can_move_up(&self, board: &Board, x: usize, y: usize) -> bool {
        !board.wall_top(x, y) && !self.contains_robot(x, (y + BOARDSIZE - 1) % BOARDSIZE)
    }
}

impl RobotPositions {
    pub fn move_right(&mut self, robot: Robot, board: &Board) {
        let (x, y) = self.robot(robot);
        for x_tmp in x.. {
            let x_tmp = x_tmp % BOARDSIZE;
            if !self.can_move_right(board, x_tmp, y) {
                if x != x_tmp {
                    self.set_robot(robot, (x_tmp, y));
                }
                return;
            }
        }
    }

    pub fn move_down(&mut self, robot: Robot, board: &Board) {
        let (x, y) = self.robot(robot);
        for y_tmp in y.. {
            let y_tmp = y_tmp % BOARDSIZE;
            if !self.can_move_down(board, x, y_tmp) {
                if y != y_tmp {
                    self.set_robot(robot, (x, y_tmp));
                }
                return;
            }
        }
    }

    pub fn move_left(&mut self, robot: Robot, board: &Board) {
        let (x, y) = self.robot(robot);
        for i in 0.. {
            let x = (x + BOARDSIZE - i) % BOARDSIZE;
            if !self.can_move_left(board, x, y) {
                if i != 0 {
                    self.set_robot(robot, (x, y));
                }
                return;
            }
        }
    }

    pub fn move_up(&mut self, robot: Robot, board: &Board) {
        let (x, y) = self.robot(robot);
        for i in 0.. {
            let y = (y + BOARDSIZE - i) % BOARDSIZE;
            if !self.can_move_up(board, x, y) {
                if i != 0 {
                    self.set_robot(robot, (x, y));
                }
                return;
            }
        }
    }
}

impl RobotPositions {
    pub fn from_array(pos: [(u8, u8); 4]) -> Self {
        RobotPositions(((pos[0].0 as u32) << 28) | ((pos[0].1 as u32) << 24) |
                       ((pos[1].0 as u32) << 20) |
                       ((pos[1].1 as u32) << 16) |
                       ((pos[2].0 as u32) << 12) |
                       ((pos[2].1 as u32) << 8) | ((pos[3].0 as u32) << 4) |
                       pos[3].1 as u32)
    }
    pub fn set_robot(&mut self, rob: Robot, (x, y): (usize, usize)) {
        let pos = ((x as u32) << 4) | (y as u32);
        let rob = rob as usize;
        self.0 &= !(0xFF << 8 * (3 - rob));
        self.0 |= pos << 8 * (3 - rob);
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
        (((self.0 >> 20) & 0xF) as usize, ((self.0 >> 16) & 0xF) as usize)
    }
    pub fn blue(self) -> (usize, usize) {
        (((self.0 >> 12) & 0xF) as usize, ((self.0 >> 8) & 0xF) as usize)
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

impl fmt::Debug for RobotPositions {
    fn fmt(&self, mut fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt,
               "[{:?}, {:?}, {:?}, {:?}]",
               self.red(),
               self.green(),
               self.blue(),
               self.yellow())
    }
}

impl fmt::Display for RobotPositions {
    fn fmt(&self, mut fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt,
               "Red: {}\nGreen: {}\nBlue: {}\nYellow: {}",
               self.red_display(),
               self.green_display(),
               self.blue_display(),
               self.yellow_display())
    }
}
