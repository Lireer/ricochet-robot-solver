extern crate ricochet_board;
extern crate ricochet_solver;
extern crate rustc_serialize;
#[macro_use]
extern crate text_io;

use rustc_serialize::json::*;
use std::fs::File;
use std::io::prelude::*;
use ricochet_board::*;
use ricochet_solver::*;

fn main() {
    // Erzeugung des Boards
    let board = example_board();

    // Erzeugung der Positionen
    let mut positions = ask_for_robotpositions();
    let mut target = ask_for_target();
    // Target::Red(Symbol::Circle);

    let mut save = File::create("test.json").expect("Create test.json");
    write!(save, "{}", as_pretty_json(&(&positions, &board))).expect("Write into test.json");

    'outer: loop {
        println!("Solving...");
        let solve = solve(&board, positions, target);
        let path = solve.1;
        println!("Steps needed to reach target: {}", path.len());
        println!("Press enter to show path.");
        let key: String = read!("{}\n");
        println!("Step Robot   Direction");
        for i in 0..path.len() {
            println!(" {step:>2}  {robot:<8}{dir:<6}",
                     step = i + 1,
                     robot = path[i].0,
                     dir = path[i].1);
        }
        println!("Continue? (Y/n)");
        loop {
            let input: String = read!("{}\n");
            match input.to_lowercase().trim() {
                "y" | "" => break,
                "n" => break 'outer,
                _ => println!("Input not accepted! {}", input),
            }
        }
        println!("Is the end position the new starting position? (Y/n)");
        loop {
            let input: String = read!("{}\n");
            match input.to_lowercase().trim() {
                "y" | "" => {
                    positions = solve.0;
                    break;
                }
                "n" => {
                    positions = ask_for_robotpositions();
                    break;
                }
                _ => println!("Input not accepted! {}", input),
            }
        }
        println!("Please enter the new target");
        target = ask_for_target();
    }
}

fn ask_for_target() -> Target {
    println!("What color is the target?");
    println!("Accepted input: \"red\"(r), \"green\"(g), \"blue\"(b), \"yellow\"(y), \"spiral\"(s)");
    let color: String = read!("{}\n");
    loop {
        match color.to_lowercase().trim() {
            "red" | "r" => return Target::Red(ask_for_symbol()),
            "green" | "g" => return Target::Green(ask_for_symbol()),
            "blue" | "b" => return Target::Blue(ask_for_symbol()),
            "yellow" | "y" => return Target::Yellow(ask_for_symbol()),
            "spiral" | "s" => return Target::Spiral,
            _ => println!("Input not accepted: {}", color),
        }
    }
}

fn ask_for_symbol() -> Symbol {
    println!("What is the shape of the target?");
    println!("Accepted input: \"Circle\"(c), \"Triangle\"(t), \"Square\"(s), \"Hexagon\"(h)");
    let shape: String = read!("{}\n");
    loop {
        match shape.to_lowercase().trim() {
            "circle" | "c" => return Symbol::Circle,
            "triangle" | "t" => return Symbol::Triangle,
            "square" | "s" => return Symbol::Square,
            "hexagon" | "h" => return Symbol::Hexagon,
            _ => println!("Input not accepted: {}", shape),
        }
    }
}

fn ask_for_robotpositions() -> RobotPositions {
    let mut positions = [(0, 0); 4];
    println!("Please input the coordinates of the Robots.\nPlease write in this format: \"x,y\"");
    for (i, &robot) in [Robot::Red, Robot::Green, Robot::Blue, Robot::Yellow].iter().enumerate() {
        println!("{:?}: ", robot);
        let a: u8;
        let b: u8;
        let pos: String = read!("{}\n");
        scan!(pos.trim().bytes() => "{},{}", a, b);
        positions[i] = (a, b);
    }
    return RobotPositions::from_array(positions);
}

fn example_board() -> Board {
    let mut board = default_board();
    fill_board_with_walls(&mut board); // Set walls on example board
    set_targets_on_board(&mut board); // Set targets on example board
    return board;
}

fn default_board() -> Board {
    let mut board = Board {
        fields: [[Field {
            bottom: false,
            right: false,
        }; BOARDSIZE]; BOARDSIZE],
        targets: Default::default(),
    };
    board.wall_enclosure(); // Set outer walls
    board.set_center_walls(); // Set walls around the four center fields
    return board;
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
    board.targets.insert((Target::Red(Circle), (12, 12))); //(14, 4)
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
