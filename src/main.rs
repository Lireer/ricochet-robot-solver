use text_io::{read, try_read, try_scan};

use ricochet_board::{Board, Field, Robot, RobotPosition, Symbol, Target, BOARDSIZE};
use ricochet_solver::{solve, Database};

fn main() {
    // Erzeugung des Boards
    let board = example_board();

    // Print board
    // println!("{:?}", board);

    let mut database = Database::new();

    // Erzeugung der Positionen
    let mut positions = ask_for_robot_positions();
    // let mut save = File::create("test.json").expect("Create test.json");
    // write!(save, "{}", as_pretty_json(&(&positions, &board))).expect("Write into test.json");

    'outer: loop {
        let target = ask_for_target();
        println!("Solving...");
        let solve = solve(&board, positions, target, database);
        let path = solve.1;
        println!("Steps needed to reach target: {}", path.len());
        println!("Press enter to show path.");
        let _: String = read!("{}\n");
        println!("Step Robot   Direction");
        for (step, (robot, dir)) in path.iter().enumerate() {
            println!(" {:>2}  {:<8}{:<6}", step + 1, robot, dir);
        }
        println!("Continue? (Y/n)");

        loop {
            let input: String = read!("{}\n");
            match input.to_lowercase().trim() {
                "y" | "" => break,
                "n" => break 'outer,
                _ => println!("Input invalid! {}", input),
            }
        }

        database = Database::new();
        println!("Is the end position the new starting position? (Y/n)");
        loop {
            let input: String = read!("{}\n");
            match input.to_lowercase().trim() {
                "y" | "" => {
                    positions = solve.0;
                    break;
                }
                "n" => {
                    positions = ask_for_robot_positions();
                    break;
                }
                _ => println!("Input invalid! {}", input),
            }
        }
    }
}

fn ask_for_target() -> Target {
    let mut target;
    println!("What color is the target?");
    loop {
        println!(
            "Accepted input: \"red\"(r), \"green\"(g), \"blue\"(b), \"yellow\"(y), \
             \"spiral\"(s)"
        );
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
                _ => println!("Input invalid! {}", color),
            }
        }
        println!("Please confirm your input.");
        println!("Is the {} the correct target? (Y/n)", target);
        loop {
            let input: String = read!("{}\n");
            match input.to_lowercase().trim() {
                "y" | "" => return target,
                "n" => break,
                _ => println!("Input invalid! {}", input),
            }
        }
    }
}

fn ask_for_symbol() -> Symbol {
    println!("What is the shape of the target?");
    println!("Accepted input: \"Circle\"(c), \"Triangle\"(t), \"Square\"(s), \"Hexagon\"(h)");
    loop {
        let shape: String = read!("{}\n");
        match shape.to_lowercase().trim() {
            "circle" | "c" => return Symbol::Circle,
            "triangle" | "t" => return Symbol::Triangle,
            "square" | "s" => return Symbol::Square,
            "hexagon" | "h" => return Symbol::Hexagon,
            _ => println!("Input invalid: {}", shape),
        }
    }
}

fn ask_for_robot_positions() -> RobotPosition {
    let mut positions = [(0, 0); 4];
    'outer: loop {
        println!(
            "Please input the coordinates of the Robots.\nPlease write in this format: \
             \"column,row\""
        );
        for (i, &robot) in [Robot::Red, Robot::Green, Robot::Blue, Robot::Yellow]
            .iter()
            .enumerate()
        {
            println!("{:?}: ", robot);
            loop {
                let pos: String = read!("{}\n");
                match parse_robot_position(pos) {
                    Ok((a, b)) if (a as usize) < BOARDSIZE || (b as usize) < BOARDSIZE => {
                        positions[i] = (a, b);
                        break;
                    }
                    _ => println!("Input invalid"),
                }
            }
        }
        let robopos = RobotPosition::from_array(positions);
        println!("Please confirm your input.");
        println!("{}", robopos);
        println!("Is this correct? (Y/n)");
        loop {
            let input: String = read!("{}\n");
            match input.to_lowercase().trim() {
                "y" | "" => break 'outer,
                "n" => break,
                _ => println!("Input invalid! {}", input),
            }
        }
    }
    RobotPosition::from_array(positions)
}

fn parse_robot_position(pos: String) -> Result<(u8, u8), text_io::Error> {
    let a: u8;
    let b: u8;
    try_scan!(pos.trim().bytes() => "{},{}", a, b);
    Ok((a, b))
}

fn example_board() -> Board {
    let mut board = default_board();
    fill_board_with_walls(&mut board); // Set walls on example board
    set_targets_on_board(&mut board); // Set targets on example board
    board
}

fn default_board() -> Board {
    let board = Board {
        fields: [[Field {
            bottom: false,
            right: false,
        }; BOARDSIZE]; BOARDSIZE],
        targets: Default::default(),
    };
    board
        .wall_enclosure() // Set outer walls
        .set_center_walls() // Set walls around the four center fields
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
