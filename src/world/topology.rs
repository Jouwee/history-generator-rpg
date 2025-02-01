use std::{collections::{BTreeMap, VecDeque}, f32::consts::PI};

use noise::{NoiseFn, Perlin};
use crate::{commons::{matrix_index::MatrixIndex, rng::Rng}, engine::{geometry::{Size2D, Vec2, Vector2}, Point2D}};

use super::region::Region;

pub struct WorldTopology {
    pub size: Size2D,
    pub elevation: Vec<i32>,
    pub precipitation: Vec<u8>,
    pub temperature: Vec<u8>,
    pub vegetation: Vec<f32>,
    pub soil_ferility: Vec<f32>,
    pub region_id: Vec<u8>
}

impl WorldTopology {

    pub fn new(size: Size2D) -> WorldTopology {
        let len = size.area();
        WorldTopology { 
            size,
            elevation: vec![0; len],
            precipitation: vec![0; len],
            temperature: vec![0; len],
            vegetation: vec![0.0; len],
            soil_ferility: vec![0.0; len],
            region_id: vec![0; len]
        }
    }

    pub fn tile(&self, x: usize, y: usize) -> WorldTileData {
        let i = (y * self.size.x()) + x;
        return WorldTileData {
            xy: Point2D(x, y),
            elevation: self.elevation[i],
            precipitation: self.precipitation[i],
            temperature: self.temperature[i],
            vegetation: self.vegetation[i],
            soil_fertility: self.soil_ferility[i],
            region_id: self.region_id[i],
        }
    }

    pub fn plate_tectonics(&mut self, params: &mut WorldTopologyGenerationParameters) {
        let idx = MatrixIndex::new((self.size.0, self.size.1));
        let noise = Perlin::new(params.rng.derive("noise").seed());
        let mut plate_map = vec![0; self.size.area()];
        struct PlateTectonics {
            seed: Point2D,
            base_elevation: i32,
            direction: Vector2
        }
        // Generate plates and origins
        let mut plates = BTreeMap::new();
        for i in 1..params.num_plate_tectonics+1 {
            let seed = Point2D(params.rng.randu_range(0, self.size.x()), params.rng.randu_range(0, self.size.y()));
            let dist_to_edge = seed.dist_squared(&Point2D(0, 0))
                .min(seed.dist_squared(&Point2D(self.size.x(), 0)))
                .min(seed.dist_squared(&Point2D(0, self.size.y())))
                .min(seed.dist_squared(&Point2D(self.size.x(), self.size.y())));
            let dist_to_edge = (dist_to_edge.sqrt() / (self.size.x() / 2) as f32).clamp(0., 1.); // TODO: Use diagonal instead of clamp
            let oceanic_plate = params.rng.rand_chance(1. - dist_to_edge.powf(3.));
            let elevation;
            if oceanic_plate {
                elevation = params.rng.randf_range(-64., -32.) as i32;
            } else {
                elevation = params.rng.randf_range(32., 64.) as i32;
            }
            plates.insert(i, PlateTectonics {
                base_elevation: elevation,
                seed,
                direction: Vector2::new(params.rng.randf_range(0., 2.*PI), params.rng.randf())
            });
        }
        // Flood-fill
        let mut ff_queue = VecDeque::new();
        for (i, v) in plates.iter() {
            ff_queue.push_back((v.seed, *i));
        }
        let shuffle_frequency = params.rng.randu_range(5, 13);
        let mut i_f = 0;
        while ff_queue.len() > 0 {
            let (point, id) = ff_queue.pop_front().unwrap();
            let i = idx.idx(point.0, point.1);
            if plate_map[i] != 0 {
                continue
            }
            plate_map[i] = id;
            if point.0 > 0 && plate_map[idx.idx(point.0 - 1, point.1)] == 0 {
                ff_queue.push_back((Point2D(point.0 - 1, point.1), id));
            }
            if point.0 < self.size.x() - 1 && plate_map[idx.idx(point.0 + 1, point.1)] == 0 {
                ff_queue.push_back((Point2D(point.0 + 1, point.1), id));
            }
            if point.1 > 0 && plate_map[idx.idx(point.0, point.1 - 1)] == 0 {
                ff_queue.push_back((Point2D(point.0, point.1 - 1), id));
            }
            if point.1 < self.size.y() - 1 && plate_map[idx.idx(point.0, point.1 + 1)] == 0 {
                ff_queue.push_back((Point2D(point.0, point.1 + 1), id));
            }
            if i_f % shuffle_frequency == 0 {
                // Shuffles the queue ever so slightly
                let pop = ff_queue.pop_back();
                if let Some(pop) = pop {
                    ff_queue.push_front(pop);
                }
            }
            i_f += 1;
        }
        // Boundary checks
        enum Boundary {
            Transverse,
            Convergent(f32),
            Divergent(f32)
        }
        let mut boundaries = Vec::new();
        for y in 1..self.size.y() {
            for x in 1..self.size.x() {
                let i = (y * self.size.x()) + x;
                let mut i2 = (y * self.size.x()) + x - 1;
                if plate_map[i] == plate_map[i2] {
                    i2 = ((y-1) * self.size.x()) + x;
                    if plate_map[i] == plate_map[i2] {
                        continue
                    }
                }
                let plate1 = plates.get(&plate_map[i]).unwrap();
                let plate2 = plates.get(&plate_map[i2]).unwrap();
                // Ref: http://blog.procgenesis.com/
                let angle_diff = (plate1.direction.angle - plate2.direction.angle).abs();
                let avg_mag = (plate1.direction.magnitude + plate2.direction.magnitude) / 2.;
                let shear_force: f32 = f32::sin(angle_diff) * avg_mag;
                let direct_force: f32 = f32::cos(angle_diff) * avg_mag;
                if shear_force.abs() > direct_force.abs() {
                    boundaries.push((Point2D(x, y), Boundary::Transverse));
                } else if direct_force > 0. {
                    boundaries.push((Point2D(x, y), Boundary::Convergent(direct_force)));
                } else {
                    boundaries.push((Point2D(x, y), Boundary::Divergent(direct_force.abs())));
                }
            }
        }
        // Add plate boundary heights
        for b in boundaries.iter() {
            let i = (b.0.1 * self.size.x()) + b.0.0;
            let noise = noise.get([b.0.0 as f64 / 2., b.0.1 as f64 / 2.]) as f32;
            let noise = (noise + 1.) / 2.;
            match b.1 {
                Boundary::Convergent(strength) => self.elevation[i] += (128. * strength * noise) as i32,
                Boundary::Divergent(strength) => self.elevation[i] -= (128. * strength * noise) as i32,
                Boundary::Transverse => (),
            }
        }
        // Adds the base elevation of each plate
        for y in 0..self.size.y() {
            for x in 0..self.size.x() {
                let i = (y * self.size.x()) + x;
                if let Some(plate) = plates.get(&plate_map[i]) {
                    self.elevation[i] = self.elevation[i] as i32 + plate.base_elevation as i32;
                }
            }
        }
        // Smoothing pass
        let mask = [1., 2., 8.];
        let sum = mask[0] * 4. + mask[1] * 4. + mask[2];
        let unit_value = 9. / sum;
        let mask = [mask[0] * unit_value, mask[1] * unit_value, mask[2] * unit_value];
        for y in 1..self.size.y() - 1 {
            for x in 1..self.size.x() - 1 {
                let sum =
                    (mask[1]  * self.elevation[(y * self.size.x()) + x + 1] as f32) + 
                    (mask[2]  * self.elevation[(y * self.size.x()) + x] as f32) +
                    (mask[1]  * self.elevation[(y * self.size.x()) + x - 1] as f32) +
                    (mask[0] * self.elevation[((y + 1) * self.size.x()) + x + 1] as f32) + 
                    (mask[1]  * self.elevation[((y + 1) * self.size.x()) + x] as f32) +
                    (mask[0] * self.elevation[((y + 1) * self.size.x()) + x - 1] as f32) +
                    (mask[0] * self.elevation[((y - 1) * self.size.x()) + x + 1] as f32) + 
                    (mask[1]  * self.elevation[((y - 1) * self.size.x()) + x] as f32) +
                    (mask[0] * self.elevation[((y - 1) * self.size.x()) + x - 1] as f32);
                self.elevation[(y * self.size.x()) + x] = (sum / 9.) as i32;
            }
        }
        // Noise pass
        for y in 1..self.size.y() - 1 {
            for x in 1..self.size.x() - 1 {
                let i: usize = (y * self.size.x()) + x;
                // Domain warping
                let x = x as f64 + noise.get([x as f64 / 20., y as f64 / 20.]) * 8.;
                let y = y as f64 + noise.get([x as f64 / 20., y as f64 / 20., 1000.]) * 8.;
                //
                let low_freq = noise.get([x / 20., y / 20.]) as f32;
                let med_freq = noise.get([x / 7., y / 7.]) as f32;
                let high_freq = noise.get([x / 1., y / 1.]) as f32;
                let noise = ((low_freq * 0.6 + med_freq * 0.3 + high_freq * 0.1) * 16.) as i32;
                self.elevation[i] = self.elevation[i] as i32 + noise;
            }
        }
    }

    pub fn precipitation(&mut self, params: &mut WorldTopologyGenerationParameters) {
        let idx = MatrixIndex::new((self.size.0, self.size.1));
        let noise = Perlin::new(params.rng.derive("noise").seed());
        for y in 0..self.size.y() {
            for x in 0..self.size.x() {
                let i = idx.idx(x, y);
                let noise = noise.get([x as f64 / 50., y as f64 / 50.]);
                let noise = (noise + 1.) / 2.;
                self.precipitation[i] = (noise * 256.) as u8;
            }
        }
    }

    pub fn erosion(&mut self, params: &mut WorldTopologyGenerationParameters) {
        let idx = MatrixIndex::new((self.size.0, self.size.1));
        // Temporary f32 elevation map
        let mut elevation = vec![0.; self.size.area()];
        for i in 0..self.size.area() {
            elevation[i] = self.elevation[i] as f32;
        }

        for i in 0..100000 {
            let mut xy = Vec2::xy(params.rng.randf_range(0., self.size.0 as f32), params.rng.randf_range(0., self.size.1 as f32));
            let mut momentum = Vec2::xy(params.rng.randf_range(-1., 1.), params.rng.randf_range(-1., 1.));
            let mut carrying = 0.;

            let pickup = 0.5;
            let drop_off = pickup * 0.1;
            let max_carry = 5.;

            for _ in 0..100 {
                let xyp = Point2D(xy.x as usize, xy.y as usize);
                let neighbours = idx.p2d_neighbours8(xyp);

                // println!("{:?}", xyp);

                if neighbours.len() == 0 {
                    panic!("{:?} {:?}", xy, xyp);
                }

                let mut lowest = neighbours[0];
                for neighbour in neighbours {
                    if elevation[neighbour] < elevation[lowest] {
                        lowest = neighbour;
                    } else if elevation[neighbour] == elevation[lowest] && i % 2 == 0 { // "Randomness" without actual randomness for speed
                        lowest = neighbour;
                    }
                }

                let lowestp = idx.to_p2d(lowest);
                let slope = Vec2::xy(lowestp.0 as f32, lowestp.1 as f32) - xy;
                //slope.magnitude = elevation[idx.p2d(xy)] as f32 - elevation[lowest] as f32;
                momentum = momentum + slope;

                // println!("from: {:?} to: {:?} slope: {:?}", xy, Vec2::xy(lowestp.0 as f32, lowestp.1 as f32), slope);

                // println!("momentum: {:?}, slope: {:?}", momentum, slope);
                
                momentum = Vec2::xy(momentum.x * 0.9, momentum.y * 0.9);

                if momentum.magnitude() < 0.01 {
                    elevation[idx.p2d(xyp)] += carrying * 0.5;
                    break;
                }

                let limited = Vec2::xy(momentum.x.clamp(-1., 1.), momentum.y.clamp(-1., 1.));

                let new_xy = xy + limited;

                // carrying += pickup;
                // elevation[idx.p2d(xyp)] -= pickup;

                let drop = f32::min(drop_off, carrying);
                carrying -= drop;
                elevation[idx.p2d(xyp)] += drop;
                if new_xy.x < 0. || new_xy.y < 0. || new_xy.x >= self.size.x() as f32 || new_xy.y >= self.size.y() as f32 {
                    break;
                }
                // if new_xy != xy {
                    let pick = f32::min(pickup, max_carry - carrying);
                    carrying += pick;
                    elevation[idx.p2d(xyp)] -= pick;
                // }
                // if carrying == 0. {
                //     break;
                // }
                xy = new_xy;
            }
        }
        // Saves the f32 vector back
        for i in 0..self.size.area() {
            self.elevation[i] = elevation[i] as i32;
        }
    }

    pub fn noise(&mut self, rng: &Rng, regions: &Vec<Region>) {
        let rng = rng.derive("world_map");
        let n_temp = Perlin::new(rng.derive("temperature").seed());
        let n_reg = Perlin::new(rng.derive("region").seed());
        let n_fert = Perlin::new(rng.derive("fertility").seed());
        for y in 0..self.size.y() {
            for x in 0..self.size.x() {
                let i = (y * self.size.x()) + x;
                let xf = x as f64;
                let yf = y as f64;
                {
                    // Domain warping
                    let x = xf + n_temp.get([xf / 20., yf / 20.]) * 8.;
                    let y = yf + n_temp.get([xf / 20., yf / 20., 1000.]) * 8.;
                    let low = n_temp.get([x / 100.0, y / 100.0]);
                    let med = n_temp.get([x / 10.0, y / 10.0]);
                    let noise = (low * 0.8) + (med * 0.2);
                    self.temperature[i] = (noise * 6.0) as u8;
                }
                {
                    let mut region_candidates: Vec<u8> = Vec::new();
                    for (j, region) in regions.iter().enumerate() {
                        if self.elevation[i] >= region.elevation.0 && self.elevation[i] <= region.elevation.1 && self.temperature[i] >= region.temperature.0 && self.temperature[i] <= region.temperature.1 {
                            region_candidates.push(j as u8);
                        }
                    }
                    match region_candidates.len() {
                        0 => panic!("No region candidate for elevation {} and temperature {}", self.elevation[i], self.temperature[i]),
                        1 => self.region_id[i] = region_candidates.pop().expect("Already checked"),
                        _ => {
                            let low = n_reg.get([xf / 30.0, yf / 30.0]) / 2. + 0.5;
                            let med = n_reg.get([xf / 10.0, yf / 10.0]) / 2. + 0.5;
                            let noise = (low * 0.8) + (med * 0.2);
                            // TODO: This crashes without the mod aparently
                            self.region_id[i] = region_candidates[(noise * region_candidates.len() as f64) as usize % region_candidates.len()];
                        }
                    }
                }
                {
                    let region = &regions[self.region_id[i] as usize];
                    let region_fertility_range = region.soil_fertility_range;
                    let region_vegetation_range = region.vegetation;
                    let noise_modif = n_fert.get([xf / 10.0, yf / 10.0]) as f32;
                    let noise_modif = (noise_modif + 1.0) / 2.0;
                    self.soil_ferility[i] = noise_modif * (region_fertility_range.1 - region_fertility_range.0) + region_fertility_range.0;
                    self.vegetation[i] = noise_modif * (region_vegetation_range.1 - region_vegetation_range.0) + region_vegetation_range.0;
                }
            }
        }
    }

}

pub struct WorldTopologyGenerationParameters {
    pub rng: Rng,
    pub num_plate_tectonics: u8
}

#[derive(Debug)]
pub struct WorldTileData {
    pub xy: Point2D,
    pub elevation: i32,
    pub precipitation: u8,
    pub temperature: u8,
    pub vegetation: f32,
    pub soil_fertility: f32,
    pub region_id: u8
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::geometry::Size2D;


    #[test]
    fn check_determinism() {

        let rng = Rng::seeded("test");
        let mut params_a = WorldTopologyGenerationParameters {
            rng: rng.derive("topology"),
            num_plate_tectonics: 25
        };
        let mut params_b = WorldTopologyGenerationParameters {
            rng: rng.derive("topology"),
            num_plate_tectonics: 25
        };

        let mut world_a = WorldTopology::new(Size2D(32, 32));
        let mut world_b = WorldTopology::new(Size2D(32, 32));

        world_a.plate_tectonics(&mut params_a);
        world_b.plate_tectonics(&mut params_b);
        compare(&world_a, &world_b);

        world_a.precipitation(&mut params_a);
        world_b.precipitation(&mut params_b);
        compare(&world_a, &world_b);

        let regions = vec!(Region {
            id: 0,
            name: String::from("Ocean"),
            elevation: (-2000, 0),
            temperature: (0, 5),
            vegetation: (0.0, 0.0),
            soil_fertility_range: (0.8, 1.2),
            gold_generation_range: (0.8, 1.2),
            fauna: Vec::from([
                String::from("whale"),
                String::from("fish")
            ]),
            flora: Vec::from([
                String::from("kelp"),
                String::from("coral")
            ])
        },
        Region {
            id: 1,
            name: String::from("Coastal"),
            elevation: (0, 2000),
            temperature: (0, 5),
            vegetation: (0.0, 0.1),
            soil_fertility_range: (0.8, 1.2),
            gold_generation_range: (0.8, 1.2),
            fauna: Vec::from([
                String::from("whale"),
                String::from("fish")
            ]),
            flora: Vec::from([
                String::from("kelp"),
                String::from("coral")
            ])
        });

        world_a.noise(&params_a.rng, &regions);
        world_b.noise(&params_b.rng, &regions);
        compare(&world_a, &world_b);

    }

    fn compare(world_a: &WorldTopology, world_b: &WorldTopology) {
        for i in 0..world_a.size.area() {
            assert_eq!(world_a.elevation[i], world_b.elevation[i]);
            assert_eq!(world_a.precipitation[i], world_b.precipitation[i]);
            assert_eq!(world_a.temperature[i], world_b.temperature[i]);
            assert_eq!(world_a.vegetation[i], world_b.vegetation[i]);
            assert_eq!(world_a.soil_ferility[i], world_b.soil_ferility[i]);
            assert_eq!(world_a.region_id[i], world_b.region_id[i]);
        }
    }
}