use std::time::Instant;

use crate::{commons::rng::Rng, engine::geometry::Coord2, resources::resources::Resources, world::{creature::{CauseOfDeath, Profession}, date::WorldDate, item::Item, unit::{Demographics, Unit, UnitId, UnitResources, UnitType}, world::World}, Event};

use super::{creature_simulation::{CreatureSideEffect, CreatureSimulation}, factories::{ArtifactFactory, CreatureFactory}};

pub(crate) struct HistorySimulation {
    pub(crate) date: WorldDate,
    params: HistorySimParams
}

pub(crate) struct HistorySimParams {
    pub(crate) rng: Rng,
    pub(crate) resources: Resources,
    pub(crate) number_of_seed_cities: u8,
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

        for i in 0..self.params.number_of_seed_cities {

            let pos = self.find_unit_suitable_pos(&mut self.params.rng.clone(), &world);
            let pos = match pos {
                None => {
                    println!("## BREAK {}", i);
                    break
                },
                Some(candidate) => candidate,
            };

            let mut unit = Unit {
                xy: pos,
                creatures: Vec::new(),
                cemetery: Vec::new(),
                unit_type: UnitType::City,
                resources: UnitResources {
                    // Enough food for a year
                    food: self.params.seed_cities_population as f32
                },
                leader: None,
                artifacts: Vec::new()
            };

            while unit.creatures.len() < self.params.seed_cities_population as usize {
                
                let family = factory.make_family_or_single(&self.date, self.params.resources.species.id_of("species:human"), world);
                for creature_id in family {
                    unit.creatures.push(creature_id);
                }

            }

            self.params.rng.next();

            world.units.add::<UnitId>(unit);
        }
    }

    pub(crate) fn simulate_step(&mut self, step: WorldDate, world: &mut World) {
        self.date = self.date + step;
        let now = Instant::now();

        let mut stats = (0, Demographics::new());

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
        stats.1.print_console();

        if stats.0 == 0 {
            println!("Dead world.");

            let mut map = (0, 0, 0);
            for creature in world.creatures.iter() {
                let creature = creature.borrow();
                if let Some(death) = &creature.death {
                    match death.1 {
                        CauseOfDeath::OldAge => { map.0 += 1; },
                        CauseOfDeath::Starvation => { map.1 += 1; },
                        CauseOfDeath::Disease => { map.2 += 1; },
                    }
                }
            }

            println!("Deaths:");
            println!("Old age: {}", map.0);
            println!("Starvation: {}", map.1);
            println!("Disease: {}", map.2);

        }

    }

    fn simulate_step_unit(&mut self, world: &mut World, step: &WorldDate, now: &WorldDate, mut rng: Rng, unit_id: &UnitId, stats: &mut (usize, Demographics)) {
        let mut unit = world.units.get_mut(unit_id);
        let mut side_effects = Vec::new();

        let mut resources = unit.resources.clone();


        for creature_id in unit.creatures.iter() {
            let mut creature = world.get_creature_mut(creature_id);
            stats.1.count(now, &creature);

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

            // TODO: Take Tile params in consideration
            if creature.death.is_none() {
                resources = creature.profession.base_resource_production() + resources;
                resources.food -= 1.0;
            }

        }

        unit.resources = resources;

        drop(unit);


        // let mut deferred_side_effects = Vec::new();
        let mut marriage_pool = Vec::new();
        let mut change_job_pool = Vec::new();
        let mut artisan_pool = Vec::new();
        let mut comissions_pool = Vec::new();
        // TODO: Move this to a impl
        for (creature_id, side_effect) in side_effects.into_iter() {
            match side_effect {
                CreatureSideEffect::None => (),
                CreatureSideEffect::Death(cause_of_death) => {
                    {
                        let mut creature = world.get_creature_mut(&creature_id);
                        // CreatureSimulation::kill_creature(&world, now, &mut creature, &cause_of_death);
                        creature.death = Some((now.clone(), cause_of_death));
                        if let Some(spouse_id) = creature.spouse {
                            let mut spouse = world.get_creature_mut(&spouse_id);
                            spouse.spouse = None;
                        }
                        let mut unit = world.units.get_mut(unit_id);
                        let i = unit.creatures.iter().position(|id| *id == creature_id).unwrap();
                        let id = unit.creatures.remove(i);
                        unit.cemetery.push(id);

                        let mut inheritor = None;
                        let mut has_possession = false;

                        if let Some(details) = &creature.details {
                            if details.inventory.len() > 0 {
                                has_possession = true;
                                for candidate_id in creature.offspring.iter() {
                                    let candidate = world.get_creature(&candidate_id);
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
                                let mut inheritor = world.get_creature_mut(&inheritor_id);
                                let mut creature = world.get_creature_mut(&creature_id);
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
                },
                CreatureSideEffect::HaveChild => {

                    let mut creature = world.get_creature_mut(&creature_id);
                    let child = CreatureSimulation::have_child_with_spouse(now, &mut rng, &creature_id, &mut creature);
                    drop(creature);
                    if let Some(child) = child {
                        let father = child.father;
                        let mother = child.mother;
                        // TODO: TODO what???
                        let creature_id = world.add_creature(child);
                        let mut unit = world.units.get_mut(unit_id);
                        unit.creatures.push(creature_id);
                        world.events.push(Event::CreatureBirth { date: now.clone(), creature_id });
                        {
                            let mut father = world.get_creature_mut(&father);
                            father.offspring.push(creature_id);
                        }
                        {
                            let mut mother = world.get_creature_mut(&mother);
                            mother.offspring.push(creature_id);
                        }
                    }
                },
                CreatureSideEffect::LookForMarriage => {
                    let creature = world.get_creature_mut(&creature_id);
                    marriage_pool.push((creature_id, creature.gender));
                },
                CreatureSideEffect::LookForNewJob => {
                    change_job_pool.push(creature_id);
                },
                CreatureSideEffect::MakeArtifact => {
                    let item = ArtifactFactory::create_artifact(&mut rng, &self.params.resources, &self.params.resources.materials.id_of("mat:steel"));
                    let id = world.add_artifact(item);
                    {
                        let mut creature = world.get_creature_mut(&creature_id);             
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
            }
            


        }


        {
            let mut unit = world.units.get_mut(unit_id);
            let need_election = match unit.leader {
                None => true,
                Some(creature_id) => {
                    let creature = world.get_creature(&creature_id);
                    creature.death.is_some()
                }
            } && unit.creatures.len() > 0;
            if need_election {
                let mut candidates_pool = Vec::new();
                for creature_id in unit.creatures.iter() {
                    let creature = world.get_creature(creature_id);
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
                    let mut leader = world.get_creature_mut(&new_leader);
                    leader.profession = Profession::Ruler;

                    // TODO: Spouse / children of leader being peasant is weird

                }
                unit.leader = Some(new_leader);
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
                        let mut creature_a = world.get_creature_mut(&candidate_a.0);
                        let mut creature_b = world.get_creature_mut(&candidate_b.0);
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
            let artisan = world.get_creature(&artisan_id);
            let item = match artisan.profession {
                Profession::Blacksmith => {
                    Some(ArtifactFactory::create_artifact(&mut rng, &self.params.resources, &self.params.resources.materials.id_of("mat:steel")))
                },
                Profession::Sculptor => {
                    Some(ArtifactFactory::create_statue(&self.params.resources, comission_creature_id, &world))
                },
                _ => None
            };
            drop(artisan);
            if let Some(item) = item {
                let id = world.add_artifact(item.clone());
                {
                    let mut creature = world.get_creature_mut(&comission_creature_id);
                    match &item {
                        Item::Statue { material: _, scene: _ } => {
                            let mut unit = world.units.get_mut(unit_id);
                            unit.artifacts.push(id);
                        },
                        Item::Sword(_) | Item::Mace(_) => {
                            creature.details().inventory.push(id);
                        },
                    }
                }
                world.events.push(Event::ArtifactComission { date: now.clone(), creature_id: comission_creature_id, creator_id: artisan_id, item_id: id });
            }
        }

        for creature_id in change_job_pool {
            let mut creature = world.get_creature_mut(&creature_id);
            // Ideally this would look at what the city needs
            let rand_job = rng.randf();
            if rand_job < 0.8 {
                creature.profession = Profession::Peasant;
            } else if rand_job < 0.88 {
                creature.profession = Profession::Farmer;
            } else if rand_job < 0.90 {
                creature.profession = Profession::Sculptor;
            } else if rand_job < 0.95 {
                creature.profession = Profession::Blacksmith;
            } else {
                creature.profession = Profession::Guard;
            }
            let profession = creature.profession;
            drop(creature);
            world.events.push(Event::CreatureProfessionChange { date: now.clone(), creature_id: creature_id, new_profession: profession });
        }

        // return deferred_side_effects
    }

    fn find_unit_suitable_pos(&self, rng: &mut Rng, world: &World) -> Option<Coord2> {
        for _ in 0..20 {
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

}