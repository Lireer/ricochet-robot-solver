use chrono::Local;
use float_ord::FloatOrd;
use fxhash::FxBuildHasher;
use getset::Getters;
use indexmap::IndexSet;
use rand::SeedableRng;
use ricochet_board::{Direction, Robot, RobotPositions, Round};
use std::collections::HashMap;

use crate::{Path, Solver};

type NodeMap = HashMap<RobotPositions, NodeData, FxBuildHasher>;

#[derive(Debug, Clone, Getters, PartialEq, Eq)]
#[getset(get = "pub")]
struct NodeData {
    position: RobotPositions,
    visits: usize,
    score_sum: u64,
}

impl NodeData {
    pub fn new(position: RobotPositions) -> Self {
        Self {
            position,
            visits: 1,
            score_sum: 0,
        }
    }

    pub fn children(&self, round: &Round) -> Vec<(RobotPositions, (Robot, Direction))> {
        if round.target_reached(&self.position) {
            Vec::new()
        } else {
            self.position.reachable_positions(round.board()).collect()
        }
    }

    pub fn is_terminal(&self, round: &Round) -> bool {
        round.target_reached(&self.position)
    }

    pub fn mean_score(&self) -> f64 {
        self.score_sum as f64 / self.visits as f64
    }

    pub fn update_score(&mut self, score: u64) {
        self.visits += 1;
        self.score_sum += score;
    }
}

/// Solver using Monte Carlo Tree Search (MCTS).
#[derive(Debug)]
pub struct Mcts {
    time_per_move: chrono::Duration,
    exploration_weight: f64,
    num_rollouts: usize,
    nodes: NodeMap,
    seed: u64,
}

impl Mcts {
    pub fn new(time_per_move: chrono::Duration) -> Self {
        Self::new_seeded(time_per_move, rand::random())
    }

    pub fn new_seeded(time_per_move: chrono::Duration, seed: u64) -> Self {
        Self {
            time_per_move,
            exploration_weight: 0.5,
            num_rollouts: 5,
            nodes: HashMap::with_capacity_and_hasher(65536, Default::default()),
            seed,
        }
    }

    fn choose_best_child(
        &self,
        of_node: &RobotPositions,
        round: &Round,
        rng: &mut impl rand::Rng,
    ) -> (RobotPositions, (Robot, Direction)) {
        assert!(!round.target_reached(of_node));
        assert!(self.nodes.get(of_node).is_some());
        let children = self.nodes.get(of_node).unwrap().children(round);
        let min = children
            .iter()
            .filter(|(pos, _)| self.nodes.contains_key(pos))
            .min_by_key(|(pos, _)| FloatOrd(self.nodes.get(pos).unwrap().mean_score()));
        match min {
            Some(min) => min.clone(),
            None => {
                // choose a random move
                children[rng.gen_range(0..children.len())].clone()
            }
        }
    }

    /// Perform the selection step and return a path in form of a vec containing the node indices.
    fn selection(&mut self, start: &RobotPositions, round: &Round) -> Vec<RobotPositions> {
        let mut path: IndexSet<RobotPositions> = IndexSet::with_capacity(1024);
        path.insert(start.clone());

        let mut current_node = start.clone();
        loop {
            // Check if node is unexplored or target has been reached.
            let node_data = match self.nodes.get(&current_node) {
                None => break,
                Some(data) if data.is_terminal(round) => break,
                Some(data) => data,
            };

            // First explore any unknown child node.
            let children = node_data.children(round);
            let unexplored_child = children
                .iter()
                .find(|&child| !self.nodes.contains_key(&child.0));
            if let Some(child) = unexplored_child {
                current_node = child.0.clone();
                path.insert(current_node.clone());
                continue;
            }

            // Continue with the node with the highest uct score.
            current_node = node_data
                .children(round)
                .iter()
                .filter_map(|(pos, _)| self.nodes.get(&pos))
                .filter(|&data| !path.contains(&data.position))
                .max_by_key(|&data| FloatOrd(self.uct_score(&data, node_data.visits)))
                .map(|data| data.position.clone())
                .expect("Ran into a dead end during selection");

            path.insert(current_node.clone());
        }

        path.iter().cloned().collect()
    }

    fn expansion(&mut self, pos: &RobotPositions) {
        if !self.nodes.contains_key(pos) {
            self.nodes.insert(pos.clone(), NodeData::new(pos.clone()));
        }
    }

    fn simulation(&self, from: &RobotPositions, round: &Round, rng: &mut impl rand::Rng) -> u64 {
        let mut moves = 0;
        let mut current_pos = from.clone();
        while !round.target_reached(&current_pos) {
            let mut reachable = current_pos
                .reachable_positions(round.board())
                .map(|(pos, _)| pos)
                .collect::<Vec<_>>();
            current_pos = reachable.swap_remove(rng.gen_range(0..reachable.len()));
            moves += 1;
        }
        moves
    }

    fn backpropagation(&mut self, path: Vec<RobotPositions>, length: u64) {
        for (i, pos) in path.iter().enumerate() {
            let data = self.nodes.get_mut(&pos).unwrap();
            // path from leaf + number of moves from `pos` to leaf
            data.update_score(length + path.len() as u64 - 1 - i as u64)
        }
    }

    /// Perform selection, expansion, simulation and backpropagation once.
    fn run(&mut self, current_root: &RobotPositions, round: &Round, rng: &mut impl rand::Rng) {
        let leaf_path = self.selection(current_root, round);
        let leaf = leaf_path.last().unwrap().clone();
        self.expansion(&leaf);
        let mut length = u64::MAX;
        for _ in 0..self.num_rollouts {
            length = length.min(self.simulation(&leaf, round, rng));
        }
        self.backpropagation(leaf_path, length);
    }

    /// Calculates the uct score of a node using the negative mean score, since lower scores are
    /// better.
    fn uct_score(&self, node_data: &NodeData, mut parent_visits: usize) -> f64 {
        if parent_visits == 0 {
            parent_visits = 1
        };
        (node_data.mean_score() * -1.0)
            + self.exploration_weight
                * f64::sqrt(f64::ln(parent_visits as f64) / node_data.visits as f64)
    }
}

impl Solver for Mcts {
    fn solve(&mut self, round: &Round, start_positions: RobotPositions) -> Path {
        let mut rng = rand::rngs::StdRng::seed_from_u64(self.seed);
        let mut current_pos = start_positions.clone();
        let mut movements = Vec::new();

        while !round.target_reached(&current_pos) {
            let move_start = Local::now();

            while Local::now() - move_start <= self.time_per_move {
                self.run(&current_pos, round, &mut rng);
            }

            let (new_pos, movement) = self.choose_best_child(&current_pos, round, &mut rng);
            movements.push(movement);
            current_pos = new_pos;
        }

        Path::new(start_positions, current_pos, movements)
    }
}

#[cfg(test)]
mod tests {
    use rand::{Rng, SeedableRng};
    use ricochet_board::*;

    use crate::{Mcts, Path, Solver};

    fn create_board() -> (RobotPositions, Game) {
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

        let pos = RobotPositions::from_tuples(&[(0, 1), (5, 4), (7, 1), (7, 15)]);
        (pos, Game::from_templates(&templates))
    }

    // Test robot already on target
    #[test]
    fn on_target() {
        let (_, game) = create_board();
        let target = Target::Green(Symbol::Triangle);
        let target_position = game.get_target_position(&target).unwrap();

        let start = RobotPositions::from_tuples(&[(0, 1), (5, 4), target_position.into(), (7, 15)]);
        let end = start.clone();

        let round = Round::new(game.board().clone(), target, target_position);

        let expected = Path::new(start.clone(), end, vec![]);
        assert_eq!(
            Mcts::new(chrono::Duration::seconds(1)).solve(&round, start),
            expected
        );
    }

    // Test short path
    #[test]
    fn solve() {
        let (pos, game) = create_board();
        let target = Target::Red(Symbol::Triangle);

        let round = Round::new(
            game.board().clone(),
            target,
            game.get_target_position(&target).unwrap(),
        );

        let expected = Path::new(
            pos.clone(),
            RobotPositions::from_tuples(&[(1, 3), (5, 4), (7, 1), (7, 15)]),
            vec![
                (Robot::Red, Direction::Up),
                (Robot::Red, Direction::Right),
                (Robot::Red, Direction::Down),
            ],
        );

        assert_eq!(
            Mcts::new_seeded(chrono::Duration::seconds(1), 3).solve(&round, pos),
            expected
        );
    }

    #[test]
    fn monte_carlo_solve() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(10);

        let (pos, game) = create_board();
        let target = Target::Red(Symbol::Triangle);
        let round = Round::new(
            game.board().clone(),
            target,
            game.get_target_position(&target).unwrap(),
        );

        let mut tries = 0;
        let mut total_moves: u64 = 0;
        let mut path;

        loop {
            path = Vec::new();
            let mut current_pos = pos.clone();
            tries += 1;

            loop {
                let robot = ROBOTS[rng.gen_range(0..4)];
                let direction = DIRECTIONS[rng.gen_range(0..4)];
                let new_pos =
                    current_pos
                        .clone()
                        .move_in_direction(&round.board(), robot, direction);
                if new_pos == current_pos {
                    continue;
                }
                current_pos = new_pos;
                path.push((robot, direction));

                total_moves += 1;
                if round.target_reached(&current_pos) {
                    break;
                }
            }

            if path.len() <= 3 {
                break;
            }
        }

        assert_eq!(tries, 2781);
        assert_eq!(total_moves, 596132);
        assert_eq!(
            path,
            vec![
                (Robot::Red, Direction::Up),
                (Robot::Red, Direction::Right),
                (Robot::Red, Direction::Down)
            ]
        );
    }
}
