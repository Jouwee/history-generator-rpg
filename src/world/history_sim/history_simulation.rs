use std::time::Instant;

use crate::{commons::rng::Rng, engine::geometry::Coord2, resources::resources::Resources, world::{creature::{CauseOfDeath, CreatureId, Profession, SIM_FLAG_GREAT_BEAST}, date::WorldDate, history_sim::battle_simulator::BattleSimulator, unit::{Demographics, SettlementComponent, Unit, UnitId, UnitResources, UnitType}, world::World}, Event};

use super::{creature_simulation::{CreatureSideEffect, CreatureSimulation}, factories::{ArtifactFactory, CreatureFactory}};

pub(crate) struct HistorySimulation {
    pub(crate) date: WorldDate,
    params: HistorySimParams
}

pub(crate) struct HistorySimParams {
    pub(crate) rng: Rng,
    pub(crate) resources: Resources,
    pub(crate) number_of_seed_cities: u16,
    pub(crate) seed_cities_population: u32,
}

impl HistorySimulation {
    pub(crate) fn new(params: HistorySimParams) -> Self {
        HistorySimulation {
            date: WorldDate::new(0, 0, 0),
            params
        }
    }

    pub(crate) fn seed(&mut self, world: &mut World) {

        let mut factory = CreatureFactory::new(self.params.rng.derive("creature"));

        for _ in 0..self.params.number_of_seed_cities {

            let pos = self.find_unit_suitable_pos(&mut self.params.rng.clone(), &world);
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
                    food: self.params.seed_cities_population as f32
                },
                settlement: Some(SettlementComponent {
                    leader: None,
                }),
                artifacts: Vec::new(),
                population_peak: (0, 0),
                unit_type: UnitType::Village
            };

            while unit.creatures.len() < self.params.seed_cities_population as usize {
                
                let family = factory.make_family_or_single(&self.date, self.params.resources.species.id_of("species:human"), world, &self.params.resources);
                for creature_id in family {
                    unit.creatures.push(creature_id);
                }

            }

            self.params.rng.next();

            world.units.add::<UnitId>(unit);
        }
    }

    pub(crate) fn simulate_step(&mut self, step: WorldDate, world: &mut World) -> bool {
        self.date = self.date + step;
        world.date = self.date.clone();
        let now = Instant::now();


        // TODO(tfWpiQPF): Find a cooler way to spawn
        if self.params.rng.rand_chance(0.3) {
            let pos = self.find_unit_suitable_pos(&mut self.params.rng.clone(), world);

            if let Some(pos) = pos {
                let species = self.params.resources.species.id_of("species:varningr");
                let mut factory = CreatureFactory::new(self.params.rng.derive("creature"));
                let creature = factory.make_single(species, 10, SIM_FLAG_GREAT_BEAST, world, &self.params.resources);
                let unit = Unit {
                    // TODO(PaZs1uBR): These don't make sense
                    artifacts: Vec::new(),
                    cemetery: Vec::new(),
                    creatures: vec!(creature),
                    settlement: None,
                    population_peak: (0, 0),
                    resources: UnitResources { food: 0. },
                    // TODO(PaZs1uBR): Unit type
                    unit_type: UnitType::BanditCamp,
                    xy: pos
                };

                println!("[!!!] spawn spider");

                world.units.add::<UnitId>(unit);

            } else {
                println!("[!!!] failed to spawn");
            }


        }



        let mut stats = (0, 0, 0, Demographics::new());

        for id in world.units.iter_ids::<UnitId>() {
            self.simulate_step_unit(world, &step, &self.date.clone(), self.params.rng.clone(), &id, &mut stats);
            self.params.rng.next();
        }


        println!("");
        println!("Elapsed: {:.2?}", now.elapsed());
        println!("Memory: {:?}b", stats.0);
        println!("Year: {}", self.date.year());
        println!("Total creatures: {}", world.creatures.len());
        println!("Total events: {}", world.events.len());
        println!("Populated units: {}", stats.2);
        println!("Desolate units: {}", stats.1);
        stats.3.print_console();

        if stats.0 == 0 {
            println!("Dead world.");

            let mut map = (0, 0, 0, 0);
            for creature in world.creatures.iter() {
                let creature = creature.borrow();
                if let Some(death) = &creature.death {
                    match death.1 {
                        CauseOfDeath::OldAge => { map.0 += 1; },
                        CauseOfDeath::Starvation => { map.1 += 1; },
                        CauseOfDeath::Disease => { map.2 += 1; },
                        CauseOfDeath::KilledInBattle(_) => { map.3 += 1; },
                    }
                }
            }

            println!("Deaths:");
            println!("Old age: {}", map.0);
            println!("Starvation: {}", map.1);
            println!("Disease: {}", map.2);
            println!("Battle: {}", map.3);
            return false;
        }
        return true;

    }

    fn simulate_step_unit(&mut self, world: &mut World, step: &WorldDate, now: &WorldDate, mut rng: Rng, unit_id: &UnitId, stats: &mut (usize, usize, usize, Demographics)) {
        let mut unit = world.units.get_mut(unit_id);
        let mut side_effects = Vec::new();

        let mut resources = unit.resources.clone();

        let unit_tile = world.map.tile(unit.xy.x as usize, unit.xy.y as usize);

        if unit.creatures.len() == 0 {
            stats.1 += 1;
        } else {
            stats.2 += 1;
        }
        

        for creature_id in unit.creatures.iter() {
            let mut creature = world.creatures.get_mut(creature_id);
            stats.3.count(now, &creature);

            let side_effect = CreatureSimulation::simulate_step_creature(step, now, &mut rng, &unit, &mut creature);
            side_effects.push((*creature_id, side_effect));

            stats.0 += std::mem::size_of_val(&creature.species) +
                    std::mem::size_of_val(&creature.birth) +
                    std::mem::size_of_val(&creature.gender) +
                    std::mem::size_of_val(&creature.death) +
                    std::mem::size_of_val(&creature.profession) +
                    std::mem::size_of_val(&creature.father) +
                    std::mem::size_of_val(&creature.mother) +
                    std::mem::size_of_val(&creature.spouse) +
                    std::mem::size_of_val(&creature.offspring);

            if creature.death.is_none() {
                let mut production = creature.profession.base_resource_production();
                production.food = production.food * unit_tile.soil_fertility;
                resources = production + resources;
                resources.food -= 1.0;
            }

        }

        unit.resources = resources;

        drop(unit);

        let mut marriage_pool = Vec::new();
        let mut change_job_pool = Vec::new();
        let mut artisan_pool = Vec::new();
        let mut comissions_pool = Vec::new();
        // TODO: Move all of these to a impl
        for (creature_id, side_effect) in side_effects.into_iter() {
            match side_effect {
                CreatureSideEffect::None => (),
                CreatureSideEffect::Death(cause_of_death) => Self::kill_creature(world, creature_id, *unit_id, cause_of_death),
                CreatureSideEffect::HaveChild => {

                    let unit = world.units.get(unit_id);
                    // TODO: Hard limit
                    if unit.creatures.len() > 30 {
                        continue;
                    }
                    drop(unit);

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
                CreatureSideEffect::MakeArtifact => {
                    let item = ArtifactFactory::create_artifact(&mut rng, &self.params.resources);
                    let id = world.artifacts.add(item);
                    {
                        let mut creature = world.creatures.get_mut(&creature_id);             
                        creature.details().inventory.push(id);
                    }
                    world.events.push(Event::ArtifactCreated { date: *now, artifact: id, creator: creature_id });
                },
                CreatureSideEffect::ArtisanLookingForComission => {
                    artisan_pool.push(creature_id);
                }
                CreatureSideEffect::ComissionArtifact => {
                    comissions_pool.push(creature_id);
                }
                CreatureSideEffect::BecomeBandit => {
                    
                    // TODO: Steal close artifact?
                    
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
                                        leader: Some(creature_id)
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
                                println!("[WARN] No position found for new bandit camp");
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
                CreatureSideEffect::AttackNearbyUnits => Self::attack_nearby_unit(world, &mut rng, *unit_id)
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

        for comission_creature_id in comissions_pool {
            if artisan_pool.len() == 0 {
                break;
            }
            let artisan_id = artisan_pool.remove(rng.randu_range(0, artisan_pool.len()));
            let artisan = world.creatures.get(&artisan_id);
            let item = match artisan.profession {
                Profession::Blacksmith => {
                    Some(ArtifactFactory::create_artifact(&mut rng, &self.params.resources))
                },
                Profession::Sculptor => {
                    Some(ArtifactFactory::create_statue(&self.params.resources, comission_creature_id, &world))
                },
                _ => None
            };
            drop(artisan);
            if let Some(item) = item {
                let id = world.artifacts.add(item.clone());
                {
                    let mut creature = world.creatures.get_mut(&comission_creature_id);
                    match &item.artwork_scene {
                        Some(_) => {
                            let mut unit = world.units.get_mut(unit_id);
                            unit.artifacts.push(id);
                        },
                        None => {
                            creature.details().inventory.push(id);
                        },
                    }
                }
                world.events.push(Event::ArtifactComission { date: now.clone(), creature_id: comission_creature_id, creator_id: artisan_id, item_id: id });
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

    fn kill_creature(world: &mut World, creature_id: CreatureId, unit_id: UnitId, cause_of_death: CauseOfDeath) {
        let now = world.date.clone();
        {
            let mut creature = world.creatures.get_mut(&creature_id);
            if creature.death.is_some() {
                println!("[WARN] Trying to kill already dead creature");
                return;
            }
            creature.death = Some((now.clone(), cause_of_death));
            if let Some(spouse_id) = creature.spouse {
                let mut spouse = world.creatures.get_mut(&spouse_id);
                spouse.spouse = None;
            }
            let mut unit = world.units.get_mut(&unit_id);
            let i = unit.creatures.iter().position(|id| *id == creature_id).unwrap();
            let id = unit.creatures.remove(i);
            unit.cemetery.push(id);

            let mut inheritor = None;
            let mut has_possession = false;

            if let Some(details) = &creature.details {
                if details.inventory.len() > 0 {
                    has_possession = true;
                    for candidate_id in creature.offspring.iter() {
                        let candidate = world.creatures.get(candidate_id);
                        if candidate.death.is_none() {
                            inheritor = Some((*candidate_id, details.inventory.clone()));
                            break;
                        }
                    }
                }
            }

            drop(creature);

            if has_possession {
                if let Some((inheritor_id, inventory)) = inheritor {
                    let mut inheritor = world.creatures.get_mut(&inheritor_id);
                    let mut creature = world.creatures.get_mut(&creature_id);
                    creature.details().inventory.clear();
                    inheritor.details().inventory.append(&mut inventory.clone());
                    drop(creature);
                    drop(inheritor);
                    for item in inventory.iter() {
                        world.events.push(Event::InheritedArtifact { date: now.clone(), creature_id: inheritor_id, from: creature_id, item: *item });
                    }
                } else {
                    world.events.push(Event::BurriedWithPosessions { date: now.clone(), creature_id });
                }
            }

            // TODO: Inherit leadership

        }
        world.events.push(Event::CreatureDeath { date: now.clone(), creature_id: creature_id, cause_of_death: cause_of_death });
    }

    fn attack_nearby_unit(world: &mut World, rng: &mut Rng, unit_id: UnitId) {
        let mut candidates = Vec::new();
        {
            let source_unit = world.units.get(&unit_id);
            for (id, unit) in world.units.iter_id_val::<UnitId>() {
                if id != unit_id {
                    let unit = unit.borrow();
                    // TODO(PaZs1uBR): Magic number
                    if unit.creatures.len() > 0 && unit.xy.dist_squared(&source_unit.xy) < 20.*20. {
                        candidates.push(id);
                        break;
                    }
                }
            }
        }

        if let Some(target) = rng.item(&candidates) {
            let battle;
            {
                let unit = world.units.get(&unit_id);
                let target_unit = world.units.get(target);
                battle = BattleSimulator::simulate_attack(unit_id, &unit, *target, &target_unit, rng, world);
            }

            // TODO(PaZs1uBR): Log
            println!("#B Battle ---------");
            for l in battle.log.iter() {
                println!("#B {}", l);
            }

            for (id, unit_id, killer) in battle.deaths {
                let cause_of_death = CauseOfDeath::KilledInBattle(killer);
                // TODO(PaZs1uBR): They died at the place they were fighting, not where they came from.
                Self::kill_creature(world, id, unit_id, cause_of_death);
            }

            for (id, xp) in battle.xp_add {
                let mut creature = world.creatures.get_mut(&id);
                creature.experience += xp;
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
                return unit.xy.dist_squared(&candidate) < 5. * 5.
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