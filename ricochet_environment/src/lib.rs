use pyo3::prelude::*;
use ricochet_board::{
    template, Board, Direction, Game, PositionEncoding, Robot, RobotPositions, Round, Symbol,
    Target,
};

/// The type of a reward which can be obtained by stepping through the environment.
pub type Reward = f64;

/// The observation of the state of an environment.
///
/// The tuple consists of
/// - the board with all fields set to true that have a wall to the right
/// - the board with all fields set to true that have a wall at the bottom
/// - the positions of the robots in the order red, blue, green, yellow as (column, row) tuples
/// - the position of the target
/// - the color of the target
pub type Observation<'a> = (
    &'a Vec<Vec<bool>>,
    &'a Vec<Vec<bool>>,
    Vec<(PositionEncoding, PositionEncoding)>,
    (PositionEncoding, PositionEncoding),
    usize,
);

/// The base module of the created package.
#[pymodule]
fn ricochet_env(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<RustyEnvironment>()?;

    Ok(())
}

/// An action that can be performed in the environment.
///
/// It consists of a robot and the direction the specified robot should move in.
#[derive(Debug, Copy, Clone)]
pub struct Action {
    robot: Robot,
    direction: Direction,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct RustyEnvironment {
    round: Round,
    wall_observation: (Vec<Vec<bool>>, Vec<Vec<bool>>),
    starting_position: RobotPositions,
    current_position: RobotPositions,
    steps_taken: usize,
}

#[pymethods]
impl RustyEnvironment {
    #[new]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let templates = template::gen_templates()
            .iter()
            .step_by(3)
            .cloned()
            .enumerate()
            .map(|(i, mut temp)| {
                temp.rotate_to(template::ORIENTATIONS[i]);
                temp
            })
            .collect::<Vec<template::BoardTemplate>>();

        let game = Game::from_templates(&templates);
        let target = Target::Red(Symbol::Triangle);
        let starting_position = RobotPositions::from_tuples(&[(0, 1), (5, 4), (7, 1), (7, 15)]);

        Self {
            round: Round::new(
                game.board().clone(),
                target,
                game.get_target_position(&target).unwrap(),
            ),
            wall_observation: create_wall_bitboards(game.board()),
            current_position: starting_position.clone(),
            starting_position,
            steps_taken: 0,
        }
    }

    pub fn step(&mut self, py_gil: Python, action: Action) -> PyObject {
        self.current_position = self.current_position.clone().move_in_direction(
            self.round.board(),
            action.robot,
            action.direction,
        );

        let mut reward = 0.0;
        let mut done = false;
        if self.round.target_reached(&self.current_position) {
            reward = 1.0;
            done = true;
        }

        let output = (self.observation(), reward, done);
        output.to_object(py_gil)
    }

    pub fn reset(&mut self, py_gil: Python) -> PyObject {
        self.current_position = self.starting_position.clone();
        self.steps_taken = 0;
        self.observation().to_object(py_gil)
    }
}

impl RustyEnvironment {
    fn observation(&self) -> Observation {
        let target_pos = self.round.target_position();
        let target = match self.round.target() {
            Target::Red(_) => 0,
            Target::Blue(_) => 1,
            Target::Green(_) => 2,
            Target::Yellow(_) => 3,
            Target::Spiral => 4,
        };
        (
            &self.wall_observation.0,
            &self.wall_observation.1,
            robot_positions_as_vec(&self.current_position),
            (target_pos.column(), target_pos.row()),
            target,
        )
    }
}

impl Action {
    pub fn new(robot: Robot, direction: Direction) -> Self {
        Self { robot, direction }
    }

    pub fn robot(&self) -> Robot {
        self.robot
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }
}

impl<'source> FromPyObject<'source> for Action {
    fn extract(raw_action: &'source PyAny) -> PyResult<Self> {
        let action = raw_action.extract::<usize>()?;
        let robot = match action / 4 {
            0 => Robot::Red,
            1 => Robot::Blue,
            2 => Robot::Green,
            3 => Robot::Yellow,
            _ => panic!(
                "failed to convert value {} to an action. Only values in [0:16] are valid.",
                action
            ),
        };
        let direction = match action % 4 {
            0 => Direction::Up,
            1 => Direction::Right,
            2 => Direction::Down,
            3 => Direction::Left,
            _ => unreachable!(),
        };
        Ok(Self::new(robot, direction))
    }
}

fn robot_positions_as_vec(pos: &RobotPositions) -> Vec<(PositionEncoding, PositionEncoding)> {
    pos.to_array()
        .iter()
        .map(|p| (p.column(), p.row()))
        .collect()
}

/// Creates two bitboards with the same format as `self`.
///
/// The first board in the returned tuple contains all walls, which are to the right of a field.
/// The second board contains all walls, which are in the down direction of a field.
fn create_wall_bitboards(board: &Board) -> (Vec<Vec<bool>>, Vec<Vec<bool>>) {
    let size = board.side_length() as usize;
    let mut right_board = vec![vec![false; size]; size];
    let mut down_board = right_board.clone();
    for col in 0..size {
        for row in 0..size {
            let field = &board.get_walls()[col][row];
            right_board[col][row] = field.right;
            down_board[col][row] = field.down;
        }
    }
    (right_board, down_board)
}
