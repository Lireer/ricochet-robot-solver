extern crate ricochet_board;
extern crate rustc_serialize;
use rustc_serialize::json::*;
use rustc_serialize::Decodable;

use ricochet_board::*;
use std::fs::File;

fn read() -> (RobotPositions, Board) {
    let mut file = File::open("tests/test.json").expect("test.json not found");
    let json = Json::from_reader(&mut file).expect("invalid json");
    let mut decoder = Decoder::new(json);
    Decodable::decode(&mut decoder).expect("json does not match (RobotPositions, Board)")
}

#[test]
fn read_test_json() {
    read();
}

#[test]
fn move_right() {
    let (mut positions, board) = read();
    assert_eq!(positions.rob_position[0], (14, 6));
    positions.move_right(Robot::Red, &board);
    assert_eq!(positions.rob_position[0], (15, 6));
}

#[test]
fn move_left() {
    let (mut positions, board) = read();
    assert_eq!(positions.rob_position[0], (14, 6));
    positions.move_left(Robot::Red, &board);
    assert_eq!(positions.rob_position[0], (12, 6));
}

#[test]
fn move_up() {
    let (mut positions, board) = read();
    assert_eq!(positions.rob_position[0], (14, 6));
    positions.move_up(Robot::Red, &board);
    assert_eq!(positions.rob_position[0], (14, 3));
}

#[test]
fn move_down() {
    let (mut positions, board) = read();
    assert_eq!(positions.rob_position[0], (14, 6));
    positions.move_down(Robot::Red, &board);
    assert_eq!(positions.rob_position[0], (14, 13));
}
