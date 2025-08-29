use std::collections::{HashMap, VecDeque};

use math::Vec2i;

pub struct AStar {
    to: Vec2i,
    came_from: HashMap<Vec2i, Vec2i>,
    cost_so_far: HashMap<Vec2i, f32>,
    frontier: VecDeque<(f32, usize, Vec2i)>,
    size: Vec2i
}

impl AStar {

    pub fn new(size: Vec2i, to: Vec2i) -> AStar {
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

    pub fn find_path<F>(&mut self, from: Vec2i, cost: F) where F: Fn(Vec2i) -> MovementCost {
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

    pub fn get_path(&self, from: Vec2i) -> Vec<Vec2i> {

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

    pub fn neighbors(&self, point: Vec2i) -> Vec<Vec2i> {
        let mut neighbors = Vec::new();
        if point.x() >= 1 {
            neighbors.push(point + Vec2i(-1, 0));
        }
        if point.y() >= 1 {
            neighbors.push(point + Vec2i(0, -1));
        }
        if point.x() < self.size.0 as i32 {
            neighbors.push(point + Vec2i(1, 0));
        }
        if point.y() < self.size.1 as i32 {
            neighbors.push(point + Vec2i(0, 1));
        }
        return neighbors;
    }

    fn heuristic(a: Vec2i, b: Vec2i) -> f32 {
        return f32::abs((a.x() - b.x()) as f32) + f32::abs((a.y() - b.y()) as f32);
    }
    
}

pub enum MovementCost {
    Impossible,
    Cost(f32)
}
