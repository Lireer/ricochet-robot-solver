use chrono::Local;
use itertools::Itertools;
use rand::Rng;
use rayon::iter::{ParallelBridge, ParallelIterator};
use ricochet_board::{template, RobotPositions, Round, Symbol, Target};
use ricochet_solver::{Path, Solver};
use serde::Serialize;
use std::collections::HashSet;
use std::sync::mpsc;
use std::{fs, thread, usize};

const BOARD_TARGET_VARIANTS: usize = 3 * 9 * 6 * 3 * 17;
const CSV_PATH: &str = "solutions.csv";

fn main() {
    let (sender, receiver) = mpsc::channel::<SolutionData>();

    // start writer thread with receiver
    let writer_thread = thread::spawn(move || {
        let file = fs::OpenOptions::new()
            .append(true)
            .open(CSV_PATH)
            .expect(&format!("failed to open {}", CSV_PATH));
        let mut writer = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(file);
        let mut counter = 0;
        while let Ok(data) = receiver.recv() {
            writer.serialize(data).expect("failed to write data to csv");
            counter += 1;
            if counter % 1000 == 0 {
                println!("{}: {:>10} records written", Local::now(), counter);
            }
        }
        println!("{}: finished writing", Local::now());
    });

    // start rayon threads with sender
    (0..BOARD_TARGET_VARIANTS)
        .cycle()
        .map(move |i| (i, sender.clone()))
        // .take(BOARD_TARGET_VARIANTS * 2)
        .par_bridge()
        .for_each(|(board_seed, sender)| {
            let mut data = SolutionData::new(board_seed);
            let start_time = Local::now();
            let path = ricochet_solver::AStar::new().solve(&data.round(), data.start_positions());
            data.finalize(Local::now() - start_time, path);
            sender.send(data).expect("could not send data to writer");
        });
    println!("{}: waiting for writer to finish", Local::now());
    writer_thread.join().expect("could not join writer thread");
}

#[derive(Debug, Serialize)]
struct SolutionData {
    board_seed: usize,
    positions: u32,
    time_micros: Option<i64>,
    length: Option<usize>,
    robots_used: Option<usize>,
    #[serde(skip)]
    path: Option<Path>,
}

impl SolutionData {
    pub fn new(board_seed: usize) -> Self {
        let positions = loop {
            let pos: u32 = rand::thread_rng().gen();
            if Self::valid_position(pos) {
                break pos;
            }
        };
        Self {
            board_seed,
            positions,
            time_micros: None,
            length: None,
            robots_used: None,
            path: None,
        }
    }

    pub fn finalize(&mut self, duration: chrono::Duration, path: Path) {
        self.time_micros = duration.num_microseconds();
        self.length = Some(path.len());
        self.robots_used = Some(path.movements().iter().map(|mm| mm.0).unique().count());
        self.path = Some(path);
    }

    pub fn round(&self) -> Round {
        let mut indices = Vec::new();
        let mut div_mod = |i: usize, div: usize| {
            indices.push(i % div);
            i / div
        };

        let mut div = self.board_seed;
        for denom in vec![17, 3, 9, 6, 3] {
            div = div_mod(div, denom);
        }

        let templates = ricochet_board::template::gen_templates();
        let mut chosen_tpl = Vec::with_capacity(4);

        // Choose a red template for the upper left piece.
        chosen_tpl.push(templates[indices[1]].clone());

        for &idx in &indices[2..] {
            let next_tpl = templates
                .iter()
                .filter(|&tpl| !chosen_tpl.iter().any(|ct| ct.color() == tpl.color()))
                .nth(idx)
                .unwrap()
                .clone();
            chosen_tpl.push(next_tpl);
        }
        assert!(chosen_tpl.len() == 4);
        assert!(
            chosen_tpl
                .iter()
                .map(|tpl| tpl.color())
                .collect::<HashSet<_>>()
                .len()
                == 4
        );

        let target = num_to_target(indices[0]);

        chosen_tpl
            .iter_mut()
            .zip(template::ORIENTATIONS.iter())
            .for_each(|(tpl, orient)| tpl.rotate_to(*orient));
        let game = ricochet_board::Game::from_templates(&chosen_tpl);
        let target_position = game
            .get_target_position(&target)
            .expect("Failed to find the position of the target on the board");
        Round::new(game.board().clone(), target, target_position)
    }

    pub fn start_positions(&self) -> RobotPositions {
        RobotPositions::from_tuples(&Self::positions_as_tuples(self.positions))
    }

    fn positions_as_tuples(pos: u32) -> [(u16, u16); 4] {
        let mut out = [(0, 0); 4];

        for (shift, out_idx) in (0..4).rev().enumerate() {
            out[out_idx].1 = ((pos >> 8 * shift) & 0b1111) as u16;
            out[out_idx].0 = ((pos >> 8 * shift + 4) & 0b1111) as u16;
        }

        out
    }

    fn valid_position(pos: u32) -> bool {
        !Self::positions_as_tuples(pos)
            .iter()
            .any(|(col, row)| (7..=8).contains(col) && (7..=8).contains(row))
    }
}

fn num_to_target(n: usize) -> Target {
    match n {
        0..=3 => Target::Red(num_to_target_symbol(n % 4)),
        4..=7 => Target::Blue(num_to_target_symbol(n % 4)),
        8..=11 => Target::Green(num_to_target_symbol(n % 4)),
        12..=15 => Target::Yellow(num_to_target_symbol(n % 4)),
        16 => Target::Spiral,
        _ => panic!(),
    }
}

fn num_to_target_symbol(n: usize) -> Symbol {
    match n {
        0 => Symbol::Circle,
        1 => Symbol::Triangle,
        2 => Symbol::Square,
        3 => Symbol::Hexagon,
        _ => panic!(),
    }
}
