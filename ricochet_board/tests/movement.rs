use rustc_serialize::json::*;
use rustc_serialize::Decodable;

use ricochet_board::*;
use std::fs;

fn read() -> (RobotPositions, Board) {
    let mut file = fs::File::open("tests/test.json").expect("test.json not found");
    let json = Json::from_reader(&mut file).expect("invalid json");
    let mut decoder = Decoder::new(json);
    let board =
        Decodable::decode(&mut decoder).expect("json does not match (RobotPosition, Board)");
    let pos = RobotPositions::from_array([(0, 1), (5, 4), (7, 1), (7, 15)]);
    (pos, board)
}

#[test]
fn read_test_json() {
    read();
}

#[test]
fn move_right() {
    let (mut positions, board) = read();
    assert_eq!(positions.green(), Position::from_tuple((7, 1)));
    positions.move_in_direction(&board, Color::Green, Direction::Right);
    assert_eq!(positions.green(), Position::from_tuple((13, 1)));
}

#[test]
fn move_left() {
    let (mut positions, board) = read();
    assert_eq!(positions.green(), Position::from_tuple((7, 1)));
    positions.move_in_direction(&board, Color::Green, Direction::Left);
    assert_eq!(positions.green(), Position::from_tuple((5, 1)));
}

#[test]
fn move_up() {
    let (mut positions, board) = read();
    assert_eq!(positions.green(), Position::from_tuple((7, 1)));
    positions.move_in_direction(&board, Color::Green, Direction::Up);
    assert_eq!(positions.green(), Position::from_tuple((7, 0)));
}

#[test]
fn move_down() {
    let (mut positions, board) = read();
    assert_eq!(positions.green(), Position::from_tuple((7, 1)));
    positions.move_in_direction(&board, Color::Green, Direction::Down);
    assert_eq!(positions.green(), Position::from_tuple((7, 6)));
}
