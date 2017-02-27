#![feature(box_syntax)]

extern crate ricochet_board;
#[macro_use] extern crate enum_primitive;
extern crate num;

use ricochet_board::*;
use num::FromPrimitive;

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
    println!("Start");
    let mut database = Database(box [Entry(255); 1 << 32]);
    let (x, y) = board.targets
        .iter()
        .find(|&&(t, _)| t == target)
        .unwrap()
        .1;
    database.0[positions.0 as usize].reached(0);
    if let Some(result_position) = eval(board, positions, &mut database, x, y, 0, target) {
        return find_direction(0, &mut database, board, result_position);
    }
    for steps in 1.. {
        for j in 0..database.0.len() {
            if database.0[j].steps() == Some(steps) {
                if let Some(result_position) = eval(board,
                                                    RobotPositions(j as u32),
                                                    &mut database,
                                                    x,
                                                    y,
                                                    steps,
                                                    target) {
                    println!("Lösung möglich, als nächstes Weg finden");
                    return find_direction(steps, &mut database, board, result_position);
                }
            }
        }
        assert!(steps < 63);
    }
    unreachable!()
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

const DIRECTIONS: [fn(&mut RobotPositions, robot: Robot, board: &Board); 4] =
    [RobotPositions::move_right,
     RobotPositions::move_left,
     RobotPositions::move_up,
     RobotPositions::move_down];


fn find_direction(steps: u8, //one less than needed to reach the target
                  database: &mut Database,
                  board: &Board,
                  result_position: RobotPositions)
                  -> Vec<(Robot, Direction)> {
    let visited_pos = visited_positions(database, steps);
    let mut path = vec![];
    let mut path_pos = vec![];
    for i in (0..steps-1).rev() {
        for j in 0..visited_pos[i as usize].len() {
            let diff = (j as u32) ^ (result_position.0 as u32); // mark all bits that differ
            let last = diff.trailing_zeros() + 1; // find the position of the most right bit that differed
            let first = 32 - diff.leading_zeros() - 1; // find the position of the most left bit that differed
            let last_sector = last >> 2; // the last two bits only tell which bit of the coordinate changed, drop them
            let first_sector = first >> 2;
            if last_sector == first_sector // if the sector is the same, this is potentially a source location
            {
                match can_reach(&mut RobotPositions(j as u32), result_position, board) {
                    Some(x) => {path_pos.push(RobotPositions(j as u32));
                                path.push(x);
                                break;
                                }
                    None => {}
                }
            }
        }
    }
    return path;
}

fn can_reach(mut start: &mut RobotPositions,
             goal: RobotPositions,
             board: &Board)
             -> Option<(Robot, Direction)> {
    for (i, &robot) in [Robot::Red, Robot::Green, Robot::Blue, Robot::Yellow].iter().enumerate() {
        for (j, dir) in DIRECTIONS.iter().enumerate() {
            dir(&mut start, robot, board);
            if *start == goal {
                return Some((robot, Direction::from_usize(j).unwrap()));
            }
        }
    }
    None
}

/// makes an array with all the positions that were reached with the number of steps needed
/// to reach this position minus one as the index
fn visited_positions(database: &Database,
                     steps: u8)
                     -> Vec<Vec<RobotPositions>> {
    println!("Array mit allen Positionen erzeugen");
    let mut vis_pos = vec![vec![];steps as usize];
    for i in 0..database.0.len() {
        if database.0[i].steps() != None {
            println!("Versuch {}", i);
            vis_pos[(database.0[i].steps().unwrap() as usize - 1)].push(RobotPositions(i as u32));
            println!("Versuch {} geklappt", i);
        }
    }
    println!("Array erzeugt");
    return vis_pos;
}

/// calculates all new possible positions starting from a startposition
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
