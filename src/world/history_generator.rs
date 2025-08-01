use std::time::Instant;

use crate::{commons::rng::Rng, engine::geometry::Size2D, resources::resources::Resources, world::{date::WorldDate, history_sim::history_simulation::HistorySimulation, topology::WorldTopology}};

use super::world::World;

#[derive(Clone)]
pub(crate) struct WorldGenerationParameters {
    pub(crate) seed: u32,
    // Terain
    /// Size of the world, in chunks
    pub(crate) world_size: Size2D,
    pub(crate) num_plate_tectonics: u8,
    // History
    /// Number of years to simulate
    pub(crate) history_length: u16,
    pub(crate) number_of_seed_cities: u16,
    pub(crate) seed_cities_population: u32,
    // Storyteller settings
    /// Storyteller strength, from 0. to 1.
    pub(crate) st_strength: f32,
    /// Target city population
    pub(crate) st_city_population: u16,
}

pub(crate) struct WorldHistoryGenerator {
    pub(crate) year: u32,
    pub(crate) world: World,
    pub(crate) parameters: WorldGenerationParameters,
    history_sim: HistorySimulation,
    pub(crate) stop: bool
}

impl WorldHistoryGenerator {

    pub(crate) fn seed_world(parameters: WorldGenerationParameters, resources: &Resources) -> WorldHistoryGenerator {
        let mut rng = Rng::seeded(parameters.seed);

        let mut world_map = WorldTopology::new(parameters.world_size);
        let now = Instant::now();
        world_map.plate_tectonics(&mut rng, parameters.num_plate_tectonics);
        println!("Plate tectonics in {:.2?}", now.elapsed());
        let now: Instant = Instant::now();
        world_map.precipitation(&mut rng);
        println!("Precipitation {:.2?}", now.elapsed());
        // let now: Instant = Instant::now();
        // world_map.erosion(&mut params);
        // println!("Erosion {:.2?}", now.elapsed());
        world_map.noise(&rng, &resources.biomes);

        let mut world = World::new(world_map);

        let mut history_sim = HistorySimulation::new(rng.derive("history"), resources.clone(), parameters.clone());
        history_sim.seed(&mut world);


        let generator = WorldHistoryGenerator {
            parameters,
            history_sim,
            world,
            year: 1,
            stop: false,
        };

        return generator;
    }

    pub(crate) fn simulate_year(&mut self) {
        self.stop = !self.history_sim.simulate_step(WorldDate::new(1, 0, 0), &mut self.world);
        self.year = self.history_sim.date.year() as u32;
    }

}
