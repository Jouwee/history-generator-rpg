use std::collections::{HashMap, VecDeque};

use crate::engine::geometry::{Coord2, Size2D};


pub(crate) struct AStar {
    to: Coord2,
    came_from: HashMap<Coord2, Coord2>,
    cost_so_far: HashMap<Coord2, f32>,
    frontier: VecDeque<(f32, usize, Coord2)>,
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
        astar.frontier.push_front((0., 0, to));
        astar.cost_so_far.insert(to, 0.);
        return astar
    }

    pub(crate) fn to(&self) -> &Coord2 {
        return &self.to;
    }

    pub(crate) fn find_path<F>(&mut self, from: Coord2, cost: F) where F: Fn(Coord2) -> MovementCost {
        while !self.frontier.is_empty() {
            let (_, depth, current) = self.frontier.pop_front().unwrap();
            
            if current == from {
                break
            }

            // Reverses the neighbour order every other step, so the path isn't always an L shape. Purely visual
            let mut neighbors = self.neighbors(current);
            if depth % 2 == 0 {
                neighbors.reverse();
            }
            
            for next in neighbors {
                let cost = cost(next);
                match cost {
                    MovementCost::Impossible => (),
                    MovementCost::Cost(cost) => {
                        let new_cost = self.cost_so_far.get(&current).unwrap() + cost;
                        if !self.cost_so_far.contains_key(&next) || new_cost < *self.cost_so_far.get(&next).unwrap() {
                            self.cost_so_far.insert(next, new_cost);
                            
                            // Adds sorted by heuristic
                            let priority = new_cost + Self::heuristic(next, from);
                            let u = self.frontier.binary_search_by(|(p, _, _)| p.total_cmp(&priority));
                            match u {
                                Ok(u) | Err(u) => self.frontier.insert(u, (priority, depth + 1, next)),
                            }

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
        return path
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

    fn heuristic(a: Coord2, b: Coord2) -> f32 {
        return f32::abs((a.x - b.x) as f32) + f32::abs((a.y - b.y) as f32);
    }
    
}

pub(crate) enum MovementCost {
    Impossible,
    Cost(f32)
}
