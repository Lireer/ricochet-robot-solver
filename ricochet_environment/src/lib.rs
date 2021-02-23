pub(crate) mod builder;

use crate::builder::{EnvironmentBuilder, RobotConfig, TargetConfig, WallConfig};
use pyo3::prelude::*;
use ricochet_board::{
    Board, Direction, PositionEncoding, Robot, RobotPositions, Round, Symbol, Target,
};

/// The base module of the created package.
#[pymodule]
fn ricochet_env(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<RustyEnvironment>()?;

    Ok(())
}

/// The type of a reward which can be obtained by stepping through the environment.
pub type Reward = f64;

/// The type of a coordinate on the board.
pub type Coordinate = (PositionEncoding, PositionEncoding);

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
    Vec<Coordinate>,
    Coordinate,
    usize,
);

/// An action that can be performed in the environment.
///
/// It consists of a robot and the direction the specified robot should move in.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Action {
    robot: Robot,
    direction: Direction,
}

/// A target to reach as part of a round.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TargetColor {
    Red,
    Blue,
    Green,
    Yellow,
    Any,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct RustyEnvironment {
    config: EnvironmentBuilder,
    round: Round,
    wall_observation: (Vec<Vec<bool>>, Vec<Vec<bool>>),
    starting_position: RobotPositions,
    current_position: RobotPositions,
    steps_taken: usize,
}

#[pymethods]
impl RustyEnvironment {
    #[new]
    pub fn new(
        board_size: PositionEncoding,
        walls: WallConfig,
        targets: TargetConfig,
        robots: RobotConfig,
    ) -> Self {
        Self::new_seeded(board_size, walls, targets, robots, rand::random())
    }

    #[staticmethod]
    pub fn new_seeded(
        board_size: PositionEncoding,
        walls: WallConfig,
        targets: TargetConfig,
        robots: RobotConfig,
        seed: u128,
    ) -> Self {
        let mut config = EnvironmentBuilder::new_seeded(board_size, walls, targets, robots, seed);
        let round = config.new_round();
        let starting_position = config.new_positions();

        Self {
            wall_observation: create_wall_bitboards(round.board()),
            round,
            current_position: starting_position.clone(),
            starting_position,
            steps_taken: 0,
            config,
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
        self.round = self.config.new_round();
        if *self.config.walls() != WallConfig::Fix {
            self.wall_observation = create_wall_bitboards(self.round.board());
        }
        self.starting_position = self.config.new_positions();
        self.current_position = self.starting_position.clone();
        self.steps_taken = 0;

        self.get_state(py_gil)
    }

    pub fn render(&self) -> String {
        ricochet_board::draw_board(self.round.board().get_walls())
    }

    pub fn get_state(&self, py_gil: Python) -> PyObject {
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
                "failed to convert value {} into an action. Only values in [0:16] are valid.",
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

impl<'source> FromPyObject<'source> for TargetColor {
    fn extract(raw_target: &'source PyAny) -> PyResult<Self> {
        let target = match raw_target.extract()? {
            0 => TargetColor::Red,
            1 => TargetColor::Blue,
            2 => TargetColor::Green,
            3 => TargetColor::Yellow,
            4 => TargetColor::Any,
            i => panic!(
                "could not convert value {} into a target. Only values in [0:4] are valid.",
                i
            ),
        };
        Ok(target)
    }
}

impl From<TargetColor> for Target {
    fn from(tc: TargetColor) -> Self {
        match tc {
            TargetColor::Red => Target::Red(Symbol::Circle),
            TargetColor::Blue => Target::Blue(Symbol::Circle),
            TargetColor::Green => Target::Green(Symbol::Circle),
            TargetColor::Yellow => Target::Yellow(Symbol::Circle),
            TargetColor::Any => Target::Spiral,
        }
    }
}

impl From<Target> for TargetColor {
    fn from(target: Target) -> Self {
        match target {
            Target::Red(_) => TargetColor::Red,
            Target::Blue(_) => TargetColor::Blue,
            Target::Green(_) => TargetColor::Green,
            Target::Yellow(_) => TargetColor::Yellow,
            Target::Spiral => TargetColor::Any,
        }
    }
}

fn robot_positions_as_vec(pos: &RobotPositions) -> Vec<Coordinate> {
    pos.to_array()
        .iter()
        .map(|p| (p.column(), p.row()))
        .collect()
}

/// Creates two bitboards with the same dimensions as `self`.
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
