use std::time::Instant;

use crate::{commons::{rng::Rng, xp_table::xp_to_level}, engine::geometry::Coord2, game::factory::item_factory::ItemFactory, history_trace, resources::resources::Resources, warn, world::{creature::{CreatureId, Profession, SIM_FLAG_GREAT_BEAST}, date::WorldDate, history_generator::WorldGenerationParameters, history_sim::{creature_simulation::{add_item_to_inventory, attack_nearby_unit, execute_plot, find_supporters_for_plot, kill_creature, start_plot}, storyteller::Storyteller}, item::ItemQuality, unit::{SettlementComponent, Unit, UnitId, UnitResources, UnitType}, world::World}, Event};

use super::{creature_simulation::{CreatureSideEffect, CreatureSimulation}, factories::{ArtifactFactory, CreatureFactory}};

pub(crate) struct HistorySimulation {
    pub(crate) date: WorldDate,
    generation_params: WorldGenerationParameters,
    pub(crate) rng: Rng,
    pub(crate) resources: Resources,
}

impl HistorySimulation {
    pub(crate) fn new(rng: Rng, resources: Resources, generation_params: WorldGenerationParameters) -> Self {
        HistorySimulation {
            date: WorldDate::new(0, 0, 0),
            generation_params,
            rng,
            resources
        }
    }

    pub(crate) fn seed(&mut self, world: &mut World) {

        let mut factory = CreatureFactory::new(self.rng.derive("creature"));

        for _ in 0..self.generation_params.number_of_seed_cities {

            let pos = self.find_unit_suitable_pos(&mut self.rng.clone(), &world);
            let pos = match pos {
                None => break,
                Some(candidate) => candidate,
            };

            let mut unit = Unit {
                xy: pos,
                creatures: Vec::new(),
                cemetery: Vec::new(),
                resources: UnitResources {
                    // Enough food for a year
                    food: self.generation_params.seed_cities_population as f32
                },
                settlement: Some(SettlementComponent {
                    leader: None,
                    material_stock: Vec::new()
                }),
                artifacts: Vec::new(),
                population_peak: (0, 0),
                unit_type: UnitType::Village
            };

            while unit.creatures.len() < self.generation_params.seed_cities_population as usize {
                
                let family = factory.make_family_or_single(&self.date, self.resources.species.id_of("species:human"), world, &self.resources);
                for creature_id in family {
                    unit.creatures.push(creature_id);
                }

            }

            self.rng.next();

            world.units.add::<UnitId>(unit);
        }
    }

    pub(crate) fn simulate_step(&mut self, step: WorldDate, world: &mut World) -> bool {
        self.date = self.date + step;
        world.date = self.date.clone();
        let now = Instant::now();

        // TODO(tfWpiQPF): Find a cooler way to spawn
        if self.rng.rand_chance(0.2) {
            let pos = self.find_unit_suitable_pos(&mut self.rng.clone(), world);

            if let Some(pos) = pos {
                let species = self.resources.species.id_of("species:varningr");
                let mut factory = CreatureFactory::new(self.rng.derive("creature"));
                let creature = factory.make_single(species, 10, SIM_FLAG_GREAT_BEAST, world);
                let unit = Unit {
                    artifacts: Vec::new(),
                    cemetery: Vec::new(),
                    creatures: vec!(creature),
                    settlement: None,
                    population_peak: (0, 0),
                    resources: UnitResources { food: 0. },
                    unit_type: UnitType::VarningrLair,
                    xy: pos
                };

                println!("[!!!] spawn varningr");

                world.units.add::<UnitId>(unit);

            } else {
                println!("[!!!] failed to spawn");
            }


        }

        // Check plot completion
        for plot in world.plots.iter() {
            plot.borrow_mut().verify_success(world);
        }

        let mut creatures = 0;

        for id in world.units.iter_ids::<UnitId>() {

            let unit = world.units.get(&id);
            creatures += unit.creatures.len();
            drop(unit);

            self.simulate_step_unit(world, &step, &self.date.clone(), self.rng.clone(), &id);
            self.rng.next();
        }


        println!("");
        println!("Elapsed: {:.2?}", now.elapsed());
        println!("Year: {}", self.date.year());
        println!("Total units: {}", world.units.len());
        println!("Total creatures: {}", world.creatures.len());
        println!("Simulated creatures: {}", creatures);
        println!("Total artifacts: {}", world.artifacts.len());
        println!("Total events: {}", world.events.len());

        return creatures > 0;
    }

    fn simulate_step_unit(&mut self, world: &mut World, step: &WorldDate, now: &WorldDate, mut rng: Rng, unit_id: &UnitId) {
        let mut unit = world.units.get_mut(unit_id);
        let mut side_effects = Vec::new();

        let mut resources = unit.resources.clone();

        let unit_tile = world.map.tile(unit.xy.x as usize, unit.xy.y as usize);

        let chances = Storyteller::new().story_teller_unit_chances(&self.generation_params, &unit);

        for creature_id in unit.creatures.iter() {
            let mut creature = world.creatures.get_mut(creature_id);

            // SMELL: Doing things outside of the method because of borrow issues
            let plot = creature.supports_plot.and_then(|plot_id| Some(world.plots.get(&plot_id)));
            creature.goals.retain(|goal| {
                !goal.check_completed(world)
            });

            let side_effect = CreatureSimulation::simulate_step_creature(step, now, &mut rng, &unit, &creature, plot, &chances);
            side_effects.push((*creature_id, side_effect));

            // Production and consumption
            let mut production = creature.profession.base_resource_production();
            production.food = production.food * unit_tile.soil_fertility;
            resources = production + resources;
            resources.food -= 1.0;
            
        }

        unit.resources = resources;

        drop(unit);

        let mut marriage_pool = Vec::new();
        let mut change_job_pool = Vec::new();
        // TODO: Move all of these to a impl
        for (creature_id, side_effect) in side_effects.into_iter() {

            // SMELL: Creature could've died from another creature action, but it already decided it's own action.
            //        This happens because I choose the action for ALL creatures, and THEN execute them.
            let creature = world.creatures.get(&creature_id);
            if creature.death.is_some() {
                warn!("Simulated creature is dead");
                continue;
            }
            drop(creature);

            history_trace!("creature_action creature_id:{:?} action:{:?}", creature_id, side_effect);

            match side_effect {
                CreatureSideEffect::None => (),
                CreatureSideEffect::Death(cause_of_death) => kill_creature(world, creature_id, *unit_id, *unit_id, cause_of_death, &mut self.resources),
                CreatureSideEffect::HaveChild => {
                    let mut creature = world.creatures.get_mut(&creature_id);
                    let child = CreatureSimulation::have_child_with_spouse(now, &world, &mut rng, &creature_id, &mut creature);
                    drop(creature);
                    if let Some(child) = child {
                        let father = child.father;
                        let mother = child.mother;
                        let creature_id = world.creatures.add(child);
                        let mut unit = world.units.get_mut(unit_id);
                        unit.creatures.push(creature_id);
                        world.events.push(Event::CreatureBirth { date: now.clone(), creature_id });
                        {
                            let mut father = world.creatures.get_mut(&father);
                            father.offspring.push(creature_id);
                        }
                        {
                            let mut mother = world.creatures.get_mut(&mother);
                            mother.offspring.push(creature_id);
                        }
                    }
                },
                CreatureSideEffect::LookForMarriage => {
                    let creature = world.creatures.get_mut(&creature_id);
                    marriage_pool.push((creature_id, creature.gender));
                },
                CreatureSideEffect::LookForNewJob => {
                    change_job_pool.push(creature_id);
                },
                CreatureSideEffect::MakeArtifact => Self::make_artifact(&creature_id, None, unit_id, world, &mut rng, &mut self.resources),
                CreatureSideEffect::BecomeBandit => {
                    // Removes creature from unit
                    let unit = world.units.get(unit_id);
                    let unit_xy = unit.xy.clone();
                    drop(unit);
                    // Looks for a camp nearby
                    let existing_camp = world.units.iter_id_val().find(|(_unit_id, unit)| {
                        let unit = unit.borrow();
                        unit.unit_type == UnitType::BanditCamp
                          && unit.xy.dist_squared(&unit_xy) < 15.*15.
                          && unit.creatures.len() > 0
                    });
                    // If there's a camp nearby
                    if let Some((camp_id, existing_camp)) = existing_camp {
                        let mut existing_camp = existing_camp.borrow_mut();
                        existing_camp.creatures.push(creature_id);
                        world.events.push(Event::JoinBanditCamp { date: *now, creature_id, unit_id: *unit_id, new_unit_id: camp_id });
                    } else {
                        // Creates new camp
                        let pos = self.find_unit_suitable_position_closeby(unit_xy, 15, &mut rng, world);
                        match pos {
                            Some(pos) => {
                                let new_camp_id = world.units.add(Unit {
                                    xy: pos,
                                    artifacts: Vec::new(),
                                    cemetery: Vec::new(),
                                    creatures: vec!(creature_id),
                                    settlement: Some(SettlementComponent {
                                        leader: Some(creature_id),
                                        material_stock: Vec::new(),
                                    }),
                                    population_peak: (0, 0),
                                    unit_type: UnitType::BanditCamp,
                                    resources: UnitResources {
                                        food: 1.
                                    }
                                });
                                world.events.push(Event::CreateBanditCamp { date: *now, creature_id, unit_id: *unit_id, new_unit_id: new_camp_id });
                            },
                            None => {
                                warn!("No position found for new bandit camp");
                                return;
                            }
                        }
                    }
                    // Removes creature from unit
                    let mut unit = world.units.get_mut(unit_id);
                    unit.remove_creature(&creature_id);
                    unit.resources.food -= 1.;
                    // Chances profession
                    let mut creature = world.creatures.get_mut(&creature_id);
                    creature.profession = Profession::Bandit;
                },
                CreatureSideEffect::AttackNearbyUnits => attack_nearby_unit(world, &mut rng, *unit_id, &mut self.resources),
                CreatureSideEffect::StartPlot(goal) => start_plot(world, creature_id, goal),
                CreatureSideEffect::FindSupportersForPlot => find_supporters_for_plot(world, creature_id),
                CreatureSideEffect::ExecutePlot => execute_plot(world, *unit_id, creature_id, &mut rng, &mut self.resources),
            }
        }


        {
            let mut unit = world.units.get_mut(unit_id);

            self.check_population_peak(now, &mut unit);

            let need_election = match &unit.settlement {
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
            } && unit.creatures.len() > 0;
            if need_election {
                let mut candidates_pool = Vec::new();
                for creature_id in unit.creatures.iter() {
                    let creature = world.creatures.get(creature_id);
                    let age = (*now - creature.birth).year();
                    if age > 18 {
                        candidates_pool.push(creature_id);
                    }
                }
                // TODO: Voting algorithm
                let new_leader = match candidates_pool.len() {
                    0 => unit.creatures[rng.randu_range(0, unit.creatures.len())],
                    _ => *candidates_pool[rng.randu_range(0, candidates_pool.len())],
                };
                // TODO: Can it maybe have no leader?                
                {
                    let mut leader = world.creatures.get_mut(&new_leader);
                    leader.profession = Profession::Ruler;

                    // TODO: Spouse / children of leader being peasant is weird

                }
                unit.settlement.as_mut().expect("No election should be held with no settlement").leader = Some(new_leader);
                world.events.push(Event::NewLeaderElected { date: now.clone(), unit_id: *unit_id, creature_id: new_leader });
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
                None => {
                    break
                }
            }
        }

        for creature_id in change_job_pool {
            let mut creature = world.creatures.get_mut(&creature_id);
            let unit = world.units.get(unit_id);
            let profession = unit.select_new_profession(&mut rng);
            creature.profession = profession;
            drop(creature);
            drop(unit);
            world.events.push(Event::CreatureProfessionChange { date: now.clone(), creature_id: creature_id, new_profession: profession });
        }
    }

    fn make_artifact(artisan_id: &CreatureId, comissioneer_id: Option<&CreatureId>, unit_id: &UnitId, world: &mut World, rng: &mut Rng, resources: &mut Resources) {
        let mut artisan = world.creatures.get_mut(artisan_id);
        let mut unit = world.units.get_mut(unit_id);
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

                let item = ItemFactory::weapon(rng, resources)
                    .quality(quality)
                    .material_pool(unit.settlement.as_mut().and_then(|sett| Some(&mut sett.material_stock)))
                    .named()
                    .make();
                Some(item)
            },
            Profession::Sculptor => {
                if let Some(comissioneer_id) = comissioneer_id {
                    Some(ArtifactFactory::create_statue(rng, resources, *comissioneer_id, &world))
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
                        unit.artifacts.push(id);
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
                world.events.push(Event::ArtifactCreated { date: world.date.clone(), artifact: id, creator: *artisan_id, unit_id: *unit_id });
            }
        }
    }

    fn check_population_peak(&self, now: &WorldDate, unit: &mut Unit) {
        if unit.creatures.len() >= unit.population_peak.1 as usize {
            unit.population_peak = (now.year(), unit.creatures.len() as u32)
        }
    }

    fn find_unit_suitable_pos(&self, rng: &mut Rng, world: &World) -> Option<Coord2> {
        for _ in 0..100 {
            let x = rng.randu_range(0, world.map.size.x());
            let y = rng.randu_range(0, world.map.size.y());
            let tile = world.map.tile(x, y);
            match tile.region_id {
                // Ocean
                0 => continue,
                // Desert
                4 => continue,
                _ => ()
            }
            let candidate = Coord2::xy(x as i32, y as i32);
            let too_close = world.units.iter().any(|unit| {
                let unit = unit.borrow();
                if unit.creatures.len() == 0 {
                    return false;
                }
                return unit.xy.dist_squared(&candidate) < 3. * 3.
            });
            if too_close {
                continue;
            }
            return Some(candidate)
        }
        return None;
    }

    fn find_unit_suitable_position_closeby(&self, center: Coord2, max_radius: i32, rng: &mut Rng, world: &World) -> Option<Coord2> {
        let x_limit = [
            (center.x - max_radius).max(0),
            (center.x + max_radius).min(world.map.size.x() as i32)
        ];
        let y_limit = [
            (center.y - max_radius).max(0),
            (center.y + max_radius).min(world.map.size.y() as i32)
        ];
        for _ in 0..100 {
            let x = rng.randi_range(x_limit[0], x_limit[1]) as usize;
            let y = rng.randi_range(y_limit[0], y_limit[1]) as usize;
            let tile = world.map.tile(x, y);
            match tile.region_id {
                // Ocean
                0 => continue,
                // Desert
                4 => continue,
                _ => ()
            }
            let candidate = Coord2::xy(x as i32, y as i32);
            let too_close = world.units.iter().any(|unit| {
                let unit = unit.borrow();
                return unit.xy.dist_squared(&candidate) < 5. * 5.
            });
            if too_close {
                continue;
            }
            return Some(candidate)
        }
        return None;
    }

}