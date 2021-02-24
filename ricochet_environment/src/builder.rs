use crate::{Coordinate, TargetColor};
use getset::{Getters, CopyGetters};
use pyo3::{FromPyObject, PyAny, PyResult};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use ricochet_board::generator::{Generator as BoardGenerator, CENTER_WALLS_FROM_SIDE_LENGTH};
use ricochet_board::template::DISTINCT_STANDARD_BOARDS;
use ricochet_board::{template, PositionEncoding, RobotPositions, Round};

/// Seed used to generate boards.
///
/// Used with `WallConfig::Fix` and every seed used in `WallConfig::Variants` is a multiple of this
/// seed.
const WALLS_SEED: u128 = 0xcafef00dd15ea5e5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WallConfig {
    /// A fixed board is generated.
    Fix,
    /// A board is randomly chosen from a finite set.
    ///
    /// The number of possible boards is given by the `usize`. If the number equals
    /// [`DISTINCT_STANDARD_BOARDS`](DISTINCT_STANDARD_BOARDS) and `board_size == 16`, a board made
    /// from standard templates will be generated.
    Variants(usize),
    /// A randomly generated board from a practically infinte set.
    Random,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetConfig {
    FromList(Vec<(TargetColor, Coordinate)>),
    Variants,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RobotConfig {
    Fix(RobotPositions),
    Random,
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters)]
pub struct EnvironmentBuilder {
    #[get_copy = "pub"]
    board_size: PositionEncoding,
    #[get = "pub"]
    walls: WallConfig,
    #[get = "pub"]
    targets: TargetConfig,
    #[get = "pub"]
    robots: RobotConfig,
    rng: rand_pcg::Pcg64Mcg,
}

impl EnvironmentBuilder {
    pub fn new_seeded(
        board_size: PositionEncoding,
        walls: WallConfig,
        targets: TargetConfig,
        robots: RobotConfig,
        seed: u128,
    ) -> Self {
        Self {
            board_size,
            walls,
            targets,
            robots,
            rng: rand_pcg::Pcg64Mcg::new(seed),
        }
    }

    /// Creates a new `Round`.
    pub fn new_round(&mut self) -> Round {
        let game = match self.walls {
            WallConfig::Fix => {
                BoardGenerator::from_seed(WALLS_SEED, self.board_size).generate_game()
            }
            WallConfig::Variants(DISTINCT_STANDARD_BOARDS) if self.board_size == 16 => {
                template::game_from_seed(self.rng.gen_range(0..DISTINCT_STANDARD_BOARDS))
            }
            WallConfig::Variants(n) => BoardGenerator::from_seed(
                WALLS_SEED * self.rng.gen_range(0..n) as u128,
                self.board_size,
            )
            .generate_game(),
            WallConfig::Random => BoardGenerator::new(self.board_size).generate_game(),
        };

        let (target, target_position) = match &self.targets {
            TargetConfig::FromList(targets) => {
                let (t, tp) = *targets.choose(&mut self.rng).expect("target list is empty");
                (t.into(), tp.into())
            }
            TargetConfig::Variants => game
                .targets()
                .iter()
                .collect::<Vec<_>>()
                .choose(&mut self.rng)
                .map(|&(&t, &tp)| (t, tp))
                .expect("could not get a target from a `Game`"),
        };

        Round::new(game.board().clone(), target, target_position)
    }

    /// Creates a new `RobotPositions`.
    pub fn new_positions(&mut self) -> RobotPositions {
        match &self.robots {
            RobotConfig::Fix(pos) => pos.clone(),
            RobotConfig::Random => loop {
                let rng = &mut self.rng;
                let range = 0..self.board_size;
                let mut new_position =
                    || (rng.gen_range(range.clone()), rng.gen_range(range.clone()));
                let pos = [
                    new_position(),
                    new_position(),
                    new_position(),
                    new_position(),
                ];

                // Make sure no robot is confined inside the center walls.
                if self.board_size >= CENTER_WALLS_FROM_SIDE_LENGTH {
                    let start = self.board_size / 2 - 1;
                    let end = start + 1;
                    if pos
                        .iter()
                        .any(|(c, r)| (start..=end).contains(c) && (start..=end).contains(r))
                    {
                        continue;
                    }
                }

                break RobotPositions::from_tuples(&pos);
            },
        }
    }
}

impl Default for EnvironmentBuilder {
    fn default() -> Self {
        Self {
            board_size: 16,
            walls: WallConfig::Variants(486),
            targets: TargetConfig::Variants,
            robots: RobotConfig::Random,
            rng: rand_pcg::Pcg64Mcg::from_entropy(),
        }
    }
}

// Following code provides implementations to extract configs from `PyObject`s

impl<'source> FromPyObject<'source> for WallConfig {
    fn extract(raw_conf: &'source PyAny) -> PyResult<Self> {
        if let Ok(variant_count) = raw_conf.extract::<usize>() {
            return Ok(WallConfig::Variants(variant_count));
        }
        match raw_conf.extract::<&str>()?.to_lowercase().as_ref() {
            "fixed" => Ok(WallConfig::Fix),
            "variants" => Ok(WallConfig::Variants(DISTINCT_STANDARD_BOARDS)),
            "random" => Ok(WallConfig::Random),
            text => panic!("unknown target configuration \"{}\"", text),
        }
    }
}

impl<'source> FromPyObject<'source> for TargetConfig {
    fn extract(raw_conf: &'source PyAny) -> PyResult<Self> {
        if let Ok(conf) = raw_conf.extract::<String>() {
            if conf.to_lowercase() != "variants" {
                panic!("unknown target configuration \"{}\"", conf);
            }
            return Ok(TargetConfig::Variants);
        }
        match raw_conf.extract::<Vec<(TargetColor, Coordinate)>>() {
            Ok(list) if list.is_empty() => {
                panic!("received an empty list of targets, at least one target has to be set")
            }
            Ok(list) => Ok(TargetConfig::FromList(list)),
            Err(_) => panic!(
                "could not convert value {} to a target configuration",
                raw_conf
            ),
        }
    }
}

impl<'source> FromPyObject<'source> for RobotConfig {
    fn extract(raw_conf: &'source PyAny) -> PyResult<Self> {
        if let Ok(conf) = raw_conf.extract::<String>() {
            if conf.to_lowercase() != "random" {
                panic!("unknown robot configuration \"{}\"", conf);
            }
            return Ok(RobotConfig::Random);
        }
        match raw_conf.extract::<Vec<Coordinate>>() {
            Ok(list) if list.len() == 4 => {
                let mut tuples = [(0, 0); 4];
                for (i, coord) in list.iter().enumerate() {
                    tuples[i] = *coord;
                }
                Ok(RobotConfig::Fix(RobotPositions::from_tuples(&tuples)))
            }
            _ => panic!(
                "could not convert value {} into a robot configuration",
                raw_conf
            ),
        }
    }
}
