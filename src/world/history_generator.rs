use std::time::Instant;

use math::rng::Rng;
use serde::{Deserialize, Serialize};

use crate::{commons::rng::Rng as OldRng, engine::geometry::Size2D, info, resources::resources::Resources, world::{date::Duration, history_sim::history_simulation::HistorySimulation, topology::WorldTopology}};

use super::world::World;

#[derive(Clone, Serialize, Deserialize)]

pub(crate) struct WorldGenerationParameters {
    pub(crate) seed: u64,
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
    /// Target number of cities
    pub(crate) st_city_count: u16,
    /// Target city population
    pub(crate) st_city_population: u16,
    /// Target number of villages
    pub(crate) st_village_count: u16,
    /// Target villages population
    pub(crate) st_village_population: u16,
}

impl WorldGenerationParameters {

    pub(crate) fn rng(&self) -> Rng {
        Rng::new(self.seed)
    }

}

pub(crate) struct WorldHistoryGenerator {
    pub(crate) world: World,
    pub(crate) parameters: WorldGenerationParameters,
    history_sim: HistorySimulation,
    pub(crate) stop: bool
}

impl WorldHistoryGenerator {

    pub(crate) fn seed_world(parameters: WorldGenerationParameters, resources: &Resources) -> WorldHistoryGenerator {
        let mut rng = OldRng::seeded(parameters.seed);

        let mut world_map = WorldTopology::new(parameters.world_size);
        let now = Instant::now();
        world_map.plate_tectonics(&mut rng, parameters.num_plate_tectonics);
        info!("Plate tectonics in {:.2?}", now.elapsed());
        let now: Instant = Instant::now();
        world_map.precipitation(&mut rng);
        info!("Precipitation {:.2?}", now.elapsed());
        // let now: Instant = Instant::now();
        // world_map.erosion(&mut params);
        // info!("Erosion {:.2?}", now.elapsed());
        world_map.noise(&rng, &resources.biomes);

        let mut world = World::new(world_map, parameters.clone());

        let mut history_sim = HistorySimulation::new(rng.derive("history"), parameters.clone());
        history_sim.seed(&mut world);


        let generator = WorldHistoryGenerator {
            parameters,
            history_sim,
            world,
            stop: false,
        };

        return generator;
    }

    pub(crate) fn simulator(world: World) -> WorldHistoryGenerator {
        let parameters = world.generation_parameters.clone();

        let rng = OldRng::seeded(parameters.seed);

        let history_sim = HistorySimulation::new(rng.derive("history"), parameters.clone());

        let generator = WorldHistoryGenerator {
            parameters,
            history_sim,
            world,
            stop: false,
        };

        return generator;
    }

    pub(crate) fn simulate_step(&mut self, step: Duration) {
        self.stop = !self.history_sim.simulate_step(step, &mut self.world);
    }

}
