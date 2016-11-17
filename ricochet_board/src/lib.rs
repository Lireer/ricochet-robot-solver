use std::collections::BTreeMap;

pub struct Field {
    bottom: bool,
    right: bool,
    target: Option<Target>,
}


const BOARDSIZE: usize = 16;

pub struct Board {
    fields: [[Field; BOARDSIZE]; BOARDSIZE],
}

pub struct RobotPositions {
    rob_position: BTreeMap<(usize, usize), Robot>,
}

#[derive(PartialEq,Copy, Clone)]
pub enum Robot {
    Red,
    Green,
    Blue,
    Yellow,
}

pub enum Target {
    Red(Symbol),
    Green(Symbol),
    Blue(Symbol),
    Yellow(Symbol),
    Spiral,
}

pub enum Symbol {
    Circle,
    Triangle,
    Square,
    Hexagon,
}

impl Board {
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
            self.wall_right(x, BOARDSIZE - 1)
        } else {
            self.wall_right(x, y - 1)
        }
    }
}

impl RobotPositions {
    fn can_move_right(&self, board: &Board, x: usize, y: usize) -> bool {
        !board.wall_right(x, y) && self.rob_position.contains_key(&((x + 1) % BOARDSIZE, y))
    }

    fn can_move_bottom(&self, board: &Board, x: usize, y: usize) -> bool {
        !board.wall_bottom(x, y) && self.rob_position.contains_key(&(x, (y + 1) % BOARDSIZE))
    }

    fn can_move_left(&self, board: &Board, x: usize, y: usize) -> bool {
        !board.wall_left(x, y) &&
        self.rob_position.contains_key(&((x + BOARDSIZE - 1) % BOARDSIZE, y))
    }

    fn can_move_top(&self, board: &Board, x: usize, y: usize) -> bool {
        !board.wall_top(x, y) &&
        self.rob_position.contains_key(&(x, (y + BOARDSIZE - 1) % BOARDSIZE))
    }
}

impl RobotPositions {
    pub fn move_right(&mut self, robot: Robot, board: &Board) {
        let &(x, y) = self.rob_position.iter().find(|&(_, v)| *v == robot).unwrap().0;
        for x_tmp in x.. {
            if !self.can_move_right(board, x_tmp, y) {
                if x != x_tmp {
                    self.rob_position.insert((x_tmp, y), robot);
                    self.rob_position.remove(&(x, y));
                }
                // TODO Zielabfrage
                return;
            }
        }
    }

    pub fn move_bottom(&mut self, robot: Robot, board: &Board) {
        let &(x, y) = self.rob_position.iter().find(|&(_, v)| *v == robot).unwrap().0;
        for y_tmp in y.. {
            if !self.can_move_bottom(board, x, y_tmp) {
                if y != y_tmp {
                    self.rob_position.insert((x, y_tmp), robot);
                    self.rob_position.remove(&(x, y));
                }
                // TODO Zielabfrage
                return;
            }
        }
    }

    pub fn move_left(&mut self, robot: Robot, board: &Board) {
        let &(x, y) = self.rob_position.iter().find(|&(_, v)| *v == robot).unwrap().0;
        for i in 0.. {
            if !self.can_move_left(board, x - i, y) {
                if i != 0 {
                    self.rob_position.insert((x - i, y), robot);
                    self.rob_position.remove(&(x, y));
                }
                // TODO Zielabfrage
                return;
            }
        }
    }

    pub fn move_top(&mut self, robot: Robot, board: &Board) {
        let &(x, y) = self.rob_position.iter().find(|&(_, v)| *v == robot).unwrap().0;
        for i in 0.. {
            if !self.can_move_top(board, x, y - 1) {
                if i != 0 {
                    self.rob_position.insert((x, y - i), robot);
                    self.rob_position.remove(&(x, y));
                }
                // TODO Zielabfrage
                return;
            }
        }
    }
}
