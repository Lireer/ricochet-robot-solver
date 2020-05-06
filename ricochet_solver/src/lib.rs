use enum_primitive::*;
use num::FromPrimitive;
use std::fmt;

use ricochet_board::{Board, Robot, RobotPosition, Target};

/// the u32 position in the database vec encodes the robot positions like follows:
///
/// | Red     | Green   | Blue    | Yellow  |
/// |x1  |y1  |x2  |y2  |x3  |y3  |x4  |y4  |
/// |0000|0000|0000|0000|0000|0000|0000|0000|
///
pub struct Database(pub Vec<Entry>);

impl Database {
    /// Creates a new Database with a capacity of `2^32`
    pub fn new() -> Self {
        Database(vec![Entry(255); 1 << 32])
    }
}

/// lower 6 bit are the number of steps required to reach this position.
/// in case all of the first 6 bits are set, this node has not been visited yet
#[derive(Copy, Clone)]
pub struct Entry(pub u8);

impl Entry {
    /// returns the number of steps required to reach this node
    fn steps(self) -> Option<u8> {
        let steps = self.0 & 0b11_1111;
        if steps == 63 {
            None
        } else {
            Some(steps)
        }
    }

    fn reached(&mut self, steps: u8) {
        let old = self.0 & 0b11_1111;
        if old == 63 {
            self.0 = steps;
        }
    }
}

pub fn solve(
    board: &Board,
    positions: RobotPosition,
    target: Target,
    mut database: Database,
) -> (RobotPosition, Vec<(Robot, Direction)>) {
    let (target_col, target_row) = board.targets.iter().find(|&&(t, _)| t == target).unwrap().1;
    match target {
        Target::Spiral => {
            if positions.contains_robot(target_col, target_row) {
                return (positions, vec![]);
            }
        }
        Target::Red(_) => {
            if positions.contains_red(target_col, target_row) {
                return (positions, vec![]);
            }
        }
        Target::Green(_) => {
            if positions.contains_green(target_col, target_row) {
                return (positions, vec![]);
            }
        }
        Target::Blue(_) => {
            if positions.contains_blue(target_col, target_row) {
                return (positions, vec![]);
            }
        }
        Target::Yellow(_) => {
            if positions.contains_yellow(target_col, target_row) {
                return (positions, vec![]);
            }
        }
    }
    database.0[positions.0 as usize].reached(0);
    vec_solve(board, positions, target, target_col, target_row, database)
}

fn vec_solve(
    board: &Board,
    positions: RobotPosition,
    target: Target,
    target_col: usize,
    target_row: usize,
    mut database: Database,
) -> (RobotPosition, Vec<(Robot, Direction)>) {
    let mut visited_pos = vec![vec![]];
    visited_pos[0] = vec![positions];
    for steps in 0.. {
        for i in 0..visited_pos[steps].len() {
            if let Some(result_position) = eval(
                board,
                visited_pos[steps][i],
                &mut database,
                target_col,
                target_row,
                steps as u8,
                target,
                &mut visited_pos,
            ) {
                return (
                    result_position,
                    find_direction((steps + 1) as u8, board, result_position, visited_pos),
                );
            }
        }
    }
    (positions, vec![])
}

enum_from_primitive! {
    #[derive(Debug,Eq,PartialEq)]
    pub enum Direction {
        Right = 0,
        Left = 1,
        Up = 2,
        Down = 3,
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = format!("{:?}", &self);
        f.pad(&string)
    }
}

const DIRECTIONS: [fn(&mut RobotPosition, robot: Robot, board: &Board); 4] = [
    RobotPosition::move_right,
    RobotPosition::move_left,
    RobotPosition::move_up,
    RobotPosition::move_down,
];

fn find_direction(
    steps: u8, // number of steps needed to reach the target
    board: &Board,
    result_position: RobotPosition,
    visited_pos: Vec<Vec<RobotPosition>>,
) -> Vec<(Robot, Direction)> {
    let mut path = vec![];
    let mut current_goal = result_position;
    for i in (0..(steps)).rev() {
        for j in 0..visited_pos[i as usize].len() {
            let diff = (visited_pos[i as usize][j as usize].0 as u32) ^ (current_goal.0 as u32); // mark all bits that differ
            let last = diff.trailing_zeros(); // find the position of the most right bit that differed
            let first = 32 - diff.leading_zeros() - 1; // find the position of the most left bit that differed
            let last_sector = last >> 2; // the last two bits only tell which bit of the coordinate changed, drop them
            let first_sector = first >> 2;
            if last_sector == first_sector
            // if the sector is the same, this is potentially a source location
            {
                if let Some(col) =
                    can_reach(visited_pos[i as usize][j as usize], current_goal, board)
                {
                    path.push(col);
                    current_goal = visited_pos[i as usize][j as usize];
                    break;
                }
            }
        }
    }
    path.reverse();
    path
}

fn can_reach(
    start: RobotPosition,
    goal: RobotPosition,
    board: &Board,
) -> Option<(Robot, Direction)> {
    for &robot in [Robot::Red, Robot::Green, Robot::Blue, Robot::Yellow].iter() {
        for (j, dir) in DIRECTIONS.iter().enumerate() {
            let mut start = start;
            dir(&mut start, robot, board);
            if start == goal {
                return Some((robot, Direction::from_usize(j).unwrap()));
            }
        }
    }
    None
}

/// calculates all new possible positions starting from a startposition
fn eval(
    board: &Board,
    start: RobotPosition,
    database: &mut Database,
    target_col: usize,
    target_row: usize,
    steps: u8,
    target: Target,
    visited_pos: &mut Vec<Vec<RobotPosition>>,
) -> Option<RobotPosition> {
    let mut new = [[start; 4]; 4];
    let mut vec: Vec<RobotPosition> = Vec::new();

    if visited_pos.len() == steps as usize + 1 {
        visited_pos.push(vec![]);
    }
    for (i, &robot) in [Robot::Red, Robot::Green, Robot::Blue, Robot::Yellow]
        .iter()
        .enumerate()
    {
        for (j, dir) in DIRECTIONS.iter().enumerate() {
            dir(&mut new[i][j], robot, board);
            if database.0[new[i][j].0 as usize].steps() == None {
                vec.push(new[i][j]);
            }
            database.0[new[i][j].0 as usize].reached(steps + 1);
            match target {
                Target::Spiral => {
                    if new[i][j].contains_robot(target_col, target_row) {
                        return Some(new[i][j]);
                    }
                }
                Target::Red(_) => {
                    if robot == Robot::Red && new[i][j].contains_red(target_col, target_row) {
                        return Some(new[i][j]);
                    }
                }
                Target::Green(_) => {
                    if robot == Robot::Green && new[i][j].contains_green(target_col, target_row) {
                        return Some(new[i][j]);
                    }
                }
                Target::Blue(_) => {
                    if robot == Robot::Blue && new[i][j].contains_blue(target_col, target_row) {
                        return Some(new[i][j]);
                    }
                }
                Target::Yellow(_) => {
                    if robot == Robot::Yellow && new[i][j].contains_yellow(target_col, target_row) {
                        return Some(new[i][j]);
                    }
                }
            }
        }
    }
    visited_pos[steps as usize + 1].append(&mut vec);
    None
}
