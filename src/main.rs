extern crate ricochet_board;
extern crate rustc_serialize;

use rustc_serialize::json::*;
use std::fs::File;
use std::io::prelude::*;
use ricochet_board::*;

fn main() {
    // Erzeugung der Positionen
    let mut positions = RobotPositions { rob_position: [(14, 6), (0, 5), (5, 4), (7, 15)] };

    // Erzeugung des Boards
    let mut board = Board {
        fields: [[Field {
            bottom: false,
            right: false,
            target: None,
        }; BOARDSIZE]; BOARDSIZE],
    };

    /// Beispiel Board mit Wänden und Targets
    ///
    board.wall_enclosure();    // Board mit Wänden umranden
    fill_board_with_walls(&mut board);    // Beispiel Board mit Wänden erzeugen
    set_targets_on_board(&mut board);    // Targets auf Beispiel Board setzen


    let mut save = File::create("Test.json").expect("Schreiben der json-Datei");

    write!(save, "{}", as_pretty_json(&(&positions, &board))).expect("Die json-Datei beschreiben");
}

fn fill_board_with_walls(board: &mut Board) {
    board.fields[4][0].right = true;
    board.fields[10][0].right = true;
    board.fields[12][0].bottom = true;
    board.fields[2][1].right = true;
    board.fields[2][1].bottom = true;
    board.fields[11][1].right = true;
    board.fields[13][2].right = true;
    board.fields[14][2].bottom = true;
    board.fields[0][3].right = true;
    board.fields[1][3].bottom = true;
    board.fields[6][3].bottom = true;
    board.fields[7][3].right = true;
    board.fields[8][3].bottom = true;
    board.fields[0][4].bottom = true;
    board.fields[5][4].right = true;
    board.fields[15][4].bottom = true;
    board.fields[5][5].bottom = true;
    board.fields[9][5].right = true;
    board.fields[9][5].bottom = true;
    board.fields[11][5].bottom = true;
    board.fields[5][6].right = true;
    board.fields[7][6].bottom = true;
    board.fields[8][6].bottom = true;
    board.fields[11][6].right = true;
    board.fields[3][7].right = true;
    board.fields[3][7].bottom = true;
    board.fields[6][7].right = true;
    board.fields[8][7].right = true;
    board.fields[2][8].bottom = true;
    board.fields[4][8].right = true;
    board.fields[5][8].bottom = true;
    board.fields[6][8].right = true;
    board.fields[7][8].bottom = true;
    board.fields[8][8].right = true;
    board.fields[8][8].bottom = true;
    board.fields[1][9].right = true;
    board.fields[15][9].bottom = true;
    board.fields[8][10].right = true;
    board.fields[8][10].bottom = true;
    board.fields[13][10].bottom = true;
    board.fields[0][11].bottom = true;
    board.fields[12][11].right = true;
    board.fields[4][12].right = true;
    board.fields[4][13].right = true;
    board.fields[8][13].right = true;
    board.fields[9][13].bottom = true;
    board.fields[14][13].bottom = true;
    board.fields[1][14].right = true;
    board.fields[1][14].bottom = true;
    board.fields[14][14].right = true;
    board.fields[5][15].right = true;
    board.fields[11][15].right = true;
}

fn set_targets_on_board(board: &mut Board) {
    use ricochet_board::Symbol::*;
    board.fields[6][4].target = Some(Target::Spiral);
    board.fields[2][1].target = Some(Target::Red(Circle));
    board.fields[8][3].target = Some(Target::Red(Triangle));
    board.fields[2][9].target = Some(Target::Red(Square));
    board.fields[9][13].target = Some(Target::Red(Hexagon));
    board.fields[3][7].target = Some(Target::Green(Circle));
    board.fields[5][12].target = Some(Target::Green(Triangle));
    board.fields[14][6].target = Some(Target::Green(Square));
    board.fields[5][8].target = Some(Target::Green(Hexagon));
    board.fields[3][13].target = Some(Target::Blue(Circle));
    board.fields[12][12].target = Some(Target::Blue(Triangle));
    board.fields[8][9].target = Some(Target::Blue(Square));
    board.fields[0][4].target = Some(Target::Blue(Hexagon));
    board.fields[1][14].target = Some(Target::Yellow(Circle));
    board.fields[14][14].target = Some(Target::Yellow(Triangle));
    board.fields[14][3].target = Some(Target::Yellow(Square));
    board.fields[11][6].target = Some(Target::Yellow(Hexagon));
}
