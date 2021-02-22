//! Tools to generate boards of different sizes.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Board, Direction, Game, Position, PositionEncoding};
use itertools::Itertools;
use rand::prelude::SliceRandom;
use rand::{Rng, SeedableRng};

/// A board generator to create boards of different sizes and configurations.
#[derive(Debug)]
pub struct Generator {
    rng: rand_pcg::Pcg64Mcg,
    side_length: PositionEncoding,
    potential_targets: Vec<Position>,
    occupied_fields: BTreeSet<Position>,
}

impl Generator {
    /// Creates a new generator with a random state.
    ///
    /// # Panics
    /// Panics if `side_length` is less than `3`.
    pub fn new(side_length: PositionEncoding) -> Self {
        Self {
            rng: rand_pcg::Pcg64Mcg::from_entropy(),
            side_length,
            potential_targets: Vec::new(),
            occupied_fields: BTreeSet::new(),
        }
    }

    /// Creates a new generator initialized with `seed`.
    ///
    /// The generator was implemented in a way that focuses on generatin boards with a `side_length`
    /// greater than 6.
    ///
    /// # Panics
    /// Panics if `side_length` is less than `3`.
    pub fn from_seed(seed: u128, side_length: PositionEncoding) -> Self {
        Self {
            rng: rand_pcg::Pcg64Mcg::new(seed.wrapping_mul(2)),
            side_length,
            potential_targets: Vec::new(),
            occupied_fields: BTreeSet::new(),
        }
    }

    /// Generates a new game with a board and targets.
    ///
    /// Some targets may be on the same field.
    pub fn generate_game(&mut self) -> Game {
        let board = self.generate_board();
        let mut unused = self.potential_targets.clone();
        let mut targets = BTreeMap::new();
        for &target in &crate::TARGETS {
            if unused.is_empty() {
                unused = self.potential_targets.clone();
            }
            let pos = *unused.choose(&mut self.rng).unwrap();
            targets.insert(target, pos);
        }

        Game::new(board, targets)
    }

    /// Generates a new board and updates potential targets.
    pub fn generate_board(&mut self) -> Board {
        let mut base = Board::new_empty(self.side_length);
        self.potential_targets = Vec::new();
        self.occupied_fields = BTreeSet::new();

        if self.side_length >= 10 {
            base = base.set_center_walls();
            let f = self.side_length / 2 - 1;
            for (col_add, row_add) in [0, 1].iter().cartesian_product(&[0, 1]) {
                self.add_occupied_field(Position::new(f + col_add, f + row_add));
            }
        }

        self.add_outer_wall_protrusions(&mut base);

        let first_quad_len = self.side_length / 2;
        let mut other_quad_len = first_quad_len;
        if self.side_length % 2 == 1 {
            other_quad_len += 1
        }
        // The parts of the quadrants in which walls will be generated in the form
        // `((col, row), (width, height))`.
        let quadrants = vec![
            ((1, 1), (first_quad_len - 1, first_quad_len - 1)),
            (
                (1, first_quad_len),
                (first_quad_len - 1, other_quad_len - 1),
            ),
            (
                (first_quad_len, 1),
                (other_quad_len - 1, first_quad_len - 1),
            ),
            (
                (first_quad_len, first_quad_len),
                (other_quad_len - 1, other_quad_len - 1),
            ),
        ];

        let fields = |occupied: &BTreeSet<Position>, ((col, row), (width, height))| {
            (col..(col + width))
                .cartesian_product(row..(row + height))
                .map(Position::from)
                .collect::<BTreeSet<_>>()
                .difference(occupied)
                .cloned()
                .collect::<Vec<_>>()
        };

        let fields_per_quad = (self.side_length as f64 / 4.0).round() as usize;
        for quad in quadrants {
            for _ in 0..fields_per_quad {
                let chosen = match fields(&self.occupied_fields, quad).choose(&mut self.rng) {
                    Some(field) => *field,
                    None => break,
                };
                self.walls_around_field(&mut base, chosen);

                self.potential_targets.push(chosen);
                self.add_occupied_field(chosen);
            }
        }

        let open_fields = fields(
            &self.occupied_fields,
            ((1, 1), (self.side_length - 2, self.side_length - 2)),
        );
        if let Some(&field) = open_fields.choose(&mut self.rng) {
            self.walls_around_field(&mut base, field);
            self.potential_targets.push(field);
        }

        base = base.wall_enclosure();
        base
    }

    /// Adds a random corner wall to the field at `pos`.
    ///
    /// # Panics
    /// May panic if `pos` is at the edge of the board.
    fn walls_around_field(&mut self, board: &mut Board, pos: Position) {
        let dirs = crate::DIRECTIONS;
        match dirs.choose(&mut self.rng).unwrap() {
            Direction::Up => {
                let above = Position::new(pos.column(), pos.row() - 1);
                board[above].down = true;
                board[pos].right = true;
            }
            Direction::Right => {
                board[pos].right = true;
                board[pos].down = true;
            }
            Direction::Down => {
                let left = Position::new(pos.column() - 1, pos.row());
                board[pos].down = true;
                board[left].right = true;
            }
            Direction::Left => {
                let left = Position::new(pos.column() - 1, pos.row());
                let above = Position::new(pos.column(), pos.row() - 1);
                board[left].right = true;
                board[above].down = true;
            }
        }
    }

    /// Adds walls protruding from the outer walls to the board.
    fn add_outer_wall_protrusions(&mut self, board: &mut Board) {
        let walls = board.get_mut_walls();
        let num_per_wall = (self.side_length as usize + 7) / 8;
        let segment_length = self.side_length as usize / num_per_wall;
        let is_odd_length = self.side_length % 2 == 1;

        // Get the indices of the fields for which walls will be set.
        let get_indices = |generator: &mut Self| {
            let mut indices = Vec::with_capacity(num_per_wall);
            let mut segment_sum = 0;
            for n in 0..num_per_wall {
                let mut len = segment_length;
                if is_odd_length && (num_per_wall - n) % 2 == 1 {
                    len += 1;
                }

                // Exclude the first field of the first segment.
                let start = segment_sum + (n == 0) as usize;

                segment_sum += len;

                let mut end = segment_sum - 1;
                if n == num_per_wall - 1 {
                    // Exclude the last two fields of the last segment.
                    end -= 1;
                }

                indices.push(generator.rng.gen_range(start..end))
            }
            indices
        };

        // Set protrusions at the top and bottom.
        let other_idx = [0, walls.len() - 1];
        for &row in &other_idx {
            for col in get_indices(self) {
                walls[col][row].right = true;
                self.add_occupied_field(Position::new(
                    col as PositionEncoding,
                    row as PositionEncoding,
                ));
            }
        }

        // Set protrusions at walls on the left and on the right.
        for &col in &other_idx {
            for row in get_indices(self) {
                walls[col][row].down = true;
                self.add_occupied_field(Position::new(
                    col as PositionEncoding,
                    row as PositionEncoding,
                ));
            }
        }
    }

    /// Adds a field and its surroundings to `self.occupied_fields`.
    fn add_occupied_field(&mut self, pos: Position) {
        let additions: Vec<(_, fn(_, _) -> _)> = vec![
            (1, PositionEncoding::checked_sub),
            (0, PositionEncoding::checked_add),
            (1, PositionEncoding::checked_add),
        ];
        for (col_add, row_add) in additions.iter().cartesian_product(&additions) {
            let col = match col_add.1(pos.column(), col_add.0) {
                Some(col) if col < self.side_length => col,
                _ => continue,
            };
            let row = match row_add.1(pos.row(), row_add.0) {
                Some(row) if row < self.side_length => row,
                _ => continue,
            };
            self.occupied_fields.insert(Position::new(col, row));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Generator;

    #[test]
    fn different_seeds() {
        let board_one = Generator::from_seed(0, 9).generate_board();
        let board_two = Generator::from_seed(u128::MAX / 2 + 1, 9).generate_board();
        assert_eq!(*board_one.get_walls(), *board_two.get_walls());
    }

    #[test]
    fn generate_games() {
        let mut gen = Generator::from_seed(1234567890, 16);
        for _ in 0..100 {
            gen.generate_game();
        }
    }
}
