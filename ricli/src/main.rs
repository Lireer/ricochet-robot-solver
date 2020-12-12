use std::collections::HashSet;
use std::iter::FromIterator;
use text_io::{read, try_scan};

use ricochet_board::{
    template, Board, Color, PositionEncoding, RobotPositions, Symbol, Target, BOARDSIZE,
};
use ricochet_solver::solve;

fn main() {
    // Create the board
    let board = 'outer: loop {
        let board = build_board_from_parts();
        println!("Please confirm your input.");
        println!("Is this the correct board? (Y/n)\n{:?}", board);
        loop {
            let input: String = read!("{}\n");
            match input.to_lowercase().trim() {
                "y" | "" => break 'outer board,
                "n" => break,
                _ => println!("Input invalid! {}", input),
            }
        }
    };

    // Ask the user where the robots are positioned
    let mut positions = ask_for_robot_positions();

    'game: loop {
        let target = ask_for_target();
        println!("Solving...");
        let solve = solve(&board, positions, target);
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
                "n" => break 'game,
                _ => println!("Input invalid! {}", input),
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
        for (i, &robot) in [Color::Red, Color::Blue, Color::Green, Color::Yellow]
            .iter()
            .enumerate()
        {
            println!("{:?}: ", robot);
            loop {
                let pos: String = read!("{}\n");
                match parse_robot_position(pos) {
                    Ok((col, row))
                        if (1..=BOARDSIZE).contains(&col) || (1..=BOARDSIZE).contains(&row) =>
                    {
                        positions[i] = (col - 1, row - 1);
                        break;
                    }
                    _ => println!("Input invalid"),
                }
            }
        }
        let robopos = RobotPositions::from_array(&positions);
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
    RobotPositions::from_array(&positions)
}

fn parse_robot_position(
    pos: String,
) -> Result<(PositionEncoding, PositionEncoding), text_io::Error> {
    let col: PositionEncoding;
    let row: PositionEncoding;
    try_scan!(pos.trim().bytes() => "{},{}", col, row);
    Ok((col, row))
}

fn build_board_from_parts() -> Board {
    let templates = template::gen_templates();

    let orientation = [
        template::Orientation::UpperLeft,
        template::Orientation::UpperRight,
        template::Orientation::BottomRight,
        template::Orientation::BottomLeft,
    ];

    let mut possible_colors: HashSet<template::TempColor> = HashSet::from_iter(
        [
            template::TempColor::Red,
            template::TempColor::Blue,
            template::TempColor::Green,
            template::TempColor::Yellow,
        ]
        .iter()
        .cloned(),
    );

    let mut board_parts = Vec::new();

    for orient in orientation.iter() {
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
                "red" | "r" if possible_colors.get(&template::TempColor::Red) != None => {
                    color = template::TempColor::Red;
                    break;
                }
                "blue" | "b" if possible_colors.get(&template::TempColor::Blue) != None => {
                    color = template::TempColor::Blue;
                    break;
                }
                "green" | "g" if possible_colors.get(&template::TempColor::Green) != None => {
                    color = template::TempColor::Green;
                    break;
                }
                "yellow" | "y" if possible_colors.get(&template::TempColor::Yellow) != None => {
                    color = template::TempColor::Yellow;
                    break;
                }
                _ => println!("Input invalid! {}", col),
            }
        }

        println!("Which of these parts is it? (1, 2, 3)");
        let mut temps: Vec<template::BoardTemplate> = templates
            .iter()
            .filter(|t| t.color() == color)
            .cloned()
            .collect();

        temps.iter_mut().for_each(|temp| temp.rotate_to(*orient));

        for (i, temp) in temps.iter().enumerate() {
            println!("{}.\n{}", i + 1, temp);
        }

        loop {
            let input: String = read!("{}\n");
            match input.trim().to_lowercase().parse::<usize>() {
                // TODO: make the limit of 3 dependant on the actual length of temps
                Ok(i) if (1..=3).contains(&i) => {
                    board_parts.push(temps.get(i - 1).unwrap().clone())
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
    Board::from_templates(&board_parts)
}