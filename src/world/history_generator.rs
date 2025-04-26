use std::{collections::HashMap, time::Instant};

use crate::{commons::{history_vec::Id, rng::Rng}, engine::geometry::Size2D, resources::resources::Resources, world::{date::WorldDate, history_sim::history_simulation::HistorySimulation, topology::{WorldTopology, WorldTopologyGenerationParameters}}};

use super::{culture::Culture, region::Region, world::World};


#[derive(Clone)]
pub(crate) struct WorldGenerationParameters {
    pub(crate) seed: u32,
    pub(crate) cultures: Vec<Culture>,
    pub(crate) regions: Vec<Region>,
}

pub(crate) struct WorldHistoryGenerator {
    pub(crate) year: u32,
    pub(crate) world: World,
    history_sim: HistorySimulation,
}

impl WorldHistoryGenerator {

    pub(crate) fn seed_world(parameters: WorldGenerationParameters, resources: &Resources) -> WorldHistoryGenerator {
        let rng = Rng::seeded(parameters.seed);
       
        let mut params = WorldTopologyGenerationParameters {
            rng: rng.derive("topology"),
            num_plate_tectonics: 25
        };

        let mut world_map = WorldTopology::new(Size2D(256, 256));
        let now = Instant::now();
        world_map.plate_tectonics(&mut params);
        println!("Plate tectonics in {:.2?}", now.elapsed());
        let now: Instant = Instant::now();
        world_map.precipitation(&mut params);
        println!("Precipitation {:.2?}", now.elapsed());
        // let now: Instant = Instant::now();
        // world_map.erosion(&mut params);
        // println!("Erosion {:.2?}", now.elapsed());
        world_map.noise(&rng, &parameters.regions);

        let mut regions = HashMap::new();
        for region in parameters.regions.iter() {
            regions.insert(Id(region.id), region.clone());
        }



        // TODO:
        // let mut culture_id = Id(0);
        // for culture in parameters.cultures.iter() {
        //     let mut culture = culture.clone();
        //     culture.id = culture_id.next();
        //     world.cultures.insert(culture.id, culture);
        // }


        let mut world = World::new(parameters.clone(), world_map, regions);

        let mut history_sim = HistorySimulation::new(crate::world::history_sim::history_simulation::HistorySimParams {
            rng: rng.derive("history"),
            resources: resources.clone(),
            number_of_seed_cities: 100,
            seed_cities_population: 50
        });
        history_sim.seed(&mut world);


        let generator = WorldHistoryGenerator {
            history_sim,
            world,
            year: 1
        };

        return generator;
    }

    pub(crate) fn simulate_year(&mut self) {
        self.history_sim.simulate_step(WorldDate::new(1, 0, 0), &mut self.world);
        self.year = self.history_sim.date.year() as u32;
    }

}
