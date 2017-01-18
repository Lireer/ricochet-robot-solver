#![feature(box_syntax)]

extern crate ricochet_board;
use ricochet_board::*;


/// lower 6 bit are the number of steps required to reach this position.
/// in case all of the first 6 bits are set, this node has not been visited yet
#[derive(Copy, Clone)]
struct Entry(u8);

/// the u32 position in the database array encodes the robot positions like follows:
///
/// | Red     | Green   | Blue    | Yellow  |
/// |x1  |y1  |x2  |y2  |x3  |y3  |x4  |y4  |
/// |0000|0000|0000|0000|0000|0000|0000|0000|
///
struct Database(Box<[Entry; 1 << 32]>);


impl Entry {
    /// returns the number of steps required to reach this node
    fn steps(self) -> Option<u8> {
        let steps = self.0 & 0b111111;
        if steps == 63 { None } else { Some(steps) }
    }

    fn reached(&mut self, steps: u8) {
        let old = self.0 & 0b111111;
        if old == 63 {
            self.0 = steps;
        }
    }
}

pub fn solve(board: &Board, positions: RobotPositions, target: Target) -> Vec<(Robot, Direction)> {
    let mut database = Database(box [Entry(255); 1 << 32]);
    let (x, y) = board.targets
        .iter()
        .find(|&&(t, _)| t == target)
        .unwrap()
        .1;
    database.0[positions.0 as usize].reached(0);
    if eval(board, positions, &mut database, x, y, 0, target) {
        return 1;
    }
    for steps in 1.. {
        for j in 0..database.0.len() {
            if database.0[j].steps() == Some(steps) {
                if let Some(target) = eval(board,
                        RobotPositions(j as u32),
                        &mut database,
                        x,
                        y,
                        steps,
                        target) {
                    return find_direction(steps, &mut database, board, target);
                }
            }
        }
        assert!(steps < 20);
    }
    unreachable!()
}

#[derive(Debug)]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
}

const DIRECTIONS: [fn(&mut RobotPositions, robot: Robot, board: &Board); 4] =
    [RobotPositions::move_right,
     RobotPositions::move_left,
     RobotPositions::move_up,
     RobotPositions::move_down];

/*
     // mark all bits that differ
     let diff = pos1 ^ pos2;
     // find the position of the most right bit that differed
     let last = diff.trailing_zeros() + 1;
     // find the position of the most left bit that differed
     let first = 32 - (diff.leading_zeros() + 1);
     // the last two bits only tell which bit of the coordinate changed, drop them
     let last_sector = last >> 2;
     let first_sector = first >> 2;
     // if the sector is the same, this is potentially a source location
     if last_sector == first_sector {
         // yay
     }
*/

fn find_direction(steps: usize, database: &mut Database, board: &Board, target: RobotPositions) -> Vec<(Robot, Direction)> {

}

fn eval(board: &Board,
        start: RobotPositions,
        database: &mut Database,
        target_x: usize,
        target_y: usize,
        steps: u8,
        target: Target)
        -> Option<RobotPositions> {
    let mut new = [[start; 4]; 4];
    for (i, &robot) in [Robot::Red, Robot::Green, Robot::Blue, Robot::Yellow].iter().enumerate() {
        for (j, dir) in DIRECTIONS.iter().enumerate() {
            dir(&mut new[i][j], robot, board);
            database.0[new[i][j].0 as usize].reached(steps + 1);
            match target {
                Target::Spiral => {
                    if new[i][j].contains_robot(target_x, target_y) {
                        return Some(new[i][j]);
                    }
                }
                Target::Red(_) => {
                    if robot == Robot::Red && new[i][j].contains_red(target_x, target_y) {
                        return Some(new[i][j]);
                    }
                }
                Target::Green(_) => {
                    if robot == Robot::Green && new[i][j].contains_green(target_x, target_y) {
                        return Some(new[i][j]);
                    }
                }
                Target::Blue(_) => {
                    if robot == Robot::Blue && new[i][j].contains_blue(target_x, target_y) {
                        return Some(new[i][j]);
                    }
                }
                Target::Yellow(_) => {
                    if robot == Robot::Yellow && new[i][j].contains_yellow(target_x, target_y) {
                        return Some(new[i][j]);
                    }
                }
            }
        }
    }
    None
}
