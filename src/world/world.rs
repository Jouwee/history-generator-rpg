use std::{fs::File, io::Write};

use crate::{engine::geometry::Coord2, game::codex::Codex, info, world::{creature::Profession, history_generator::WorldGenerationParameters, plot::Plots, unit::{UnitId, UnitType}}, Event, Item, Resources};

use super::{creature::{CreatureId, Creatures}, date::WorldDate, lineage::Lineages, topology::WorldTopology, unit::Units};

use crate::commons::id_vec::IdVec;

pub(crate) struct World {
    pub(crate) date: WorldDate,
    pub(crate) generation_parameters: WorldGenerationParameters,
    pub(crate) map: WorldTopology,
    pub(crate) units: Units,
    pub(crate) lineages: Lineages,
    pub(crate) creatures: Creatures,
    pub(crate) plots: Plots,
    pub(crate) events: Vec<Event>,
    pub(crate) artifacts: IdVec<Item>,
    pub(crate) codex: Codex,
    played_creature: Option<CreatureId>
}

impl World {

    pub(crate) fn new(map: WorldTopology, generation_parameters: WorldGenerationParameters) -> World {
        return World {
            date: WorldDate::new(1, 1, 1),
            generation_parameters,
            map,
            units: Units::new(),
            creatures: Creatures::new(),
            lineages: Lineages::new(),
            plots: Plots::new(),
            artifacts: IdVec::new(),
            events: Vec::new(),
            codex: Codex::new(),
            played_creature: None,
        }
    }

    pub(crate) fn create_scenario(&mut self) -> Result<(CreatureId, Coord2), ()> {
        let mut candidate = None;
        'outer: for unit_id in self.units.iter_ids::<UnitId>() {
            let unit = self.units.get(&unit_id);
            if unit.unit_type == UnitType::Village {
                for creature_id in unit.creatures.iter() {
                    let creature = self.creatures.get(creature_id);
                    let age = (self.date - creature.birth).year();
                    if age > 20 && age < 40 && creature.spouse.is_none() && creature.profession == Profession::Peasant {
                        candidate = Some((creature_id.clone(), unit.xy.clone()));
                        break 'outer;
                    }
                }
            }
        }
        if let Some(candidate) = candidate {
            self.played_creature = Some(candidate.0);
            self.codex = Codex::new();

            // Major sites
            for unit_id in self.units.iter_ids::<UnitId>() {
                let unit = self.units.get(&unit_id);
                if unit.creatures.len() > 0 && unit.unit_type == UnitType::Village {
                    self.codex.unit_mut(&unit_id);
                }
            }

            // Information about myself
            let myself = self.codex.creature_mut(&candidate.0);
            myself.add_name();
            myself.add_father();
            myself.add_mother();
            myself.add_birth();
            myself.add_death();
            myself.add_appearance();
            for (i, event) in self.events.iter().enumerate() {
                if event.relates_to_creature(&candidate.0) {
                    myself.add_event(i);
                }
            }

            // Information about my family
            let myself = self.creatures.get(&candidate.0);

            let father = self.codex.creature_mut(&myself.father);
            father.add_name();
            father.add_appearance();
            father.add_birth();
            father.add_death();
            let mother = self.codex.creature_mut(&myself.mother);
            mother.add_name();
            mother.add_appearance();
            mother.add_birth();
            mother.add_death();

            // Information about my relationships
            for another in myself.relationships.iter() {
                let another = self.codex.creature_mut(&another.creature_id);
                another.add_name();
                another.add_appearance();
            }

            return Ok(candidate);
        }
        return Err(());
    }

    pub(crate) fn dump_events(&self, filename: &str, resources: &Resources) {
        let mut f = File::create(filename).unwrap();
        info!("{:?} events", self.events.len());
        for event in self.events.iter() {
            writeln!(&mut f, "{}", event.event_text(resources, &self)).unwrap();
        }
    }

    pub(crate) fn creature_desc(&self, creature_id: &CreatureId, resources: &Resources) -> String {
        let creature = self.creatures.get(creature_id);
        return creature.name(creature_id, &self, resources)
    }


    pub(crate) fn date_desc(&self, date: &WorldDate) -> String {
        return String::from(format!("{}-{}-{}", date.year(), date.month(), date.day()))
    }


    pub(crate) fn is_played_creature(&self, creature_id: &CreatureId) -> bool {
        match &self.played_creature {
            None => false,
            Some(id) => id == creature_id
        }
    }

}

#[cfg(test)]
pub(crate) mod fixture {
    use std::cell::{Ref, RefMut};

    use crate::{engine::geometry::{Coord2, Size2D}, world::{creature::{Creature, CreatureGender, Profession, SIM_FLAG_INTELIGENT}, lineage::Lineage, unit::{Unit, UnitId, UnitResources}}};

    use super::*;

    pub(crate) struct WorldFixture {
        pub(crate) world: World,
        // Alive male human villager
        pub(crate) creature_a1: CreatureId,
        // Alive female human villager
        pub(crate) creature_a2: CreatureId,
        // Alive male human villager
        pub(crate) creature_a3: CreatureId,
        // Alive female human villager
        pub(crate) creature_a4: CreatureId,
    }

    impl WorldFixture {

        pub fn new() -> Self {

            let generation_parameters = WorldGenerationParameters { 
                seed: 0,
                world_size: Size2D(10, 10),
                num_plate_tectonics: 0,
                history_length: 0,
                number_of_seed_cities: 0,
                seed_cities_population: 0,
                st_strength: 0.,
                st_city_count: 0,
                st_city_population: 0,
                st_village_count: 0,
                st_village_population: 0
            };

            let mut world = World::new(WorldTopology::new(Size2D(10, 10)), generation_parameters);

            let mut resources = Resources::new();
            resources.load();

            let human_id = resources.species.id_of("species:human");
            let culture = resources.cultures.random();

            let lineage_1 = world.lineages.add(Lineage::new(culture, resources.cultures.get(&culture)));
            let lineage_2 = world.lineages.add(Lineage::new(culture, resources.cultures.get(&culture)));

            let creature_a1 = world.creatures.add(Creature {
                birth: WorldDate::new(1, 1, 1),
                death: None,
                details: None,
                experience: 0,
                father: CreatureId::ancients(),
                mother: CreatureId::ancients(),
                gender: CreatureGender::Male,
                lineage: Some(lineage_1),
                offspring: Vec::new(),
                profession: Profession::Peasant,
                relationships: Vec::new(),
                sim_flags: SIM_FLAG_INTELIGENT,
                species: human_id,
                spouse: None,
                goals: Vec::new(),
                supports_plot: None,
            });

            let creature_a2 = world.creatures.add(Creature {
                birth: WorldDate::new(1, 1, 1),
                death: None,
                details: None,
                experience: 0,
                father: CreatureId::ancients(),
                mother: CreatureId::ancients(),
                gender: CreatureGender::Female,
                lineage: Some(lineage_2),
                offspring: Vec::new(),
                profession: Profession::Peasant,
                relationships: Vec::new(),
                sim_flags: SIM_FLAG_INTELIGENT,
                species: human_id,
                spouse: None,
                goals: Vec::new(),
                supports_plot: None,
            });

            let creature_a3 = world.creatures.add(Creature {
                birth: WorldDate::new(1, 1, 1),
                death: None,
                details: None,
                experience: 0,
                father: CreatureId::ancients(),
                mother: CreatureId::ancients(),
                gender: CreatureGender::Male,
                lineage: Some(lineage_2),
                offspring: Vec::new(),
                profession: Profession::Peasant,
                relationships: Vec::new(),
                sim_flags: SIM_FLAG_INTELIGENT,
                species: human_id,
                spouse: None,
                goals: Vec::new(),
                supports_plot: None,
            });

            let creature_a4 = world.creatures.add(Creature {
                birth: WorldDate::new(1, 1, 1),
                death: None,
                details: None,
                experience: 0,
                father: CreatureId::ancients(),
                mother: CreatureId::ancients(),
                gender: CreatureGender::Female,
                lineage: Some(lineage_2),
                offspring: Vec::new(),
                profession: Profession::Peasant,
                relationships: Vec::new(),
                sim_flags: SIM_FLAG_INTELIGENT,
                species: human_id,
                spouse: None,
                goals: Vec::new(),
                supports_plot: None,
            });

            let _: UnitId = world.units.add(Unit {
                artifacts: Vec::new(),
                cemetery: Vec::new(),
                name: None,
                creatures: vec!(creature_a1, creature_a2, creature_a3, creature_a4),
                population_peak: (2, 1),
                resources: UnitResources { food: 0. },
                settlement: None,
                unit_type: crate::world::unit::UnitType::Village,
                xy: Coord2::xy(1, 1)
            });

            return WorldFixture {
                world,
                creature_a1,
                creature_a2,
                creature_a3,
                creature_a4,
            }
        }

        pub(crate) fn creature_a1(&'_ self) -> Ref<'_, Creature> {
            return self.world.creatures.get(&self.creature_a1);
        }

        pub(crate) fn creature_a2(&'_ self) -> Ref<'_, Creature> {
            return self.world.creatures.get(&self.creature_a2);
        }

        pub(crate) fn creature_a3(&'_ self) -> Ref<'_, Creature> {
            return self.world.creatures.get(&self.creature_a3);
        }

        pub(crate) fn creature_a4(&'_ self) -> Ref<'_, Creature> {
            return self.world.creatures.get(&self.creature_a4);
        }

        pub(crate) fn creature_a3_mut(&'_ mut self) -> RefMut<'_, Creature> {
            return self.world.creatures.get_mut(&self.creature_a3);
        }

        pub(crate) fn creature_a4_mut(&'_ mut self) -> RefMut<'_, Creature> {
            return self.world.creatures.get_mut(&self.creature_a4);
        }

    }

}