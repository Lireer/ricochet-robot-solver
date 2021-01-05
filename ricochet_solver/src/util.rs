use std::collections::hash_map::Entry;

use fnv::FnvHashMap;
use getset::{CopyGetters, Getters};
use ricochet_board::{Color, Direction, RobotPositions};

use crate::Solution;

#[derive(Debug, Clone)]
pub struct VisitedNodes {
    nodes: FnvHashMap<RobotPositions, VisitedNode>,
}

impl VisitedNodes {
    /// Creates a new `VisitedNodes` with the given `capacity`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: FnvHashMap::with_capacity_and_hasher(capacity, Default::default()),
        }
    }

    /// Removes all stored nodes.
    pub fn clear(&mut self) {
        self.nodes.clear()
    }

    /// Returns the visit information of a node.
    pub fn get_node(&self, positions: &RobotPositions) -> Option<&VisitedNode> {
        self.nodes.get(positions)
    }

    pub fn add_node(
        &mut self,
        positions: RobotPositions,
        from: &RobotPositions,
        steps: usize,
        moved: (Color, Direction),
    ) -> bool {
        match self.nodes.entry(positions) {
            Entry::Occupied(occupied) if occupied.get().steps_to_reach() <= steps => {
                // Ignore positions if `occupied` has less or equal steps
                false
            }
            Entry::Occupied(mut occupied) => {
                // A shorter path has been found, insert the new path.
                let visited = VisitedNode::new(steps, from.clone(), moved.0, moved.1);
                occupied.insert(visited);
                true
            }
            Entry::Vacant(vacant) => {
                let visited = VisitedNode::new(steps, from.clone(), moved.0, moved.1);
                vacant.insert(visited);
                true
            }
        }
    }

    /// Returns the shortest known path to `positions`.
    ///
    /// # Panics
    /// Panics if `positions` has yet to be visited.
    pub fn path_to(&self, positions: &RobotPositions) -> Solution {
        let mut path = Vec::with_capacity(32);
        let mut current_pos = positions.clone();

        // Create the path by following the nodes previous positions.
        loop {
            let current_node = self
                .get_node(&current_pos)
                .expect("Failed to find a supposed source position");
            path.push(current_node.reached_with());
            current_pos = current_node.previous_position().clone();
            if current_node.steps_to_reach() == 1 {
                // current_pos is now the start of the path
                break;
            }
        }

        path.reverse();
        Solution::new(current_pos, positions.clone(), path)
    }
}

#[derive(Debug, Clone, CopyGetters, Getters)]
pub struct VisitedNode {
    #[getset(get_copy = "pub")]
    steps_to_reach: usize,
    #[getset(get = "pub")]
    previous_position: RobotPositions,
    robot: Color,
    direction: Direction,
}

impl VisitedNode {
    pub fn new(
        steps: usize,
        previous_position: RobotPositions,
        robot: Color,
        direction: Direction,
    ) -> Self {
        VisitedNode {
            steps_to_reach: steps,
            previous_position,
            robot,
            direction,
        }
    }

    pub fn reached_with(&self) -> (Color, Direction) {
        (self.robot, self.direction)
    }
}
