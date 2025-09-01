use std::{fs::File, io::Write};

use common::error::Error;
use math::{rng::Rng, Vec2i};
use serde::{Deserialize, Serialize};

use crate::{commons::rng::Rng as OldRng, engine::geometry::Coord2, game::codex::Codex, history_trace, info, resources::resources::resources, warn, world::{creature::{CauseOfDeath, Creature, CreatureGender, Goal, Profession}, history_generator::WorldGenerationParameters, item::{ItemId, Items}, plot::Plots, site::{Site, SiteId, SiteResources, SiteType, Structure, StructureType}}, Event, Resources};

use super::{creature::{CreatureId, Creatures}, date::WorldDate, lineage::Lineages, topology::WorldTopology, site::Sites};

use crate::commons::id_vec::IdVec;

#[derive(Serialize, Deserialize)]
pub(crate) struct World {
    pub(crate) date: WorldDate,
    pub(crate) generation_parameters: WorldGenerationParameters,
    pub(crate) map: WorldTopology,
    pub(crate) sites: Sites,
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
            sites: Sites::new(),
            creatures: Creatures::new(),
            lineages: Lineages::new(),
            plots: Plots::new(),
            artifacts: IdVec::new(),
            events: Vec::new(),
            codex: Codex::new(),
            played_creature: None,
        }
    }

    pub(crate) fn rng(&self) -> Rng {
        self.generation_parameters.rng()
    }

    pub(crate) fn create_scenario(&mut self) -> Result<(CreatureId, Coord2), ()> {
        let mut candidate = None;
        'outer: for site_id in self.sites.iter_ids::<SiteId>() {
            let site = self.sites.get(&site_id);
            if site.site_type == SiteType::Village {
                for creature_id in site.creatures.iter() {
                    let creature = self.creatures.get(creature_id);
                    let age = (self.date - creature.birth).get_years();
                    if age > 20 && age < 40 && creature.spouse.is_none() && creature.profession == Profession::Peasant {
                        candidate = Some((creature_id.clone(), site.xy.into()));
                        break 'outer;
                    }
                }
            }
        }
        if let Some(candidate) = candidate {
            self.played_creature = Some(candidate.0);
            self.codex = Codex::new();

            // Major sites
            for site_id in self.sites.iter_ids::<SiteId>() {
                let site = self.sites.get(&site_id);
                if site.creatures.len() > 0 && site.site_type == SiteType::Village {
                    self.codex.site_mut(&site_id);
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

    pub(crate) fn get_site_at(&self, coord: &Coord2) -> Option<SiteId> {
        for site_id in self.sites.iter_ids::<SiteId>() {
            let site = self.sites.get(&site_id);
            if site.xy.eq(&coord.to_vec2i()) {
                return Some(site_id)
            }
        }
        return None
    }

}

#[cfg(test)]
pub(crate) mod fixture {
    use std::cell::{Ref, RefMut};

    use crate::{engine::geometry::Size2D, world::{creature::{Creature, CreatureGender, Profession, SIM_FLAG_INTELIGENT}, lineage::Lineage, site::{Site, SiteId, SiteResources}}};

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

            let _: SiteId = world.sites.add(Site {
                artifacts: Vec::new(),
                cemetery: Vec::new(),
                name: None,
                creatures: vec!(creature_a1, creature_a2, creature_a3, creature_a4),
                population_peak: (2, 1),
                resources: SiteResources { food: 0. },
                settlement: None,
                site_type: crate::world::site::SiteType::Village,
                xy: Vec2i(1, 1),
                structures: Vec::new()
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

    pub(crate) fn creature_couple_have_child(&mut self, mother_id: CreatureId, site_id: &SiteId, rng: &mut OldRng) -> Result<(), Error> {
        let mother = self.creatures.get_mut(&mother_id);

        let father_id = mother.spouse.ok_or("Woman with no spouse trying to have a child")?;
        let father = self.creatures.get(&father_id);
        let lineage = father.lineage.clone();
        let gender = CreatureGender::random_det(rng);
        let child = Creature {
            birth: self.date.clone(),
            death: None,
            profession: Profession::None,
            lineage,
            mother: mother_id,
            father: father_id,
            gender,
            offspring: Vec::new(),
            species: mother.species,
            spouse: None,
            details: None,
            experience: 0,
            sim_flags: father.sim_flags,
            relationships: vec!(),
            goals: vec!(),
            supports_plot: None,
        };

        drop(mother);
        drop(father);
        let child_id = self.creatures.add(child);
        let mut site = self.sites.get_mut(site_id);
        site.creatures.push(child_id);

        let structure = site.structure_occupied_by_mut(&mother_id).ok_or("Mother had child without a house")?;
        structure.add_ocuppant(child_id);

        drop(site);
        self.record_event(Event::CreatureBirth { date: self.date.clone(), creature_id: child_id });
        {
            let mut father = self.creatures.get_mut(&father_id);
            father.offspring.push(child_id);
        }
        {
            let mut mother = self.creatures.get_mut(&mother_id);
            mother.offspring.push(child_id);
        }
        Ok(())
    }

    pub(crate) fn creature_start_new_home_same_site(&mut self, creature_id: CreatureId, site_id: &SiteId) -> Result<(), Error> {
        let mut site = self.sites.get_mut(site_id);
        let creature = self.creatures.get(site_id);
        let current_home = site.structure_occupied_by_mut(&creature_id).ok_or("Homeless creature trying to start new home")?;

        // Spouse and children
        let family = current_home.occupants_drain(|id| {
            if id == &creature_id {
                return true;
            }
            if let Some(spouse) = creature.spouse {
                if id == &spouse {
                    return true;
                }
            }
            return creature.offspring.contains(id);
        }, self.date);

        // Moves into existing house
        for structure in site.structures.iter_mut() {
            if structure.get_type() == &StructureType::House && structure.get_status().is_abandoned() {
                for id in family {
                    structure.add_ocuppant(id);
                }
                return Ok(());
            }
        }

        // Start new house
        let mut structure = Structure::new(StructureType::House);
        for creature_id in family {
            structure.add_ocuppant(creature_id);
        }
        site.structures.push(structure);
        return Ok(())
    }

    pub(crate) fn creature_leave_for_bandit_camp(&mut self, creature_id: CreatureId, site_id: SiteId, rng: &mut Rng) -> Result<(), Error> {
        let site = self.sites.get(&site_id);
        let site_xy = site.xy.clone();
        drop(site);
        // Looks for a camp nearby
        let existing_camp = self.sites.iter_id_val().find(|(_site_id, site)| {
            let site = site.borrow();
            site.site_type == SiteType::BanditCamp
                && site.xy.dist_squared(&site_xy) < 15.*15.
                && site.creatures.len() > 0
        });
        // If there's a camp nearby
        if let Some((camp_id, existing_camp)) = existing_camp {
            let mut existing_camp = existing_camp.borrow_mut();
            existing_camp.creatures.push(creature_id);
            // Bandit camps always have only 1 structure (see below)
            existing_camp.structures.get_mut(0).unwrap().add_ocuppant(creature_id);
            self.events.push(Event::JoinBanditCamp { date: self.date, creature_id: creature_id, site_id: site_id, new_site_id: camp_id });
        } else {
            // Creates new camp
            let pos = self.site_search_new_pos_closeby(site_xy, 15, rng).ok_or("No position found for new bandit camp")?;
            let mut site = Site {
                xy: pos,
                artifacts: Vec::new(),
                cemetery: Vec::new(),
                creatures: vec!(creature_id),
                settlement: None,
                name: None,
                population_peak: (0, 0),
                site_type: SiteType::BanditCamp,
                resources: SiteResources {
                    food: 1.
                },
                structures: vec!(Structure::new(StructureType::BanditCamp))
            };
            site.structures.get_mut(0).unwrap().add_ocuppant(creature_id);
            let new_camp_id = self.sites.add(site);
            self.events.push(Event::CreateBanditCamp { date: self.date, creature_id: creature_id, site_id: site_id, new_site_id: new_camp_id });
        }
        // Removes creature from site
        let mut site = self.sites.get_mut(&site_id);
        site.remove_creature(&creature_id, self.date);
        site.resources.food -= 1.;
        // Chances profession
        let mut creature = self.creatures.get_mut(&creature_id);
        creature.profession = Profession::Bandit;
        Ok(())
    }

    pub(crate) fn creature_kill_creature(&mut self, killed_id: CreatureId, killed_site: SiteId, killer_id: CreatureId, killed_with: Option<ItemId>, death_site: SiteId) {
        self.kill_creature(killed_id, killed_site, death_site, CauseOfDeath::KilledInBattle(killer_id, killed_with));
    }

    pub(crate) fn kill_creature(&mut self, creature_id: CreatureId, site_from_id: SiteId, site_death_id: SiteId, cause_of_death: CauseOfDeath) {
        let now = self.date.clone();
        let died_home = site_from_id == site_death_id;
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
            let mut site = self.sites.get_mut(&site_from_id);
            site.remove_creature(&creature_id, self.date);

            if let Some(plot_id) = creature.supports_plot {
                let mut plot = self.plots.get_mut(&plot_id);
                plot.remove_supporter(creature_id, &mut creature);
            }

            // Else, the body is lost
            if died_home {
                site.cemetery.push(creature_id);
            } else {
                let mut death_site = self.sites.get_mut(&site_death_id);
                if let Some(settlement) = &mut death_site.settlement {
                    let resources = resources();
                    let species = resources.species.get(&creature.species);
                    for drop in species.drops.iter() {
                        settlement.add_material(drop, 1);
                    }
                }
            }    

            drop(site);

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
                                if OldRng::rand().rand_chance(0.8) {
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

        history_trace!("transfer_inventory {:?} {:?}", current_id, new_owner_id);

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

        // TODO(NJ5nTVIV): Add to death site
    }

    // Sites

    pub(crate) fn site_change_leader(&mut self, site_id: &SiteId, new_leader: CreatureId) -> Result<(), &'static str> {
        {
            let mut leader = self.creatures.get_mut(&new_leader);
            leader.profession = Profession::Ruler;
        }
        {
            let mut site = self.sites.get_mut(site_id);
            site.settlement.as_mut().ok_or("No election should be held with no settlement")?.leader = Some(new_leader);

            // Swaps the house to the townhall
            let move_into_townhall = site.structure_occupied_by(&new_leader).ok_or("Homeless leader")?.get_type() == &StructureType::House;
            if move_into_townhall {
                let townhall = site.structures.iter_mut().find(|s| s.get_type() == &StructureType::TownHall).ok_or("Village with no townhall")?;
                let townhall_occupants = townhall.occupants_take(self.date);

                let house = site.structure_occupied_by_mut(&new_leader).ok_or("Homeless leader")?;
                let house_occupants = house.occupants_take(self.date);
                for id in townhall_occupants {
                    house.add_ocuppant(id);
                }

                let townhall = site.structures.iter_mut().find(|s| s.get_type() == &StructureType::TownHall).ok_or("Village with no townhall")?;
                for id in house_occupants {
                    townhall.add_ocuppant(id);
                }
            }
        }
        self.record_event(Event::NewLeaderElected { date: self.date.clone(), site_id: *site_id, creature_id: new_leader });
        return Ok(())
    }

    fn site_search_new_pos_closeby(&self, center: Vec2i, max_radius: i32, rng: &mut Rng) -> Option<Vec2i> {
        let x_limit = [
            (center.x() - max_radius).max(3),
            (center.x() + max_radius).min(self.map.size.x() as i32 - 3)
        ];
        let y_limit = [
            (center.y() - max_radius).max(3),
            (center.y() + max_radius).min(self.map.size.y() as i32 - 3)
        ];
        for _ in 0..100 {
            let x = rng.usize_range(x_limit[0] as usize, x_limit[1] as usize);
            let y = rng.usize_range(y_limit[0] as usize, y_limit[1] as usize);
            let candidate = Vec2i(x as i32, y as i32);
            let too_close = self.sites.iter().any(|site| {
                let site = site.borrow();
                if site.creatures.len() == 0 {
                    if site.xy == candidate.into() {
                        return true;
                    }
                    return false;
                }
                return site.xy.dist_squared(&candidate.into()) < 3. * 3.
            });
            if too_close {
                continue;
            }
            return Some(candidate)
        }
        return None;
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