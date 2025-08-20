use std::{fs::File, io::Write};

use serde::{Deserialize, Serialize};

use crate::{commons::rng::Rng, engine::geometry::Coord2, game::codex::Codex, history_trace, info, resources::resources::resources, warn, world::{creature::{CauseOfDeath, Goal, Profession}, history_generator::WorldGenerationParameters, item::{ItemId, Items}, plot::Plots, unit::{UnitId, UnitType}}, Event, Resources};

use super::{creature::{CreatureId, Creatures}, date::WorldDate, lineage::Lineages, topology::WorldTopology, unit::Units};

use crate::commons::id_vec::IdVec;

#[derive(Serialize, Deserialize)]
pub(crate) struct World {
    pub(crate) date: WorldDate,
    pub(crate) generation_parameters: WorldGenerationParameters,
    pub(crate) map: WorldTopology,
    pub(crate) units: Units,
    pub(crate) lineages: Lineages,
    pub(crate) creatures: Creatures,
    pub(crate) plots: Plots,
    pub(crate) events: Vec<Event>,
    pub(crate) artifacts: Items,
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

    pub(crate) fn add_event_to_codex(&mut self, event_id: usize) {
        let event = self.events.get(event_id).unwrap();
        for creature in event.related_creatures() {
            self.codex.creature_mut(&creature).add_event(event_id);
        }
        for artifact in event.related_artifacts() {
            self.codex.artifact_mut(&artifact).add_event(event_id);
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

    pub(crate) fn get_played_creature(&self) -> Option<&CreatureId> {
        return self.played_creature.as_ref();
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

            let lineage_1 = world.lineages.add(Lineage::new(culture, &resources.cultures.get(&culture)));
            let lineage_2 = world.lineages.add(Lineage::new(culture, &resources.cultures.get(&culture)));

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

// World operations ----------------------------------------------------------------------------

impl World {

    pub(crate) fn creature_kill_creature(&mut self, killed_id: CreatureId, killed_unit: UnitId, killer_id: CreatureId, killed_with: Option<ItemId>, death_unit: UnitId) {
        self.kill_creature(killed_id, killed_unit, death_unit, CauseOfDeath::KilledInBattle(killer_id, killed_with));
    }

    pub(crate) fn kill_creature(&mut self, creature_id: CreatureId, unit_from_id: UnitId, unit_death_id: UnitId, cause_of_death: CauseOfDeath) {
        let now = self.date.clone();
        let died_home = unit_from_id == unit_death_id;
        {
            let mut creature = self.creatures.get_mut(&creature_id);
            if creature.death.is_some() {
                warn!("Trying to kill already dead creature");
                return;
            }
            creature.death = Some((now.clone(), cause_of_death));
            if let Some(spouse_id) = creature.spouse {
                let mut spouse = self.creatures.get_mut(&spouse_id);
                spouse.spouse = None;
            }
            let mut unit = self.units.get_mut(&unit_from_id);
            let i = unit.creatures.iter().position(|id| *id == creature_id).unwrap();
            let id = unit.creatures.remove(i);

            if let Some(plot_id) = creature.supports_plot {
                let mut plot = self.plots.get_mut(&plot_id);
                plot.remove_supporter(creature_id, &mut creature);
            }

            // Else, the body is lost
            if died_home {
                unit.cemetery.push(id);
            } else {
                let mut death_unit = self.units.get_mut(&unit_death_id);
                if let Some(settlement) = &mut death_unit.settlement {
                    let resources = resources();
                    let species = resources.species.get(&creature.species);
                    for drop in species.drops.iter() {
                        settlement.add_material(drop, 1);
                    }
                }
            }    

            drop(unit);

            let mut inheritor = None;
            let mut has_possession = false;

            if let Some(details) = &creature.details {
                if details.inventory.len() > 0 {
                    has_possession = true;
                    if died_home {
                        for candidate_id in creature.offspring.iter() {
                            let candidate = self.creatures.get(candidate_id);
                            if candidate.death.is_none() {
                                inheritor = Some(*candidate_id);
                                break;
                            }
                        }
                    }
                }
            }

            // TODO(IhlgIYVA): Extract
            if let CauseOfDeath::KilledInBattle(killer_id, _) = &cause_of_death {
                for relationship in creature.relationships.iter() {
                    let relationship_creature_id = relationship.creature_id;
                    let mut relationship_creature = self.creatures.get_mut(&relationship_creature_id);
                    let relationship = relationship_creature.relationship_find(creature_id);
                    if let Some(relationship) = relationship {
                        if relationship.friend_or_better() {
                            // TODO(IhlgIYVA): How did I get to a point where it killed his own "friend"?
                            if relationship_creature_id == *killer_id {
                                warn!("Killed its own friend. rel: {:?} killer: {:?}", relationship_creature_id, killer_id);
                                continue
                            }
                            let killer = self.creatures.get(killer_id);
                            let killer_relationship = relationship_creature.relationship_find_mut_or_insert(&relationship_creature_id, *killer_id, &killer);
                            killer_relationship.add_opinion(-75);

                            if killer_relationship.mortal_enemy_or_worse() {
                                // TODO(IhlgIYVA): Determinate
                                // TODO(IhlgIYVA): Magic number
                                if Rng::rand().rand_chance(0.8) {
                                    let goal = Goal::KillBeast(*killer_id);
                                    history_trace!("creature_add_goal creature_id:{:?} goal:{:?}", relationship_creature_id, goal);
                                    relationship_creature.goals.push(goal);
                                }
                            }

                        }
                    }
                }
            }

            // Purges unnecessary data after death
            creature.relationships.clear();

            drop(creature);

            if has_possession {
                if let Some(inheritor_id) = inheritor {
                    self.transfer_inventory(creature_id, inheritor_id);
                } else {
                    if died_home {
                        let creature = self.creatures.get(&creature_id);
                        let inventory = creature.details.as_ref().map(|d| d.inventory.clone());
                        drop(creature);
                        if let Some(inventory) = inventory {
                            self.record_event(Event::BurriedWithPosessions { date: now.clone(), creature_id, items_ids: inventory });
                        }
                    } else {
                        self.drop_inventory(creature_id);
                    }
                }
            }

            // TODO: Inherit leadership

        }
        self.record_event(Event::CreatureDeath { date: now.clone(), creature_id: creature_id, cause_of_death: cause_of_death });
    }


    fn transfer_inventory(&mut self, current_id: CreatureId, new_owner_id: CreatureId) {
        let mut current = self.creatures.get_mut(&current_id);
        let mut inventory: Vec<ItemId> = current.details().inventory.drain(..).collect();
        for item_id in inventory.iter() {
            let mut item = self.artifacts.get_mut(item_id);
            item.owner = Some(new_owner_id);
        }
        drop(current);

        history_trace!("transfer_inventory {:?}", current_id, new_owner_id);

        for item in inventory.iter() {
            self.record_event(Event::InheritedArtifact { date: self.date.clone(), creature_id: new_owner_id, from: current_id, item: *item });
        }
        let mut new_owner = self.creatures.get_mut(&new_owner_id);
        new_owner.details().inventory.append(&mut inventory);
    }

    fn drop_inventory(&mut self, creature_id: CreatureId) {
        history_trace!("drop_inventory {:?}", creature_id);

        let mut current = self.creatures.get_mut(&creature_id);
        let inventory: Vec<ItemId> = current.details().inventory.drain(..).collect();
        for item_id in inventory.iter() {
            let mut item = self.artifacts.get_mut(item_id);
            item.owner = None;
        }

        // TODO(NJ5nTVIV): Add to death unit
    }

    // Events

    fn record_event(&mut self, event: Event) {
        let i = self.events.len();
        let mut relates_to_player = false;
        if let Some(played_creature) = self.get_played_creature() {
            relates_to_player = event.relates_to_creature(played_creature);
        }
        self.events.push(event);
        if relates_to_player {
            self.add_event_to_codex(i);
        }
    }

}