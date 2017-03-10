#![feature(box_syntax)]
extern crate ricochet_board;
extern crate ricochet_solver;
extern crate rustc_serialize;
#[macro_use]
extern crate text_io;

// use rustc_serialize::json::*;
// use std::fs::File;
// use std::io::prelude::*;
use ricochet_board::*;
use ricochet_solver::*;

fn main() {
    // Erzeugung des Boards
    let board = example_board();

    let mut database = Database(box [ricochet_solver::Entry(255); 1 << 32]);

    // Erzeugung der Positionen
    let mut positions = ask_for_robotpositions();
    // let mut save = File::create("test.json").expect("Create test.json");
    // write!(save, "{}", as_pretty_json(&(&positions, &board))).expect("Write into test.json");

    'outer: loop {
        let target = ask_for_target();
        println!("Solving...");
        let solve = solve(&board, positions, target, database);
        let path = solve.1;
        print!("\u{0007}");
        println!("Steps needed to reach target: {}", path.len());
        println!("Press enter to show path.");
        let _: String = read!("{}\n");
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
        database = Database(box [ricochet_solver::Entry(255); 1 << 32]);
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
    }
}

fn ask_for_target() -> Target {
    let mut target;
    println!("What color is the target?");
    'outer: loop {
        println!("Accepted input: \"red\"(r), \"green\"(g), \"blue\"(b), \"yellow\"(y), \
                  \"spiral\"(s)");
        loop {
            let color: String = read!("{}\n");
            match color.to_lowercase().trim() {
                "red" | "r" => {
                    target = Target::Red(ask_for_symbol());
                    break;
                }
                "green" | "g" => {
                    target = Target::Green(ask_for_symbol());
                    break;
                }
                "blue" | "b" => {
                    target = Target::Blue(ask_for_symbol());
                    break;
                }
                "yellow" | "y" => {
                    target = Target::Yellow(ask_for_symbol());
                    break;
                }
                "spiral" | "s" => {
                    target = Target::Spiral;
                    break;
                }
                _ => println!("Input not accepted: {}", color),
            }
        }
        println!("Please confirm your input.");
        println!("Is the {} the correct target? (Y/n)", target);
        loop {
            let input: String = read!("{}\n");
            match input.to_lowercase().trim() {
                "y" | "" => return target,
                "n" => break,
                _ => println!("Input not accepted! {}", input),
            }
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
    'outer: loop {
        println!("Please input the coordinates of the Robots.\nPlease write in this format: \
                  \"x,y\"");
        for (i, &robot) in [Robot::Red, Robot::Green, Robot::Blue, Robot::Yellow]
                .iter()
                .enumerate() {
            println!("{:?}: ", robot);
            let a: u8;
            let b: u8;
            let pos: String = read!("{}\n");
            scan!(pos.trim().bytes() => "{},{}", a, b);
            positions[i] = (a, b);
        }
        let robopos = RobotPositions::from_array(positions);
        println!("Please confirm your input.");
        println!("{}", robopos);
        println!("Is this correct? (Y/n)");
        loop {
            let input: String = read!("{}\n");
            match input.to_lowercase().trim() {
                "y" | "" => break 'outer,
                "n" => break,
                _ => println!("Input not accepted! {}", input),
            }
        }
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
    board.fields[0][3].bottom = true;
    board.fields[0][11].bottom = true;
    board.fields[2][2].right = true;
    board.fields[2][3].bottom = true;
    board.fields[2][4].right = true;
    board.fields[2][9].bottom = true;
    board.fields[2][10].right = true;
    board.fields[3][1].bottom = true;
    board.fields[3][13].right = true;
    board.fields[4][3].right = true;
    board.fields[4][5].right = true;
    board.fields[4][5].bottom = true;
    board.fields[4][12].bottom = true;
    board.fields[4][15].right = true;
    board.fields[5][0].right = true;
    board.fields[5][3].bottom = true;
    board.fields[5][8].right = true;
    board.fields[5][8].bottom = true;
    board.fields[5][14].right = true;
    board.fields[6][14].bottom = true;
    board.fields[8][3].right = true;
    board.fields[9][0].right = true;
    board.fields[9][2].bottom = true;
    board.fields[9][9].right = true;
    board.fields[9][11].right = true;
    board.fields[9][11].bottom = true;
    board.fields[10][6].right = true;
    board.fields[10][6].bottom = true;
    board.fields[10][9].bottom = true;
    board.fields[10][15].right = true;
    board.fields[11][8].right = true;
    board.fields[12][7].bottom = true;
    board.fields[12][14].right = true;
    board.fields[13][0].bottom = true;
    board.fields[13][1].right = true;
    board.fields[13][6].right = true;
    board.fields[13][13].bottom = true;
    board.fields[14][6].bottom = true;
    board.fields[14][11].bottom = true;
    board.fields[14][12].right = true;
    board.fields[15][4].bottom = true;
    board.fields[15][10].bottom = true;
}

fn set_targets_on_board(board: &mut Board) {
    use ricochet_board::Symbol::*;
    board.targets.insert((Target::Spiral, (12, 8)));
    board.targets.insert((Target::Red(Circle), (9, 3)));
    board.targets.insert((Target::Red(Triangle), (5, 8)));
    board.targets.insert((Target::Red(Square), (2, 4)));
    board.targets.insert((Target::Red(Hexagon), (13, 14)));
    board.targets.insert((Target::Green(Circle), (4, 13)));
    board.targets.insert((Target::Green(Triangle), (13, 1)));
    board.targets.insert((Target::Green(Square), (14, 12)));
    board.targets.insert((Target::Green(Hexagon), (4, 5)));
    board.targets.insert((Target::Blue(Circle), (10, 9)));
    board.targets.insert((Target::Blue(Triangle), (5, 3)));
    board.targets.insert((Target::Blue(Square), (10, 6)));
    board.targets.insert((Target::Blue(Hexagon), (2, 10)));
    board.targets.insert((Target::Yellow(Circle), (3, 2)));
    board.targets.insert((Target::Yellow(Triangle), (9, 11)));
    board.targets.insert((Target::Yellow(Square), (6, 14)));
    board.targets.insert((Target::Yellow(Hexagon), (14, 6)));
}
