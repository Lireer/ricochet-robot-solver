use std::fmt;

use crate::{Field, Symbol::*, Target, BOARDSIZE};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Orientation {
    UpperLeft,
    UpperRight,
    BottomRight,
    BottomLeft,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TempColor {
    Green,
    Red,
    Blue,
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WallDirection {
    Bottom,
    Right,
}

impl WallDirection {
    fn rotate(self) -> Self {
        match self {
            WallDirection::Bottom => WallDirection::Right,
            WallDirection::Right => WallDirection::Bottom,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoardTemplate {
    orientation: Orientation,
    color: TempColor,
    walls: Vec<((isize, isize), WallDirection)>,
    targets: Vec<((isize, isize), Target)>,
}

impl BoardTemplate {
    pub fn color(&self) -> TempColor {
        self.color
    }

    pub fn rotate_right(mut self) -> Self {
        self.orientation = match self.orientation {
            Orientation::UpperLeft => Orientation::UpperRight,
            Orientation::UpperRight => Orientation::BottomRight,
            Orientation::BottomRight => Orientation::BottomLeft,
            Orientation::BottomLeft => Orientation::UpperLeft,
        };

        self.walls = self
            .walls
            .iter()
            .map(|&((c, r), dir)| (((BOARDSIZE / 2 - 1) as isize - r - 1, c), dir.rotate()))
            .collect();
        self.targets = self
            .targets
            .iter()
            .map(|&((c, r), t)| (((BOARDSIZE / 2 - 1) as isize - r - 1, c), t))
            .collect();

        self
    }

    fn default_template(color: TempColor) -> Self {
        BoardTemplate {
            orientation: Orientation::UpperLeft,
            color,
            walls: Vec::new(),
            targets: Vec::new(),
        }
    }

    fn set_walls(mut self, dir: WallDirection, walls: Vec<(isize, isize)>) -> Self {
        for (c, r) in walls {
            self.walls.push(((c, r), dir));
        }
        self
    }

    fn set_target(mut self, pos: (isize, isize), target: Target) -> Self {
        self.targets.push((pos, target));
        self
    }
}

impl fmt::Display for BoardTemplate {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut print = [[Field::default(); BOARDSIZE / 2 + 1]; BOARDSIZE / 2 + 1];

        for ((c, r), d) in &self.walls {
            let field = &mut print[(c + 1) as usize][(r + 1) as usize];
            match d {
                WallDirection::Bottom => field.bottom = true,
                WallDirection::Right => field.right = true,
            }
        }

        let print: Vec<Vec<Field>> = print.iter().map(|&a| a.to_vec()).collect();
        write!(fmt, "{}", crate::board_string(print))
    }
}

pub fn gen_templates() -> Vec<BoardTemplate> {
    let mut temps = Vec::with_capacity(12);

    // Add blue boards
    temps.push(
        BoardTemplate::default_template(TempColor::Blue)
            .set_walls(
                WallDirection::Bottom,
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
                WallDirection::Bottom,
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
                WallDirection::Bottom,
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

    // Add yellow boards
    temps.push(
        BoardTemplate::default_template(TempColor::Yellow)
            .set_walls(
                WallDirection::Bottom,
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
                WallDirection::Bottom,
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
                WallDirection::Bottom,
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

    // Add red boards
    temps.push(
        BoardTemplate::default_template(TempColor::Red)
            .set_walls(
                WallDirection::Bottom,
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
                WallDirection::Bottom,
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
                WallDirection::Bottom,
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

    // Add green boards
    temps.push(
        BoardTemplate::default_template(TempColor::Green)
            .set_walls(
                WallDirection::Bottom,
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
                WallDirection::Bottom,
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
                WallDirection::Bottom,
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

    temps
}
