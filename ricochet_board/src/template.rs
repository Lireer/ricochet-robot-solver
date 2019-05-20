use crate::{Field, BOARDSIZE};

pub enum Orientation {
    UpperLeft,
    UpperRight,
    BottomRight,
    BottomLeft,
}

pub enum TempColour {
    Green,
    Red,
    Blue,
    Yellow,
}

pub struct BoardTemplate {
    orientation: Orientation,
    color: TempColour,
    template: [[Field; BOARDSIZE / 2]; BOARDSIZE / 2],
}

impl BoardTemplate {
    pub fn rotate_right(&mut self) {
        self.orientation = match self.orientation {
            Orientation::UpperLeft => Orientation::UpperRight,
            Orientation::UpperRight => Orientation::BottomRight,
            Orientation::BottomRight => Orientation::BottomLeft,
            Orientation::BottomLeft => Orientation::UpperLeft,
        };

        let mut new_temp = [[Field {
            bottom: false,
            right: false,
        }; BOARDSIZE / 2]; BOARDSIZE / 2];

        for row in 0..BOARDSIZE / 2 {
            for col in 0..BOARDSIZE / 2 {
                if self.template[row][col].bottom {
                    new_temp[(BOARDSIZE / 2 - 1) - col - 1][row].right = true
                }
                if self.template[row][col].right {
                    new_temp[(BOARDSIZE / 2 - 1) - col][row].right = true
                }
            }
        }
    }
}


// TODO: Finish generating templates
const templates: [BoardTemplate; 1] = [BoardTemplate {
    orientation: Orientation::UpperLeft,
    color: TempColour::Blue,
    template: [[Field {
        bottom: false,
        right: false,
    }; BOARDSIZE / 2]; BOARDSIZE / 2],
}];

const fn gen_templates(
    orientation: Orientation,
    colour: TempColour,
) -> BoardTemplate {
    let mut board = BoardTemplate {
        orientation: Orientation::UpperLeft,
        color: TempColour::Blue,
        template: [[Field {
            bottom: false,
            right: false,
        }; BOARDSIZE / 2]; BOARDSIZE / 2],
    };
    return board;
}
