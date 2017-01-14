extern crate ricochet_board;
extern crate ricochet_solver;
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
fn solve() {
    let (positions, board) = read();
    assert_eq!(ricochet_solver::solve(&board, positions, Target::Red(Symbol::Circle)),
               2);
}
