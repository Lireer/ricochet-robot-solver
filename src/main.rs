extern crate ricochet_board;
extern crate rustc_serialize;

use rustc_serialize::json::*;
use std::fs::File;
use std::io::prelude::*;
use ricochet_board::*;

fn main() {
    let mut positions = RobotPositions { rob_position: [(1, 4), (14, 14), (4, 5), (6, 7)] };
    println!("Text 1");

    let mut board = Board {
        fields: [[Field {
            bottom: false,
            right: false,
            target: None,
        }; BOARDSIZE]; BOARDSIZE],
    };
    println!("Text 2");

    for i in 0..BOARDSIZE {
        board.fields[i][BOARDSIZE - 1].right = true;
    }

    for i in 0..BOARDSIZE {
        board.fields[BOARDSIZE - 1][i].bottom = true;
    }

    // Test warum rot bei 1,4 rauskommt
    // positions.move_bottom(Robot::Red, &board);
    // println!("{0}", positions.rob_position[0]);
    // positions.move_right(Robot::Red, &board);
    // println!("{0}", positions.rob_position[0]);
    // positions.move_top(Robot::Red, &board);
    // println!("{0}", positions.rob_position[0]);
    // positions.move_left(Robot::Red, &board);
    // println!("{0}", positions.rob_position[0]);
    //

    let mut save = File::create("Test.json").expect("Schreiben der json-Datei");

    println!("Text 3");
    write!(save,
           "{0} \n {1}",
           as_pretty_json(&board),
           as_pretty_json(&positions))
        .expect("Die json-Datei beschreiben");
    println!("Text 4");
}
