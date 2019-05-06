use rustc_serialize::json::*;
use rustc_serialize::Decodable;

use ricochet_board::*;
use std::fs::File;

fn read() -> (RobotPosition, Board) {
    let mut file = File::open("tests/test.json").expect("test.json not found");
    let json = Json::from_reader(&mut file).expect("invalid json");
    let mut decoder = Decoder::new(json);
    Decodable::decode(&mut decoder).expect("json does not match (RobotPosition, Board)")
}

#[test]
fn read_test_json() {
    read();
}

#[test]
fn move_right() {
    let (mut positions, board) = read();
    assert_eq!(positions.green(), (7, 1));
    positions.move_right(Robot::Green, &board);
    assert_eq!(positions.green(), (13, 1));
}

#[test]
fn move_left() {
    let (mut positions, board) = read();
    assert_eq!(positions.green(), (7, 1));
    positions.move_left(Robot::Green, &board);
    assert_eq!(positions.green(), (5, 1));
}

#[test]
fn move_up() {
    let (mut positions, board) = read();
    assert_eq!(positions.green(), (7, 1));
    positions.move_up(Robot::Green, &board);
    assert_eq!(positions.green(), (7, 0));
}

#[test]
fn move_down() {
    let (mut positions, board) = read();
    assert_eq!(positions.green(), (7, 1));
    positions.move_down(Robot::Green, &board);
    assert_eq!(positions.green(), (7, 6));
}
