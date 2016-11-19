extern crate rustc_serialize;

#[derive(RustcDecodable, RustcEncodable, Copy, Clone)]
pub struct Field {
    pub bottom: bool,
    pub right: bool,
    pub target: Option<Target>,
}

pub const BOARDSIZE: usize = 16;

#[derive(RustcDecodable, RustcEncodable)]
pub struct Board {
    pub fields: [[Field; BOARDSIZE]; BOARDSIZE],
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct RobotPositions {
    pub rob_position: [(usize, usize); 4],
}

#[derive(PartialEq,Copy, Clone)]
pub enum Robot {
    Red = 0,
    Green = 1,
    Blue = 2,
    Yellow = 3,
}
#[derive(RustcDecodable, RustcEncodable, Clone, Copy)]
pub enum Target {
    Red(Symbol),
    Green(Symbol),
    Blue(Symbol),
    Yellow(Symbol),
    Spiral,
}

#[derive(RustcDecodable, RustcEncodable, Clone, Copy)]
pub enum Symbol {
    Circle,
    Triangle,
    Square,
    Hexagon,
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
    fn contains_robot(&self, x: usize, y: usize) -> bool {
        for &(k, m) in &self.rob_position {
            if x == k && y == m {
                return true;
            }
        }
        false
    }

    fn can_move_right(&self, board: &Board, x: usize, y: usize) -> bool {
        !board.wall_right(x, y) && !self.contains_robot((x + 1) % BOARDSIZE, y)
    }

    fn can_move_bottom(&self, board: &Board, x: usize, y: usize) -> bool {
        !board.wall_bottom(x, y) && !self.contains_robot(x, (y + 1) % BOARDSIZE)
    }

    fn can_move_left(&self, board: &Board, x: usize, y: usize) -> bool {
        !board.wall_left(x, y) && !self.contains_robot((x + BOARDSIZE - 1) % BOARDSIZE, y)
    }

    fn can_move_top(&self, board: &Board, x: usize, y: usize) -> bool {
        !board.wall_top(x, y) && !self.contains_robot(x, (y + BOARDSIZE - 1) % BOARDSIZE)
    }
}

impl RobotPositions {
    pub fn move_right(&mut self, robot: Robot, board: &Board) {
        let (x, y) = self.rob_position[robot as usize];
        for x_tmp in x.. {
            let x_tmp = x_tmp % BOARDSIZE;
            if !self.can_move_right(board, x_tmp, y) {
                if x != x_tmp {
                    self.rob_position[robot as usize] = (x_tmp, y);
                }
                // TODO Zielabfrage
                return;
            }
        }
    }

    pub fn move_bottom(&mut self, robot: Robot, board: &Board) {
        let (x, y) = self.rob_position[robot as usize];
        for y_tmp in y.. {
            let y_tmp = y_tmp % BOARDSIZE;
            if !self.can_move_bottom(board, x, y_tmp) {
                if y != y_tmp {
                    self.rob_position[robot as usize] = (x, y_tmp);
                }
                // TODO Zielabfrage
                return;
            }
        }
    }

    pub fn move_left(&mut self, robot: Robot, board: &Board) {
        let (x, y) = self.rob_position[robot as usize];
        for i in 0.. {
            let x = (x + BOARDSIZE - i) % BOARDSIZE;
            if !self.can_move_left(board, x, y) {
                if i != 0 {
                    self.rob_position[robot as usize] = (x, y);
                }
                // TODO Zielabfrage
                return;
            }
        }
    }

    pub fn move_top(&mut self, robot: Robot, board: &Board) {
        let (x, y) = self.rob_position[robot as usize];
        for i in 0.. {
            let y = (y + BOARDSIZE - i) % BOARDSIZE;
            if !self.can_move_top(board, x, y) {
                if i != 0 {
                    self.rob_position[robot as usize] = (x, y);
                }
                // TODO Zielabfrage
                return;
            }
        }
    }
}
