extern crate ricochet_board;
extern crate rustc_serialize;

use rustc_serialize::json::*;
use std::fs::File;
use std::io::prelude::*;
use ricochet_board::*;

fn main() {
    // Erzeugung der Positionen
    let positions = RobotPositions::from_array([(0, 1), (7, 1), (5, 4), (7, 15)]);

    // Erzeugung des Boards
    let mut board = Board {
        fields: [[Field {
            bottom: false,
            right: false,
        }; BOARDSIZE]; BOARDSIZE],
        targets: Default::default(),
    };

    /// Beispiel Board mit Wänden und Targets
    board.wall_enclosure();    // Set outer walls
    board.set_center_walls();   // Set walls around the four center fields
    fill_board_with_walls(&mut board);    // Example board
    set_targets_on_board(&mut board);    // Set targets on board


    let mut save = File::create("test.json").expect("Schreiben der json-Datei");

    write!(save, "{}", as_pretty_json(&(&positions, &board))).expect("Die json-Datei beschreiben");
}

fn fill_board_with_walls(board: &mut Board) {
    board.fields[1][0].right = true;
    board.fields[4][0].bottom = true;
    board.fields[9][0].right = true;
    board.fields[4][1].right = true;
    board.fields[13][1].right = true;
    board.fields[13][1].bottom = true;
    board.fields[15][1].bottom = true;
    board.fields[9][2].bottom = true;
    board.fields[0][3].right = true;
    board.fields[1][3].bottom = true;
    board.fields[8][3].right = true;
    board.fields[14][3].bottom = true;
    board.fields[5][4].bottom = true;
    board.fields[14][4].right = true;
    board.fields[0][5].bottom = true;
    board.fields[4][5].right = true;
    board.fields[3][6].bottom = true;
    board.fields[3][6].right = true;
    board.fields[11][6].right = true;
    board.fields[12][6].bottom = true;
    board.fields[2][7].bottom = true;
    board.fields[2][8].right = true;
    board.fields[4][9].right = true;
    board.fields[5][9].bottom = true;
    board.fields[12][9].right = true;
    board.fields[13][9].bottom = true;
    board.fields[1][10].right = true;
    board.fields[1][10].bottom = true;
    board.fields[4][11].bottom = true;
    board.fields[9][11].right = true;
    board.fields[9][11].bottom = true;
    board.fields[15][11].bottom = true;
    board.fields[0][12].bottom = true;
    board.fields[4][12].right = true;
    board.fields[14][12].bottom = true;
    board.fields[6][13].bottom = true;
    board.fields[10][13].bottom = true;
    board.fields[14][13].right = true;
    board.fields[5][14].right = true;
    board.fields[9][14].right = true;
    board.fields[3][15].right = true;
    board.fields[11][15].right = true;
}

fn set_targets_on_board(board: &mut Board) {
    use ricochet_board::Symbol::*;
    board.targets.insert((Target::Spiral, (2, 8)));
    board.targets.insert((Target::Red(Circle), (14, 4)));
    board.targets.insert((Target::Red(Triangle), (1, 3)));
    board.targets.insert((Target::Red(Square), (14, 13)));
    board.targets.insert((Target::Red(Hexagon), (4, 12)));
    board.targets.insert((Target::Green(Circle), (4, 1)));
    board.targets.insert((Target::Green(Triangle), (13, 1)));
    board.targets.insert((Target::Green(Square), (5, 9)));
    board.targets.insert((Target::Green(Hexagon), (10, 14)));
    board.targets.insert((Target::Blue(Circle), (1, 10)));
    board.targets.insert((Target::Blue(Triangle), (13, 9)));
    board.targets.insert((Target::Blue(Square), (9, 4)));
    board.targets.insert((Target::Blue(Hexagon), (3, 6)));
    board.targets.insert((Target::Yellow(Circle), (9, 11)));
    board.targets.insert((Target::Yellow(Triangle), (6, 14)));
    board.targets.insert((Target::Yellow(Square), (5, 5)));
    board.targets.insert((Target::Yellow(Hexagon), (12, 6)));
}
