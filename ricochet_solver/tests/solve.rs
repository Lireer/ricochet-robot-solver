use rustc_serialize::json::*;
use rustc_serialize::Decodable;

use ricochet_board::*;
use ricochet_solver::*;
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
fn solve() {
    let (positions, board) = read();
    let database = Database::new();
    assert_eq!(
        ricochet_solver::solve(&board, positions, Target::Red(Symbol::Square), database),
        (
            RobotPosition(3980809343 as u32),
            vec![
                (Robot::Red, Direction::Right),
                (Robot::Red, Direction::Down),
                (Robot::Green, Direction::Down),
                (Robot::Green, Direction::Left),
                (Robot::Red, Direction::Up),
                (Robot::Red, Direction::Right),
                (Robot::Red, Direction::Down),
                (Robot::Red, Direction::Right)
            ]
        )
    );
}
