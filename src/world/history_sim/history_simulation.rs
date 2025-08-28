use common::error::Error;

use crate::{commons::{rng::Rng, xp_table::xp_to_level}, engine::geometry::Coord2, game::factory::item_factory::ItemFactory, history_trace, resources::resources::resources, warn, world::{creature::{CreatureId, Profession, SIM_FLAG_GREAT_BEAST}, date::{Duration, WorldDate}, history_generator::WorldGenerationParameters, history_sim::{creature_simulation::{add_item_to_inventory, attack_nearby_site, execute_plot, find_supporters_for_plot, start_plot}, storyteller::Storyteller, world_ops}, item::ItemQuality, site::{Site, SiteId, SiteResources, SiteType}, world::World}, Event};

use super::{creature_simulation::{CreatureSideEffect, CreatureSimulation}, factories::{ArtifactFactory, CreatureFactory}};

pub(crate) struct HistorySimulation {
    pub(crate) rng: Rng,
    storyteller: Storyteller,
}

impl HistorySimulation {
    pub(crate) fn new(rng: Rng, generation_params: WorldGenerationParameters) -> Self {
        HistorySimulation {
            rng,
            storyteller: Storyteller::new(generation_params)
        }
    }

    pub(crate) fn seed(&mut self, world: &mut World) {
        for _ in 0..world.generation_parameters.number_of_seed_cities {
            let _err = world_ops::spawn_random_village(world, &mut self.rng, &resources(), world.generation_parameters.seed_cities_population as u32);
        }
    }

    pub(crate) fn simulate_step(&mut self, step: Duration, world: &mut World) -> bool {
        let resources = resources();
        world.date = world.date + step;

        let chances = self.storyteller.global_chances(&mut self.rng, &world, &step);

        if self.rng.rand_chance(chances.spawn_varningr) {
            let pos = self.find_site_suitable_pos(&mut self.rng.clone(), world);

            if let Some(pos) = pos {
                let species = resources.species.id_of("species:varningr");
                let mut factory = CreatureFactory::new(self.rng.derive("creature"));
                let creature = factory.make_single(species, 3, SIM_FLAG_GREAT_BEAST, world);
                let site = Site {
                    artifacts: Vec::new(),
                    cemetery: Vec::new(),
                    name: None,
                    creatures: vec!(creature),
                    settlement: None,
                    population_peak: (0, 0),
                    resources: SiteResources { food: 2. },
                    site_type: SiteType::VarningrLair,
                    xy: pos.to_vec2i(),
                    structures: Vec::new()
                };
                world.sites.add::<SiteId>(site);
            }
        }
        if self.rng.rand_chance(chances.spawn_wolf_pack) {
            let pos = self.find_site_suitable_pos(&mut self.rng.clone(), world);

            if let Some(pos) = pos {
                let species = resources.species.id_of("species:wolf");
                let mut factory = CreatureFactory::new(self.rng.derive("creature"));
                let creatures = vec!(
                    factory.make_single(species, 1, 0, world),
                    factory.make_single(species, 1, 0, world),
                    factory.make_single(species, 1, 0, world),
                );
                let site = Site {
                    creatures,
                    artifacts: Vec::new(),
                    cemetery: Vec::new(),
                    settlement: None,
                    name: None,
                    population_peak: (0, 0),
                    resources: SiteResources { food: 2. },
                    site_type: SiteType::WolfPack,
                    xy: pos.to_vec2i(),
                    structures: Vec::new()
                };
                world.sites.add::<SiteId>(site);
            }
        }
        if self.rng.rand_chance(chances.spawn_village) {
            let _err = world_ops::spawn_random_village(world, &mut self.rng, &resources, world.generation_parameters.st_village_population as u32);
        }

        // Check plot completion
        for plot in world.plots.iter() {
            plot.borrow_mut().verify_success(world);
        }

        let mut creatures = 0;

        for id in world.sites.iter_ids::<SiteId>() {

            let site = world.sites.get(&id);
            creatures += site.creatures.len();
            drop(site);

            let result = self.simulate_step_site(world, &step, &world.date.clone(), self.rng.clone(), &id);
            if let Err(msg) = result {
                warn!("{msg}");
            }

            self.rng.next();
        }
        return creatures > 0;
    }

    fn simulate_step_site(&mut self, world: &mut World, step: &Duration, now: &WorldDate, mut rng: Rng, site_id: &SiteId) -> Result<(), Error> {

        let chances = self.storyteller.story_teller_site_chances(site_id, &world, &step);

        let site = world.sites.get_mut(site_id);

        let mut resources = site.resources.clone();

        let site_tile = world.map.tile(site.xy.x() as usize, site.xy.y() as usize);
        
        let mut marriage_pool = Vec::new();
        let mut change_job_pool = Vec::new();
        let creatures = site.creatures.clone();
        drop(site);

        for creature_id in creatures.iter() {
            let site = world.sites.get_mut(site_id);
            let mut creature = world.creatures.get_mut(creature_id);

            // SMELL: Doing things outside of the method because of borrow issues
            let plot = creature.supports_plot.and_then(|plot_id| Some(world.plots.get(&plot_id)));
            creature.goals.retain(|goal| {
                !goal.check_completed(world)
            });

            let side_effect = CreatureSimulation::simulate_step_creature(step, now, &mut rng, &site, &creature_id, &creature, plot, &chances);

            // Production and consumption
            let mut production = creature.profession.base_resource_production();
            production.food = production.food * site_tile.soil_fertility;
            resources = production + resources;
            resources.food -= 1.0;
            
            drop(creature);
            drop(site);

            history_trace!("creature_action creature_id:{:?} action:{:?}", creature_id, side_effect);

            let result  = match side_effect {
                CreatureSideEffect::None => Ok(()),
                CreatureSideEffect::Death(cause_of_death) => {
                    world.kill_creature(*creature_id, *site_id, *site_id, cause_of_death);
                    Ok(())
                },
                CreatureSideEffect::HaveChild => world.creature_couple_have_child(*creature_id, site_id, &mut rng),
                CreatureSideEffect::MoveOutToNewHouse => world.creature_start_new_home_same_site(*creature_id, site_id),
                CreatureSideEffect::LookForMarriage => {
                    let creature = world.creatures.get_mut(creature_id);
                    marriage_pool.push((*creature_id, creature.gender));
                    Ok(())
                },
                CreatureSideEffect::LookForNewJob => {
                    change_job_pool.push(*creature_id);
                    Ok(())
                },
                CreatureSideEffect::MakeArtifact => {
                    Self::make_artifact(&creature_id, None, site_id, world, &mut rng);
                    Ok(())
                },
                CreatureSideEffect::BecomeBandit => world.creature_leave_for_bandit_camp(*creature_id, *site_id, &mut rng.to_new()),
                CreatureSideEffect::AttackNearbySites => {
                    attack_nearby_site(world, &mut rng, *site_id);
                    Ok(())
                }
                CreatureSideEffect::StartPlot(goal) => {
                    start_plot(world, *creature_id, goal);
                    Ok(())
                }
                CreatureSideEffect::FindSupportersForPlot => {
                    find_supporters_for_plot(world, *creature_id);
                    Ok(())
                }
                CreatureSideEffect::ExecutePlot => {
                    execute_plot(world, *site_id, *creature_id, &mut rng);
                    Ok(())
                }
            };

            if let Err(str) = result {
                return Err(Error::new(format!("{str} {:?}", creature_id)));
            }

        }

        {
            let mut site = world.sites.get_mut(site_id);
            site.resources = resources;
            self.check_population_peak(now, &mut site);
        }

        {
            let site = world.sites.get(site_id);

            let need_election = match &site.settlement {
                None => false,
                Some(settlement) => {
                    match settlement.leader {
                        None => true,
                        Some(creature_id) => {
                            let creature = world.creatures.get(&creature_id);
                            creature.death.is_some()
                        }
                    }
                }
            } && site.creatures.len() > 0;
            drop(site);
            if need_election {
                let site = world.sites.get(site_id);
                let mut candidates_pool = Vec::new();
                for creature_id in site.creatures.iter() {
                    let creature = world.creatures.get(creature_id);
                    let age = (*now - creature.birth).year();
                    if age > 18 {
                        candidates_pool.push(creature_id);
                    }
                }
                let new_leader = match candidates_pool.len() {
                    0 => site.creatures[rng.randu_range(0, site.creatures.len())],
                    _ => *candidates_pool[rng.randu_range(0, candidates_pool.len())],
                };     
                drop(site);
                world.site_change_leader(site_id, new_leader)?;
            }
            
        }
        
        while marriage_pool.len() > 0 {
            let candidate_a = marriage_pool.pop().unwrap();
            let candidate_b = marriage_pool.iter().position(|x| x.1 != candidate_a.1);
            match candidate_b {
                Some(candidate_b) => {
                    let candidate_b = marriage_pool.remove(candidate_b);
                    // TODO: Can marry brother/sister
                    // TODO: Large age diff: 28, [CreatureId(203) 26] and [CreatureId(46) 64] married
                    {
                        let mut creature_a = world.creatures.get_mut(&candidate_a.0);
                        let mut creature_b = world.creatures.get_mut(&candidate_b.0);
                        creature_a.spouse = Some(candidate_b.0);
                        creature_b.spouse = Some(candidate_a.0);
                    }
                    world.events.push(Event::CreatureMarriage { date: now.clone(), creature_id: candidate_a.0, spouse_id: candidate_b.0 });
                },
                None => ()
            }
        }

        for creature_id in change_job_pool {
            let mut creature = world.creatures.get_mut(&creature_id);
            let site = world.sites.get(site_id);
            let profession = site.select_new_profession(&mut rng);
            creature.profession = profession;
            drop(creature);
            drop(site);
            world.events.push(Event::CreatureProfessionChange { date: now.clone(), creature_id: creature_id, new_profession: profession });
        }
        Ok(())
    }

    fn make_artifact(artisan_id: &CreatureId, comissioneer_id: Option<&CreatureId>, site_id: &SiteId, world: &mut World, rng: &mut Rng) {
        let mut artisan = world.creatures.get_mut(artisan_id);
        let mut site = world.sites.get_mut(site_id);
        let item = match artisan.profession {
            Profession::Blacksmith => {
                let f_quality = rng.randf() + (xp_to_level(artisan.experience) as f32 * 0.01);
                let quality;
                if f_quality <= 0.5 {
                    quality = ItemQuality::Poor
                } else if f_quality <= 0.80 {
                    quality = ItemQuality::Normal
                } else if f_quality <= 0.95 {
                    quality = ItemQuality::Good
                } else if f_quality <= 1.00 {
                    quality = ItemQuality::Excelent
                } else {
                    quality = ItemQuality::Legendary
                }

                let item = ItemFactory::weapon(rng, &resources())
                    .quality(quality)
                    .material_pool(site.settlement.as_mut().and_then(|sett| Some(&mut sett.material_stock)))
                    .named()
                    .make();
                Some(item)
            },
            Profession::Sculptor => {
                if let Some(comissioneer_id) = comissioneer_id {
                    Some(ArtifactFactory::create_statue(rng, &resources(), *comissioneer_id, &world))
                } else {
                    None
                }
            },
            _ => None
        };

        if item.is_some() {
            // Levels-up artisan
            artisan.experience += 100;
        }

        drop(artisan);

        if let Some(item) = item {
            
            let who_gets_item = comissioneer_id.unwrap_or(artisan_id);
            let id = world.artifacts.add(item.clone());
            {
                let mut creature = world.creatures.get_mut(who_gets_item);
                match &item.artwork_scene {
                    Some(_) => {
                        site.artifacts.push(id);
                    },
                    None => {
                        let mut item = world.artifacts.get_mut(&id);
                        add_item_to_inventory(id, &mut item, *who_gets_item, &mut creature);
                    },
                }
            }
            if let Some(comissioneer_id) = comissioneer_id {
                world.events.push(Event::ArtifactComission { date: world.date.clone(), creature_id: *comissioneer_id, creator_id: *artisan_id, item_id: id });
            } else {
                world.events.push(Event::ArtifactCreated { date: world.date.clone(), artifact: id, creator: *artisan_id, site_id: *site_id });
            }
        }
    }

    fn check_population_peak(&self, now: &WorldDate, site: &mut Site) {
        if site.creatures.len() >= site.population_peak.1 as usize {
            site.population_peak = (now.year(), site.creatures.len() as u32)
        }
    }

    fn find_site_suitable_pos(&self, rng: &mut Rng, world: &World) -> Option<Coord2> {
        for _ in 0..100 {
            let x = rng.randu_range(3, world.map.size.x() - 3);
            let y = rng.randu_range(3, world.map.size.y() - 3);
            let candidate = Coord2::xy(x as i32, y as i32);
            let too_close = world.sites.iter().any(|site| {
                let site = site.borrow();
                let site_xy: Coord2 = site.xy.into();
                if site.creatures.len() == 0 {
                    if site_xy == candidate {
                        return true;
                    }
                    return false;
                }
                return site_xy.dist_squared(&candidate) < 3. * 3.
            });
            if too_close {
                continue;
            }
            return Some(candidate)
        }
        return None;
    }

}