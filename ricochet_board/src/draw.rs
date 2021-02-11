use crate::Field;
use draw_a_box::{find_character, Weight};

/// Width per field in the string in number of characters.
pub const FIELD_DRAW_WIDTH: usize = 5;

/// Height per field in the string in number of characters.
pub const FIELD_DRAW_HEIGHT: usize = 2;

/// Creates a string representation of the walls of a board.
pub fn draw_board(walls: &[Vec<Field>]) -> String {
    let (canvas, _) = create_board_string_vec(walls);
    let mut output = String::new();

    for row in 0..canvas[0].len() {
        for col in &canvas {
            output.push_str(col[row]);
        }
        output.push('\n');
    }

    output
}

/// Creates the strings making up the board and used by `draw_board` to create the actual
/// visualization.
///
/// The second returned value has the same size but each element is a vec containing the four
/// weights describing the string at the same position in the first value. The second value actually
/// only contains information regarding corners.
pub fn create_board_string_vec(walls: &[Vec<Field>]) -> (Vec<Vec<&str>>, Vec<Vec<Vec<Weight>>>) {
    let width = walls.len();
    let height = walls[0].len();
    let canvas_width = width * FIELD_DRAW_WIDTH + 1;
    let canvas_height = height * FIELD_DRAW_HEIGHT + 1;

    let mut canvas = vec![vec![" "; canvas_height]; canvas_width];
    let mut corner_weights = vec![vec![Vec::new(); canvas_height]; canvas_width];

    // Set corners
    for col in (0..canvas_width).step_by(FIELD_DRAW_WIDTH).take(width + 1) {
        for row in (0..canvas_height)
            .step_by(FIELD_DRAW_HEIGHT)
            .take(height + 1)
        {
            let up_left_col: usize = (col / FIELD_DRAW_WIDTH + width - 1) % width;
            let up_left_row: usize = (row / FIELD_DRAW_HEIGHT + height - 1) % height;
            let up_left = walls[up_left_col][up_left_row];
            let up_right = walls[(up_left_col + 1 + width) % width][up_left_row];
            let down_left = walls[up_left_col][(up_left_row + 1 + height) % height];

            let is_set = |is_set| match is_set {
                true => Weight::Heavy,
                false => Weight::Light,
            };
            let mut up_weight = is_set(up_left.right);
            let mut left_weight = is_set(up_left.down);
            let mut right_weight = is_set(up_right.down);
            let mut down_weight = is_set(down_left.right);

            match col {
                0 => left_weight = Weight::Empty,
                i if i == (canvas_width - 1) => right_weight = Weight::Empty,
                _ => (),
            }

            match row {
                0 => up_weight = Weight::Empty,
                i if i == (canvas_height - 1) => down_weight = Weight::Empty,
                _ => (),
            }

            corner_weights[col][row] = vec![up_weight, right_weight, down_weight, left_weight];
            canvas[col][row] = find_character(up_weight, right_weight, down_weight, left_weight);
        }
    }

    // Set horizontal connections
    for row in 0..canvas_height {
        let mut setting = Weight::Light;
        for col in 0..canvas_width {
            if row % FIELD_DRAW_HEIGHT == 0 {
                if canvas[col][row] == " " {
                    let empty = Weight::Empty;
                    canvas[col][row] = find_character(empty, setting, empty, setting)
                } else {
                    setting = corner_weights[col][row][1];
                }
            }
        }
    }

    // Set vertical connections
    for col in 0..canvas_width {
        let mut setting = Weight::Light;
        for row in 0..canvas_height {
            if col % FIELD_DRAW_WIDTH == 0 {
                if canvas[col][row] == " " {
                    let empty = Weight::Empty;
                    canvas[col][row] = find_character(setting, empty, setting, empty)
                } else {
                    setting = corner_weights[col][row][2];
                }
            }
        }
    }

    (canvas, corner_weights)
}
