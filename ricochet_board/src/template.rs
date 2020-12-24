//! Create a [`Game`](super::Game) from templates.
//!
//! These templates are the same as the quarters used to build the actual board game.

use std::fmt;

use crate::{Field, Symbol::*, Target, BOARDSIZE};

/// The orientation of a template.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Orientation {
    /// Indicates a template rotated so it fits in the upper left.
    UpperLeft,
    /// Indicates a template rotated so it fits in the upper right.
    UpperRight,
    /// Indicates a template rotated so it fits in the bottom right.
    BottomRight,
    /// Indicates a template rotated so it fits in the bottom left.
    BottomLeft,
}

impl Orientation {
    /// Returns the number of clockwise rotations needed to rotate a template to `orient`.
    pub fn right_rotations_to(self, orient: Orientation) -> usize {
        let all = [
            Orientation::UpperLeft,
            Orientation::UpperRight,
            Orientation::BottomRight,
            Orientation::BottomLeft,
        ];
        let self_pos = all.iter().position(|o| o == &self).unwrap() as isize;
        let orient_pos = all.iter().position(|o| o == &orient).unwrap() as isize;
        (orient_pos - self_pos + all.len() as isize) as usize % all.len()
    }
}

impl fmt::Display for Orientation {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                Orientation::UpperLeft => "upper left",
                Orientation::UpperRight => "upper right",
                Orientation::BottomRight => "bottom right",
                Orientation::BottomLeft => "bottom left",
            }
        )
    }
}

/// The color of a template which is given by the physical counterpart.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TempColor {
    /// Indicates a green template.
    Green,
    /// Indicates a red template.
    Red,
    /// Indicates a blue template.
    Blue,
    /// Indicates a yellow template.
    Yellow,
}

impl fmt::Display for TempColor {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                TempColor::Green => r#""green"(g)"#,
                TempColor::Red => r#""red"(r)"#,
                TempColor::Blue => r#""blue"(b)"#,
                TempColor::Yellow => r#""yellow"(y)"#,
            }
        )
    }
}

/// The directions a [`Field`](super::Field) stores walls for.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WallDirection {
    /// Indicates a wall at the bottom of a field.
    Down,
    /// Indicates a wall to the right of a field.
    Right,
}

impl WallDirection {
    /// Changes the direction of a wall when rotating a template.
    fn rotate(self) -> Self {
        match self {
            WallDirection::Down => WallDirection::Right,
            WallDirection::Right => WallDirection::Down,
        }
    }
}

/// A template representing a quarter of the ricochet board.
///
/// The physical board is built from four 8x8 pieces. Each of these pieces is assigned a color and
/// can be rotated in four different ways.
#[derive(Clone, Debug, PartialEq)]
pub struct BoardTemplate {
    orientation: Orientation,
    color: TempColor,
    walls: Vec<((isize, isize), WallDirection)>,
    targets: Vec<((isize, isize), Target)>,
}

impl BoardTemplate {
    /// Returns the color of the template.
    pub fn color(&self) -> TempColor {
        self.color
    }

    /// Returns the orientation of the template.
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// Returns the walls on the template.
    pub fn walls(&self) -> &Vec<((isize, isize), WallDirection)> {
        &self.walls
    }

    /// Returns the targets on the template.
    pub fn targets(&self) -> &Vec<((isize, isize), Target)> {
        &self.targets
    }

    /// Rotates the template clockwise.
    pub fn rotate_right(&mut self) {
        self.orientation = match self.orientation {
            Orientation::UpperLeft => Orientation::UpperRight,
            Orientation::UpperRight => Orientation::BottomRight,
            Orientation::BottomRight => Orientation::BottomLeft,
            Orientation::BottomLeft => Orientation::UpperLeft,
        };

        self.walls = self
            .walls
            .iter()
            .map(|&((c, r), dir)| match dir {
                WallDirection::Right => (((BOARDSIZE / 2) as isize - r - 1, c), dir.rotate()),
                WallDirection::Down => (((BOARDSIZE / 2 - 1) as isize - r - 1, c), dir.rotate()),
            })
            .collect();

        self.targets = self
            .targets
            .iter()
            .map(|&((c, r), t)| (((BOARDSIZE / 2) as isize - r - 1, c), t))
            .collect();
    }

    /// Rotates the template to the given orientation.
    pub fn rotate_to(&mut self, orient: Orientation) {
        for _ in 0..self.orientation.right_rotations_to(orient) {
            self.rotate_right();
        }
    }

    /// Creates a default template with `color` in the upper left with no walls or targets.
    fn default_template(color: TempColor) -> Self {
        BoardTemplate {
            orientation: Orientation::UpperLeft,
            color,
            walls: Vec::new(),
            targets: Vec::new(),
        }
    }

    /// Sets multiple walls in the given direction.
    fn set_walls(mut self, dir: WallDirection, walls: Vec<(isize, isize)>) -> Self {
        for (c, r) in walls {
            self.walls.push(((c, r), dir));
        }
        self
    }

    /// Adds `target` at `pos` to the template.
    fn set_target(mut self, pos: (isize, isize), target: Target) -> Self {
        self.targets.push((pos, target));
        self
    }
}

impl fmt::Display for BoardTemplate {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        const SIZE: usize = (BOARDSIZE / 2 + 1) as usize;
        let mut print = [[Field::default(); SIZE]; SIZE];

        for ((c, r), d) in &self.walls {
            let field = &mut print[(c + 1) as usize][(r + 1) as usize];
            match d {
                WallDirection::Down => field.down = true,
                WallDirection::Right => field.right = true,
            }
        }

        let print: Vec<Vec<Field>> = print.iter().map(|&a| a.to_vec()).collect();
        write!(fmt, "{}", crate::board_string(print))
    }
}

/// Creates a vec containing all known templates.
///
/// Each color has three templates and the vec contains them in the order red, blue, green, yellow.
pub fn gen_templates() -> Vec<BoardTemplate> {
    let mut temps = Vec::with_capacity(12);

    // Add red boards
    temps.push(
        BoardTemplate::default_template(TempColor::Red)
            .set_walls(
                WallDirection::Down,
                vec![(0, 5), (1, 3), (3, 6), (4, 0), (5, 4)],
            )
            .set_walls(
                WallDirection::Right,
                vec![(0, 3), (1, 0), (3, 6), (4, 1), (4, 5)],
            )
            .set_target((1, 3), Target::Red(Triangle))
            .set_target((3, 6), Target::Blue(Hexagon))
            .set_target((4, 1), Target::Green(Circle))
            .set_target((5, 5), Target::Yellow(Square)),
    );
    temps.push(
        BoardTemplate::default_template(TempColor::Red)
            .set_walls(
                WallDirection::Down,
                vec![(0, 5), (1, 1), (2, 4), (6, 1), (7, 4)],
            )
            .set_walls(
                WallDirection::Right,
                vec![(0, 1), (2, 4), (3, 0), (6, 2), (6, 5)],
            )
            .set_target((1, 1), Target::Red(Triangle))
            .set_target((2, 4), Target::Blue(Hexagon))
            .set_target((6, 2), Target::Green(Circle))
            .set_target((7, 5), Target::Yellow(Square)),
    );
    temps.push(
        BoardTemplate::default_template(TempColor::Red)
            .set_walls(
                WallDirection::Down,
                vec![(0, 4), (1, 5), (2, 3), (5, 2), (7, 5)],
            )
            .set_walls(
                WallDirection::Right,
                vec![(0, 6), (2, 4), (3, 0), (5, 2), (6, 5)],
            )
            .set_target((1, 6), Target::Yellow(Square))
            .set_target((2, 4), Target::Green(Circle))
            .set_target((5, 2), Target::Blue(Hexagon))
            .set_target((7, 5), Target::Red(Triangle)),
    );

    // Add blue boards
    temps.push(
        BoardTemplate::default_template(TempColor::Blue)
            .set_walls(
                WallDirection::Down,
                vec![(0, 3), (2, 3), (3, 1), (4, 5), (5, 3)],
            )
            .set_walls(
                WallDirection::Right,
                vec![(2, 2), (2, 4), (4, 3), (4, 5), (5, 0)],
            )
            .set_target((2, 4), Target::Red(Square))
            .set_target((3, 2), Target::Yellow(Circle))
            .set_target((4, 5), Target::Green(Hexagon))
            .set_target((5, 3), Target::Blue(Triangle)),
    );
    temps.push(
        BoardTemplate::default_template(TempColor::Blue)
            .set_walls(
                WallDirection::Down,
                vec![(0, 3), (1, 2), (2, 5), (5, 1), (6, 3)],
            )
            .set_walls(
                WallDirection::Right,
                vec![(0, 2), (2, 6), (3, 0), (5, 1), (5, 4)],
            )
            .set_target((1, 2), Target::Red(Square))
            .set_target((2, 6), Target::Blue(Triangle))
            .set_target((5, 1), Target::Green(Hexagon))
            .set_target((6, 4), Target::Yellow(Circle)),
    );
    temps.push(
        BoardTemplate::default_template(TempColor::Blue)
            .set_walls(
                WallDirection::Down,
                vec![(0, 4), (1, 6), (2, 0), (4, 4), (6, 3)],
            )
            .set_walls(
                WallDirection::Right,
                vec![(1, 1), (1, 6), (4, 0), (4, 5), (5, 3)],
            )
            .set_target((1, 6), Target::Green(Hexagon))
            .set_target((2, 1), Target::Yellow(Circle))
            .set_target((4, 5), Target::Red(Square))
            .set_target((6, 3), Target::Blue(Triangle)),
    );

    // Add green boards
    temps.push(
        BoardTemplate::default_template(TempColor::Green)
            .set_walls(
                WallDirection::Down,
                vec![(0, 6), (1, 4), (3, 0), (4, 5), (6, 3)],
            )
            .set_walls(
                WallDirection::Right,
                vec![(0, 4), (1, 0), (2, 1), (4, 6), (6, 3)],
            )
            .set_target((1, 4), Target::Red(Circle))
            .set_target((3, 1), Target::Green(Triangle))
            .set_target((4, 6), Target::Blue(Square))
            .set_target((6, 3), Target::Yellow(Hexagon)),
    );
    temps.push(
        BoardTemplate::default_template(TempColor::Green)
            .set_walls(
                WallDirection::Down,
                vec![(0, 5), (1, 1), (3, 6), (4, 0), (6, 3)],
            )
            .set_walls(
                WallDirection::Right,
                vec![(1, 0), (1, 2), (2, 6), (3, 1), (6, 3)],
            )
            .set_target((1, 2), Target::Green(Triangle))
            .set_target((3, 6), Target::Blue(Square))
            .set_target((4, 1), Target::Red(Circle))
            .set_target((6, 3), Target::Yellow(Hexagon)),
    );
    temps.push(
        BoardTemplate::default_template(TempColor::Green)
            .set_walls(
                WallDirection::Down,
                vec![(0, 5), (1, 1), (3, 6), (6, 1), (6, 4)],
            )
            .set_walls(
                WallDirection::Right,
                vec![(0, 2), (2, 6), (4, 0), (6, 1), (6, 5)],
            )
            .set_target((1, 2), Target::Green(Triangle))
            .set_target((3, 6), Target::Red(Circle))
            .set_target((6, 1), Target::Yellow(Hexagon))
            .set_target((6, 5), Target::Blue(Square)),
    );

    // Add yellow boards
    temps.push(
        BoardTemplate::default_template(TempColor::Yellow)
            .set_walls(
                WallDirection::Down,
                vec![(0, 3), (1, 5), (3, 4), (5, 1), (6, 4), (7, 2)],
            )
            .set_walls(
                WallDirection::Right,
                vec![(1, 6), (2, 0), (3, 4), (4, 1), (5, 5), (7, 2)],
            )
            .set_target((1, 6), Target::Yellow(Triangle))
            .set_target((3, 4), Target::Red(Hexagon))
            .set_target((5, 1), Target::Blue(Circle))
            .set_target((6, 5), Target::Green(Square))
            .set_target((7, 2), Target::Spiral),
    );
    temps.push(
        BoardTemplate::default_template(TempColor::Yellow)
            .set_walls(
                WallDirection::Down,
                vec![(0, 4), (1, 3), (2, 1), (3, 7), (5, 5), (6, 3)],
            )
            .set_walls(
                WallDirection::Right,
                vec![(0, 3), (2, 1), (3, 7), (4, 0), (5, 4), (5, 6)],
            )
            .set_target((1, 3), Target::Green(Square))
            .set_target((3, 1), Target::Red(Hexagon))
            .set_target((3, 7), Target::Spiral)
            .set_target((5, 6), Target::Blue(Circle))
            .set_target((6, 4), Target::Yellow(Triangle)),
    );
    temps.push(
        BoardTemplate::default_template(TempColor::Yellow)
            .set_walls(
                WallDirection::Down,
                vec![(0, 6), (1, 2), (2, 5), (5, 3), (6, 1), (7, 5)],
            )
            .set_walls(
                WallDirection::Right,
                vec![(1, 3), (2, 5), (3, 0), (4, 4), (5, 1), (7, 5)],
            )
            .set_target((1, 3), Target::Yellow(Triangle))
            .set_target((2, 5), Target::Red(Hexagon))
            .set_target((5, 4), Target::Green(Square))
            .set_target((6, 1), Target::Blue(Circle))
            .set_target((7, 5), Target::Spiral),
    );

    temps
}
