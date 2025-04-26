use std::collections::{HashMap, VecDeque};

use crate::engine::geometry::{Coord2, Size2D};


pub(crate) struct AStar {
    to: Coord2,
    came_from: HashMap<Coord2, Coord2>,
    cost_so_far: HashMap<Coord2, f32>,
    frontier: VecDeque<Coord2>,
    size: Size2D
}

impl AStar {

    pub(crate) fn new(size: Size2D, to: Coord2) -> AStar {
        let mut astar = AStar {
            to,
            came_from: HashMap::new(),
            cost_so_far: HashMap::new(),
            frontier: VecDeque::new(),
            size
        };
        astar.frontier.push_front(to);
        astar.cost_so_far.insert(to, 0.);
        return astar
    }

    pub(crate) fn find_path<F>(&mut self, from: Coord2, cost: F) where F: Fn(Coord2) -> MovementCost {
        while !self.frontier.is_empty() {
            let current = self.frontier.pop_front().unwrap();
            
            if current == from {
                break
            }
            
            for next in self.neighbors(current) {
                let cost = cost(next);
                match cost {
                    MovementCost::Impossible => (),
                    MovementCost::Cost(cost) => {
                        let new_cost = self.cost_so_far.get(&current).unwrap() + cost;
                        if !self.cost_so_far.contains_key(&next) || new_cost < *self.cost_so_far.get(&next).unwrap() {
                            self.cost_so_far.insert(next, new_cost);
                            // TODO: Using priotity as sorting should be faster, but inserting sorted is too costly
                            // let priority = new_cost + Self::heuristic(next, from);
                            self.frontier.push_back(next); // P = priority
                            self.came_from.insert(next, current);
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn get_path(&self, from: Coord2) -> Vec<Coord2> {

        let mut current = from;
        let mut path = Vec::new();
        if !self.came_from.contains_key(&from) { // no path was found
            return path
        }
        while current != self.to {
            path.push(current);
            current = *self.came_from.get(&current).unwrap();
        }
        path.push(self.to);
        // path.reverse();
        
        return path
        
        // return came_from, cost_so_far
    }

    pub(crate) fn neighbors(&self, point: Coord2) -> Vec<Coord2> {
        let mut neighbors = Vec::new();
        if point.x >= 1 {
            neighbors.push(point + Coord2::xy(-1, 0));
        }
        if point.y >= 1 {
            neighbors.push(point + Coord2::xy(0, -1));
        }
        if point.x < self.size.0 as i32 {
            neighbors.push(point + Coord2::xy(1, 0));
        }
        if point.y < self.size.1 as i32 {
            neighbors.push(point + Coord2::xy(0, 1));
        }
        return neighbors;
    }

    // fn heuristic(a: Coord2, b: Coord2) -> f32 {
    //     return f32::abs((a.x - b.x) as f32) + f32::abs((a.y - b.y) as f32);
    // }
    
}

pub(crate) enum MovementCost {
    Impossible,
    Cost(f32)
}
