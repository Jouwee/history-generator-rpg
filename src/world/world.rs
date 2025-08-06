use std::{collections::HashMap, fs::File, io::Write};

use crate::{game::codex::Codex, world::{item::ItemId, plot::Plots}, Event, Item, Resources};

use super::{creature::{CreatureId, Creatures}, date::WorldDate, lineage::Lineages, topology::WorldTopology, unit::Units};

use crate::commons::id_vec::IdVec;

pub(crate) struct World {
    pub(crate) date: WorldDate,
    pub(crate) map: WorldTopology,
    pub(crate) units: Units,
    pub(crate) lineages: Lineages,
    pub(crate) creatures: Creatures,
    pub(crate) plots: Plots,
    pub(crate) events: Vec<Event>,
    pub(crate) artifacts: IdVec<Item>,
    pub(crate) codex: Codex,

}

impl World {

    pub(crate) fn new(map: WorldTopology) -> World {
        return World {
            date: WorldDate::new(1, 1, 1),
            map,
            units: Units::new(),
            creatures: Creatures::new(),
            lineages: Lineages::new(),
            plots: Plots::new(),
            artifacts: IdVec::new(),
            events: Vec::new(),
            codex: Codex::new(),
        }
    }

    pub(crate) fn find_goal(&mut self, resources: &mut Resources) {
        let mut artifact = None;

        let mut events_per_item = HashMap::new();
        for event in self.events.iter() {
            let event_score = match event {
                Event::CreatureDeath { date: _, creature_id: _, cause_of_death: _ } => 0.5,
                _ => 0.01
            };
            for item_id in event.related_artifacts() {
                match events_per_item.get(&item_id) {
                    Some(v) => events_per_item.insert(item_id, *v + event_score),
                    None => events_per_item.insert(item_id, event_score),
                };
            }
        }

        for (id, item) in self.artifacts.iter_id_val::<ItemId>() {
            let i_item = item.borrow();

            // Ignores artworks
            if i_item.artwork_scene.is_some() {
                continue;
            }
            
            let mut score = 1.;

            if let Some(owner) = i_item.owner {
                let owner = self.creatures.get(&owner);
                if owner.death.is_some() {
                    // Dead owner - Viable, but less preferable
                    score -= 1.;
                } else {
                    // Ignore alive owners
                    continue;
                }
            }

            // More special damage = More interesting
            let extra = i_item.extra_damage(&resources.materials);
            score += extra.average();
            
            // More events = More interesting
            score += *events_per_item.get(&id).unwrap_or(&0.) as f32;

            if let Some(quality) = &i_item.quality {
                score = score + quality.quality.main_stat_modifier() as f32;
            }

            match artifact {
                None => artifact = Some((id, item, score)),
                Some((_id, _item, c_score)) => {
                    if score > c_score {
                        artifact = Some((id, item, score));
                    }
                }
            }
        }
        if let Some((id, item, _score)) = artifact {
            // TODO(NJ5nTVIV): Title screen
            println!("You have heard of the legends of the ancient artifact {}. You set out into the world to find it's secrets.", item.borrow().name(&resources.materials));
            let codex = self.codex.artifact_mut(&id);
            codex.add_name();

            // TODO(NJ5nTVIV): What events to add?
            for (i, event) in self.events.iter().enumerate() {
                if event.relates_to_artifact(&id) {
                    codex.add_event(i);
                }
            }


        } else {
            println!("NO GOAL FOUND");
        }
    }

    pub(crate) fn dump_events(&self, filename: &str, resources: &Resources) {
        let mut f = File::create(filename).unwrap();
        println!("{:?} events", self.events.len());
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
            let mut world = World::new(WorldTopology::new(Size2D(10, 10)));

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

        pub(crate) fn creature_a1(&self) -> Ref<Creature> {
            return self.world.creatures.get(&self.creature_a1);
        }

        pub(crate) fn creature_a2(&self) -> Ref<Creature> {
            return self.world.creatures.get(&self.creature_a2);
        }

        pub(crate) fn creature_a3(&self) -> Ref<Creature> {
            return self.world.creatures.get(&self.creature_a3);
        }

        pub(crate) fn creature_a4(&self) -> Ref<Creature> {
            return self.world.creatures.get(&self.creature_a4);
        }

        pub(crate) fn creature_a3_mut(&mut self) -> RefMut<Creature> {
            return self.world.creatures.get_mut(&self.creature_a3);
        }

        pub(crate) fn creature_a4_mut(&mut self) -> RefMut<Creature> {
            return self.world.creatures.get_mut(&self.creature_a4);
        }

    }

}