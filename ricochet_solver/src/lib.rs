use fnv::FnvHashMap;
use std::collections::hash_map::Entry;

use ricochet_board::{Board, Color, Direction, RobotPositions, Target};

struct VisitInformation {
    steps_needed: usize,
    previous_position: RobotPositions,
    robot: Color,
    direction: Direction,
}

impl VisitInformation {
    pub fn new(
        steps: usize,
        previous_position: RobotPositions,
        robot: Color,
        direction: Direction,
    ) -> Self {
        VisitInformation {
            steps_needed: steps,
            previous_position,
            robot,
            direction,
        }
    }

    pub fn steps(&self) -> usize {
        self.steps_needed
    }

    pub fn previous_position(&self) -> &RobotPositions {
        &self.previous_position
    }

    pub fn from_path(&self) -> (Color, Direction) {
        (self.robot, self.direction)
    }
}

pub fn solve(
    board: &Board,
    positions: RobotPositions,
    target: Target,
) -> (RobotPositions, Vec<(Color, Direction)>) {
    // Check if the robot has already reached the target
    if board.target_reached(target, &positions) {
        return (positions, vec![]);
    }

    mem_solve(board, positions, target)
}

fn mem_solve(
    board: &Board,
    start_pos: RobotPositions,
    target: Target,
) -> (RobotPositions, Vec<(Color, Direction)>) {
    // contains all positions from which the positions in
    let mut current_step_positions: Vec<RobotPositions> = Vec::with_capacity(256);
    current_step_positions.push(start_pos.clone());
    let mut next_step_positions: Vec<RobotPositions> = Vec::with_capacity(256);

    // store information regarding the
    let mut position_store = FnvHashMap::with_capacity_and_hasher(1024, Default::default());

    // initialize the positions which will store the solution with the starting position
    let mut solution = start_pos;

    // Forward pathing to the target.
    // Computes the min. number of steps to the target and creates a tree of reachable positions
    // in `position_store`, which is used in the backwards path creation.
    'outer: for step in 0.. {
        for pos in &current_step_positions {
            //0..visited_positions[step].len() {
            if let Some(reached) = eval(
                board,
                target,
                pos,
                step,
                &mut position_store,
                // &mut next_step_positions,
                &mut |pos: &RobotPositions| next_step_positions.push(pos.clone()),
            ) {
                solution = reached;
                break 'outer;
            };
        }
        // visited_positions.push(next_step_positions);
        current_step_positions.clear();
        std::mem::swap(&mut current_step_positions, &mut next_step_positions)
    }

    // Backwards path creation from the final positions to the starting position.
    let mut path = Vec::with_capacity(32);
    let mut current_pos = solution.clone();

    loop {
        // This should never panic since the position should be in `position_store`.
        let current_info = position_store
            .get(&current_pos)
            .expect("Failed to find a supposed source position");
        path.push(current_info.from_path());
        if current_info.steps() == 1 {
            break;
        }
        current_pos = current_info.previous_position().clone();
    }

    path.reverse();
    (solution, path)
}

const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Right,
    Direction::Left,
];

const ROBOTS: [Color; 4] = [Color::Red, Color::Blue, Color::Green, Color::Yellow];

/// Calculates all new possible positions starting from `initial_pos`.
/// `steps` is the number of steps needed to reach `initial_pos`.
/// The calculated postions are inserted into `visited_pos` and `pos_store`.
fn eval<F: FnMut(&RobotPositions)>(
    board: &Board,
    target: Target,
    initial_pos: &RobotPositions,
    steps: usize,
    pos_store: &mut FnvHashMap<RobotPositions, VisitInformation>,
    // next_step_pos: &mut Vec<RobotPositions>,
    add_pos: &mut F,
) -> Option<RobotPositions> {
    for &robot in ROBOTS.iter() {
        for &dir in DIRECTIONS.iter() {
            // create a position starting from the initial position
            let mut new_pos = initial_pos.clone();
            new_pos.move_in_direction(board, robot, dir);

            let entry = pos_store.entry(new_pos.clone());
            match entry {
                // This position has already been reached
                Entry::Occupied(_) => continue,
                // First time this position has been reached
                Entry::Vacant(entry) => entry.insert(VisitInformation::new(
                    steps + 1,
                    initial_pos.clone(),
                    robot,
                    dir,
                )),
            };

            // Check if the target has been reached
            if board.target_reached(target, &new_pos) {
                return Some(new_pos);
            }

            // Add new_pos to the already visited positions
            // next_step_pos.push(new_pos);
            add_pos(&new_pos);
        }
    }
    None
}
