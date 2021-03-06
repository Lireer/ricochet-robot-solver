use std::collections::HashSet;
use text_io::{read, try_scan};

use ricochet_board::{
    quadrant, Game, PositionEncoding, Robot, RobotPositions, Round, Symbol, Target,
};
use ricochet_solver::{IdaStar, Solver};

const BOARD_SIZE: PositionEncoding = quadrant::STANDARD_BOARD_SIZE;

fn main() {
    // Create the board
    let game = 'outer: loop {
        let game = build_board_from_parts();
        println!("Please confirm your input.");
        println!("Is this the correct board? (Y/n)\n{:?}", game.board());
        loop {
            let input: String = read!("{}\n");
            match input.to_lowercase().trim() {
                "y" | "" => break 'outer game,
                "n" => break,
                _ => println!("Input invalid! {}", input),
            }
        }
    };

    // Ask the user where the robots are positioned
    let mut positions = ask_for_robot_positions();

    'game: loop {
        let target = ask_for_target();
        let target_position = game
            .get_target_position(&target)
            .expect("Failed to find the position of the target on the board");
        let round = Round::new(game.board().clone(), target, target_position);

        println!("Solving...");
        let path = IdaStar::new().solve(&round, positions);
        let movements = path.movements();
        println!("Moves needed to reach target: {}", movements.len());
        println!("Press enter to show path.");
        let _: String = read!("{}\n");
        println!("Move Robot   Direction");
        for (move_n, (robot, dir)) in movements.iter().enumerate() {
            println!(" {:>2}  {:<8}{:<6}", move_n + 1, robot, dir);
        }
        println!("Continue? (Y/n)");

        loop {
            let input: String = read!("{}\n");
            match input.to_lowercase().trim() {
                "y" | "" => break,
                "n" => break 'game,
                _ => println!("Input invalid! {}", input),
            }
        }

        println!("Is the end position the new starting position? (Y/n)");
        loop {
            let input: String = read!("{}\n");
            match input.to_lowercase().trim() {
                "y" | "" => {
                    positions = path.end_pos().clone();
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
            "Accepted input: \"red\"(r), \"blue\"(b), \"green\"(g), \"yellow\"(y), \"spiral\"(s)"
        );
        loop {
            let color: String = read!("{}\n");
            match color.to_lowercase().trim() {
                "red" | "r" => {
                    target = Target::Red(ask_for_symbol());
                    break;
                }
                "blue" | "b" => {
                    target = Target::Blue(ask_for_symbol());
                    break;
                }
                "green" | "g" => {
                    target = Target::Green(ask_for_symbol());
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

fn ask_for_robot_positions() -> RobotPositions {
    let mut positions = [(0, 0); 4];
    'outer: loop {
        println!(
            "Please input the coordinates of the Robots.\nPlease write in this format: \
             \"column,row\""
        );
        for (i, &robot) in [Robot::Red, Robot::Blue, Robot::Green, Robot::Yellow]
            .iter()
            .enumerate()
        {
            println!("{:?}: ", robot);
            loop {
                let pos: String = read!("{}\n");
                match parse_robot_position(pos) {
                    Ok((col, row))
                        if (1..=BOARD_SIZE).contains(&col) || (1..=BOARD_SIZE).contains(&row) =>
                    {
                        positions[i] = (col - 1, row - 1);
                        break;
                    }
                    _ => println!("Input invalid"),
                }
            }
        }
        let robopos = RobotPositions::from_tuples(&positions);
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
    RobotPositions::from_tuples(&positions)
}

fn parse_robot_position(
    pos: String,
) -> Result<(PositionEncoding, PositionEncoding), text_io::Error> {
    let col: PositionEncoding;
    let row: PositionEncoding;
    try_scan!(pos.trim().bytes() => "{},{}", col, row);
    Ok((col, row))
}

fn build_board_from_parts() -> Game {
    let quadrants = quadrant::gen_quadrants();

    let mut possible_colors: HashSet<quadrant::QuadColor> = [
        quadrant::QuadColor::Red,
        quadrant::QuadColor::Blue,
        quadrant::QuadColor::Green,
        quadrant::QuadColor::Yellow,
    ]
    .iter()
    .cloned()
    .collect();

    let mut board_parts = Vec::new();

    for orient in quadrant::ORIENTATIONS.iter() {
        println!(
            "What color is the {} board part? You can find the color near the center.",
            orient
        );
        let mut accept_inputs = "Accepted input:".to_string();
        for pc in &possible_colors {
            accept_inputs = format!("{} {}", accept_inputs, &pc.to_string());
        }
        println!("{}?", accept_inputs);
        let color;
        loop {
            let col: String = read!("{}\n");
            match col.to_lowercase().trim() {
                "red" | "r" if possible_colors.get(&quadrant::QuadColor::Red) != None => {
                    color = quadrant::QuadColor::Red;
                    break;
                }
                "blue" | "b" if possible_colors.get(&quadrant::QuadColor::Blue) != None => {
                    color = quadrant::QuadColor::Blue;
                    break;
                }
                "green" | "g" if possible_colors.get(&quadrant::QuadColor::Green) != None => {
                    color = quadrant::QuadColor::Green;
                    break;
                }
                "yellow" | "y" if possible_colors.get(&quadrant::QuadColor::Yellow) != None => {
                    color = quadrant::QuadColor::Yellow;
                    break;
                }
                _ => println!("Input invalid! {}", col),
            }
        }

        println!("Which of these parts is it? (1, 2, 3)");
        let mut quads: Vec<quadrant::BoardQuadrant> = quadrants
            .iter()
            .filter(|t| t.color() == color)
            .cloned()
            .collect();

        quads.iter_mut().for_each(|quad| quad.rotate_to(*orient));

        for (i, quad) in quads.iter().enumerate() {
            println!("{}.\n{}", i + 1, quad);
        }

        loop {
            let input: String = read!("{}\n");
            match input.trim().to_lowercase().parse::<usize>() {
                // TODO: make the limit of 3 dependant on the actual length of quads
                Ok(i) if (1..=3).contains(&i) => {
                    board_parts.push(quads.get(i - 1).unwrap().clone())
                }
                _ => {
                    println!("Input invalid!");
                    continue;
                }
            };
            break;
        }

        possible_colors.retain(|&c| c != color);
    }

    // Create a board from the parts
    Game::from_quadrants(&board_parts)
}
