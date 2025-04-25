use std::{borrow::BorrowMut, cell::RefMut, collections::HashMap, time::Instant};

use crate::{commons::{astar::{AStar, MovementCost}, history_vec::{HistoryVec, Id}, id_vec::IdVec, rng::Rng, strings::Strings}, engine::{geometry::{Coord2, Size2D}, Point2D}, resources::resources::Resources, world::{date::WorldDate, faction::{Faction, FactionRelation}, history_sim::{history_simulation::HistorySimulation, structs::World}, item::{Mace, Sword}, map_features::WorldMapFeatures, person::{Importance, NextOfKin, Person, PersonSex, Relative}, topology::{WorldTopology, WorldTopologyGenerationParameters}, world::People}, ArtifactPossesionEvent, CauseOfDeath, MarriageEvent, NewSettlementLeaderEvent, PeaceDeclaredEvent, SettlementFoundedEvent, SimplePersonEvent, WarDeclaredEvent, WorldEventDate, WorldEventEnum, WorldEvents};

use super::{battle_simulator::{BattleForce, BattleResult}, culture::Culture, item::{Item, ItemQuality}, material::MaterialId, person::CivilizedComponent, region::Region, settlement::{Settlement, SettlementBuilder}, species::SpeciesIntelligence, world::ArtifactId};


#[derive(Clone)]
pub struct WorldGenerationParameters {
    pub seed: u32,
    pub cultures: Vec<Culture>,
    pub regions: Vec<Region>,
    pub great_beasts_yearly_spawn_chance: f32,
    pub legendary_artifact_comission_chance: f32
}

pub struct WorldHistoryGenerator {
    rng: Rng,
    pub year: u32,
    parameters: WorldGenerationParameters,
    pub world: World,
    resources: Resources
}

impl WorldHistoryGenerator {

    pub fn seed_world(parameters: WorldGenerationParameters, resources: &Resources) -> WorldHistoryGenerator {
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

        // TODO:


        let world = World::new(parameters.clone(), world_map, regions);

        let mut history_sim = HistorySimulation::new(crate::world::history_sim::history_simulation::HistorySimParams {
            rng: rng.derive("history"),
            resources: resources.clone(),
            number_of_seed_cities: 1,
            seed_cities_population: 200
        }, world);
        history_sim.seed();

        for _ in 0..500 {
            history_sim.simulate_step(WorldDate::new(1, 0, 0));
        }

        history_sim.dump_events("lore.log");

        let generator = WorldHistoryGenerator {
            parameters: parameters,
            resources: resources.clone(),
            rng,
            world: history_sim.into_world(),
            year: 500
        };

        return generator;
    }

    pub fn simulate_year(&mut self) {
        // TODO:
    }

    // fn create_artifact(&mut self, date: WorldEventDate, location: Coord2, material_id: &MaterialId) -> ArtifactId {
    //     let material_id = material_id.clone();
    //     let item;
    //     match self.rng.randu_range(0, 2) {
    //         0 => {
    //             let mut blade = self.resources.materials.id_of("mat:steel");
    //             let mut handle = self.resources.materials.id_of("mat:oak");
    //             let mut guard = self.resources.materials.id_of("mat:bronze");
    //             let mut pommel = self.resources.materials.id_of("mat:bronze");
    //             match self.rng.randu_range(0, 4) {
    //                 1 => blade = material_id,
    //                 2 => guard = material_id,
    //                 3 => handle = material_id,
    //                 _ => pommel = material_id,
    //             }
    //             let mut sword = Sword::new(ItemQuality::Legendary, handle, blade, pommel, guard, &self.resources.materials);
    //             sword.name = Some(self.artifact_name(self.rng.derive("name"), vec!(
    //                 "sword", "blade", "slash", "fang", "tongue", "kiss", "wing", "edge", "talon"
    //             )));
    //             item = Item::Sword(sword)
    //         },
    //         _ => {
    //             let mut head = self.resources.materials.id_of("mat:steel");
    //             let mut handle = self.resources.materials.id_of("mat:oak");
    //             let mut pommel = self.resources.materials.id_of("mat:bronze");
    //             match self.rng.randu_range(0, 3) {
    //                 1 => head = material_id,
    //                 2 => handle = material_id,
    //                 _ => pommel = material_id,
    //             }
    //             let mut mace = Mace::new(ItemQuality::Legendary, handle, head, pommel, &self.resources.materials);
    //             mace.name = Some(self.artifact_name(self.rng.derive("name"), vec!(
    //                 "breaker", "kiss", "fist", "touch"
    //             )));
    //             item = Item::Mace(mace)
    //         }
    //     }
    //     let id = self.world.artifacts.add(item);
    //     self.world.events.push(date, location, WorldEventEnum::ArtifactCreated(crate::ArtifactEvent { item: id }));
    //     return id
    // }

    // fn artifact_name(&self, mut rng: Rng, suffixes: Vec<&str>) -> String {
    //     let preffixes = [
    //         "whisper", "storm", "fire", "moon", "sun", "ice", "raven", "thunder", "flame", "frost", "ember"
    //     ];
    //     let prefix = preffixes[rng.randu_range(0, preffixes.len())];
    //     let suffix = suffixes[rng.randu_range(0, suffixes.len())];
    //     return Strings::capitalize(format!("{prefix}{suffix}").as_str());
    // }

    // fn name_person(&self, mut figure: Person, surname: &Option<String>) -> Person {
    //     if let Some(civ) = &figure.civ {
    //         let culture = self.world.cultures.get(&civ.culture).unwrap();
    //         let first_name;
    //         match figure.sex() {
    //             PersonSex::Male => first_name = culture.first_name_male_model.generate(&self.rng.derive("first_name"), 4, 15),
    //             PersonSex::Female => first_name = culture.first_name_female_model.generate(&self.rng.derive("first_name"), 4, 15)
    //         }
    //         let first_name = Strings::capitalize(&first_name);
    //         let last_name;
    //         match surname {
    //             Some(str) => last_name = String::from(str),
    //             None => last_name = Strings::capitalize(&culture.last_name_model.generate(&self.rng.derive("last_name"), 4, 15))
    //         }
    //         figure.first_name = Some(first_name);
    //         figure.birth_last_name = Some(last_name.clone());
    //         figure.last_name = Some(last_name.clone());
    //     }
    //     return figure
    // }

    // fn spawn_great_beast(&mut self, year: u32) {
    //     let mut species = "species:fiend";
    //     if self.rng.rand_chance(0.3) {
    //         species = "species:leshen";
    //     }
    //     let species = self.resources.species.id_of(species);
    //     let mut suitable_location = None;
    //     'candidates: for _ in 1..10 {
    //         let txy = Coord2::xy(self.rng.randu_range(0, self.world.map.size.x()) as i32, self.rng.randu_range(0, self.world.map.size.y()) as i32);
    //         let tile = self.world.map.tile(txy.x as usize, txy.y as usize);
    //         if tile.region_id == 0 {// Ocean
    //             continue;
    //         }
    //         for (_, settlement) in self.world.settlements.iter() {
    //             if settlement.borrow().xy.to_coord().dist_squared(&txy) < 3.0_f32.powi(2) {
    //                 continue 'candidates;
    //             }
    //         }
    //         suitable_location = Some(txy);
    //         break;
    //     }
    //     if let Some(xy) = suitable_location {
    //         let id = self.next_person_id.next();
    //         self.world.people.insert(Person::new(id, &species, Importance::Important, year, xy));
    //         self.world.events.push(WorldEventDate { year }, xy, WorldEventEnum::PersonBorn(SimplePersonEvent { person_id: id }))
    //     }
    // }

    // fn beast_hunt_nearby(&mut self, date: WorldEventDate, person_id: &Id) {
    //     let beast = self.world.people.get(&person_id).unwrap();
    //     let mut rng = self.rng.derive("beast_attack");
    //     let xy = beast.position + Coord2::xy(rng.randi_range(-15, 15), rng.randi_range(-15, 15));
    //     let mut result = None;
    //     if let Some((sett_id, settlement)) = self.world.settlements.iter().find(|(_, sett)| sett.borrow().xy.to_coord() == xy) {
    //         let mut creature_force = BattleForce::from_creatures(&self.resources, vec!(&beast));
    //         let mut settlement_force = BattleForce::from_defending_settlement(&self.world, &self.resources, sett_id, &settlement.borrow());
    //         let battle = creature_force.battle(&mut settlement_force, &mut rng, settlement.borrow().xy.to_coord(), sett_id);
    //         result = Some(battle);
    //     }
    //     drop(beast);
    //     if let Some(battle) = result {
    //         self.apply_battle_result(date, battle);
    //     }
    // }

    // fn create_simple_artifact(&mut self, date: WorldEventDate, creature_id: &Id) {
    //     let position = self.world.people.get(&creature_id).unwrap().position.clone();
    //     let artifact_id = self.create_artifact(date, position, &self.resources.materials.id_of("mat:steel"));
    //     let mut creature = self.world.people.get_mut(&creature_id).unwrap();
    //     creature.possesions.push(artifact_id);
    //     creature.importance = creature.importance.at_least(&Importance::Unimportant);
    //     self.world.events.push(date, position, WorldEventEnum::ArtifactPossession(ArtifactPossesionEvent { item: artifact_id, person: *creature_id }));

    // }


    // fn colonize_new_settlement(&mut self, date: WorldEventDate, id: Id) {
    //     let mut person = self.world.people.get_mut(&id).unwrap();
    //     let xy = person.position.clone();
    //     if let Some(civ) = &mut person.civ {
    //         let culture = self.world.cultures.get(&civ.culture).unwrap();
    //         let settlement = generate_settlement(&self.rng, date.year, xy, id.clone(), culture, civ.faction, &self.world, &self.world.map, &self.parameters.regions).clone();
    //         if let Some(settlement) = settlement {
    //             let position = settlement.xy;
    //             let id = self.world.settlements.insert(settlement);
    //             self.world.events.push(date, position.to_coord(), WorldEventEnum::SettlementFounded(SettlementFoundedEvent { settlement_id: id, founder_id: id }));
    //             let mut faction = self.world.factions.get_mut(&civ.faction);
    //             faction.settlements.insert(id);
    //             civ.leader_of_settlement = Some(id);
    //             if let Some(spouse) = person.spouse() {
    //                 let mut spouse = self.world.people.get_mut(spouse).unwrap();
    //                 let spouse = spouse.borrow_mut();
    //                 (*spouse).position = position.to_coord();
    //             }
    //             person.position = position.to_coord();

    //             // Generate road to nearby settlements
    //             let mut settlements_to_connect = Vec::new();
    //             for (sid, sett) in self.world.settlements.iter() {
    //                 if let Ok(sett) = sett.try_borrow() {
    //                     if sid != id && sett.xy.dist_squared(&position) < 10.*10. {
    //                         settlements_to_connect.push(sett.xy.to_coord());
    //                     }
    //                 }
    //             }
    //             let from = position.to_coord();
    //             for to in settlements_to_connect {
    //                 let mut astar = AStar::new(self.world.map.size, to);
    //                 astar.find_path(from, |p| {
    //                     if !self.world.map.size.in_bounds(p) {
    //                         return MovementCost::Impossible;
    //                     }
    //                     if self.world.map_features.has_road(p) {
    //                         return MovementCost::Cost(0.);
    //                     }
    //                     let region = self.world.map.tile(p.x as usize, p.y as usize).region_id;
    //                     match region {
    //                         0 => MovementCost::Impossible, // Ocean
    //                         1 => MovementCost::Cost(2.0), // Coastal
    //                         2 => MovementCost::Cost(0.5), // Grassland
    //                         3 => MovementCost::Cost(5.0), // Forest
    //                         4 => MovementCost::Cost(2.0), // Desert
    //                         _ => MovementCost::Cost(1.0)
    //                     }
    //                 });
    //                 let path = astar.get_path(from);
    //                 for point in path {
    //                     self.world.map_features.add_road(point);
    //                 }
    //             }

    //         }
    //     }
    // }

}

// fn generate_settlement(rng: &Rng, founding_year: u32, seed_pos: Coord2, leader: Id, culture: &Culture, faction: Id, world_graph: &World, world_map: &WorldTopology, regions: &Vec<Region>) -> Option<Settlement> {
//     let mut rng = rng.derive("settlement");
//     let mut xy = None;
//     let dist = 25;
//     let x = ((seed_pos.x - dist).clamp(0, world_map.size.x() as i32 - 1))..((seed_pos.x + dist).clamp(0, world_map.size.x() as i32 - 1));
//     let y = ((seed_pos.y - dist).clamp(0, world_map.size.y() as i32 - 1))..((seed_pos.y + dist).clamp(0, world_map.size.y() as i32 - 1));
//     'candidates: for _ in 1..20 {
//         let txy = Point2D(rng.randu_range(x.start as usize, x.end as usize), rng.randu_range(y.start as usize, y.end as usize));
//         let tile = world_graph.map.tile(txy.0, txy.1);
//         if tile.region_id == 0 {// Ocean
//             continue;
//         }
//         for (_, settlement) in world_graph.settlements.iter() {
//             if settlement.borrow().xy.dist_squared(&txy) <= 2_f32.powi(2) {
//                 continue 'candidates;
//             }
//         }
//         xy = Some(txy);
//         break;
//     }
//     if let Some(xy) = xy {
//         let region_id = world_map.tile(xy.0, xy.1).region_id as usize;
//         let region = regions.get(region_id).unwrap();
//         return Some(SettlementBuilder::colony(&rng, xy, founding_year, leader, culture, faction, region).create())
//     } else {
//         None
//     }
// }
// enum ActionToSimulate {
//     None,
//     Death(Id),
//     GreatBeastHunt(Id),
//     ComissionArtifact(Id),
//     MarryRandomPerson(Id),
//     HaveChildWith(Id, Id),
//     ColonizeNewSettlement(Id)
// }