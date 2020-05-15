use ricochet_board::*;

fn create_board() -> (RobotPositions, Board) {
    const ORIENTATIONS: [template::Orientation; 4] = [
        template::Orientation::UpperLeft,
        template::Orientation::UpperRight,
        template::Orientation::BottomRight,
        template::Orientation::BottomLeft,
    ];
    const TEMPS_PER_COLOR: usize = 3;

    let templates = template::gen_templates();
    let templates = [
        templates[0 * TEMPS_PER_COLOR].clone(),
        templates[1 * TEMPS_PER_COLOR].clone(),
        templates[2 * TEMPS_PER_COLOR].clone(),
        templates[3 * TEMPS_PER_COLOR].clone(),
    ]
    .iter()
    .cloned()
    .enumerate()
    .map(|(i, mut temp)| {
        temp.rotate_to(ORIENTATIONS[i]);
        temp
    })
    .collect::<Vec<template::BoardTemplate>>();

    let pos = RobotPositions::from_array(&[(0, 1), (5, 4), (7, 1), (7, 15)]);
    (pos, Board::from_templates(&templates))
}

#[test]
fn board_creation() {
    create_board();
}

#[test]
fn move_right() {
    let (mut positions, board) = create_board();
    assert_eq!(positions.green(), Position::from_tuple((7, 1)));
    positions.move_in_direction(&board, Color::Green, Direction::Right);
    assert_eq!(positions.green(), Position::from_tuple((15, 1)));
}

#[test]
fn move_left() {
    let (mut positions, board) = create_board();
    assert_eq!(positions.green(), Position::from_tuple((7, 1)));
    positions.move_in_direction(&board, Color::Green, Direction::Left);
    assert_eq!(positions.green(), Position::from_tuple((5, 1)));
}

#[test]
fn move_up() {
    let (mut positions, board) = create_board();
    assert_eq!(positions.green(), Position::from_tuple((7, 1)));
    positions.move_in_direction(&board, Color::Green, Direction::Up);
    assert_eq!(positions.green(), Position::from_tuple((7, 0)));
}

#[test]
fn move_down() {
    let (mut positions, board) = create_board();
    assert_eq!(positions.green(), Position::from_tuple((7, 1)));
    positions.move_in_direction(&board, Color::Green, Direction::Down);
    assert_eq!(positions.green(), Position::from_tuple((7, 6)));
}
