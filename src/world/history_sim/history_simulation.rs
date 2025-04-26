use std::{fs::File, io::Write, time::Instant};

use crate::{commons::rng::Rng, engine::geometry::Coord2, resources::resources::Resources, world::{creature::{CauseOfDeath, CreatureGender, CreatureId, Profession}, date::WorldDate, item::Item, unit::{Unit, UnitId, UnitResources, UnitType}, world::World}};

use super::{creature_simulation::{CreatureSideEffect, CreatureSimulation}, factories::{ArtifactFactory, CreatureFactory}, structs::{Demographics, Event}};

pub(crate) struct HistorySimulation {
    world: World,
    date: WorldDate,
    params: HistorySimParams
}

pub(crate) struct HistorySimParams {
    pub(crate) rng: Rng,
    pub(crate) resources: Resources,
    pub(crate) number_of_seed_cities: u8,
    pub(crate) seed_cities_population: u32,
}

impl HistorySimulation {
    pub(crate) fn new(params: HistorySimParams, world: World) -> Self {
        HistorySimulation {
            world,
            date: WorldDate::new(0, 0, 0),
            params
        }
    }

    pub(crate) fn into_world(self) -> World {
        return self.world;
    }

    pub(crate) fn seed(&mut self) {

        let mut factory = CreatureFactory::new(self.params.rng.derive("creature"));

        let mut x = 110;
        let mut y = 130;

        for _ in 0..self.params.number_of_seed_cities {
            let mut unit = Unit {
                // TODO:
                xy: Coord2::xy(x, y),
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

            x += 2;
            if x > 140 {
                y +=2;
                x = 130;
            }

            while unit.creatures.len() < self.params.seed_cities_population as usize {
                
                let family = factory.make_family_or_single(&self.date, self.params.resources.species.id_of("species:human"), &mut self.world);
                for creature_id in family {
                //     let creature_id = self.world.add_creature(creature);
                    unit.creatures.push(creature_id);
                }

            }

            self.world.units.add::<UnitId>(unit);
        }
    }

    pub(crate) fn simulate_step(&mut self, step: WorldDate) {
        self.date = self.date + step;
        let now = Instant::now();

        // let mut side_effects = Vec::new();


        let mut stats = (0, 0, 0, 0, 0);

        for id in self.world.units.iter_ids::<UnitId>() {
            // let mut unit = unit.borrow_mut();
            self.simulate_step_unit(&step, &self.date.clone(), self.params.rng.clone(), &id, &mut stats);
            // for side_effect in local_side_effects.into_iter() {
            //     side_effects.push(side_effect);
            // }
            self.params.rng.next();
        }


        println!("");
        println!("Elapsed: {:.2?}", now.elapsed());
        println!("Memory: {:?}b", stats.4);
        println!("Year: {}", self.date.year());
        // println!("Units: {}", self.world.units.len());
        // println!("Creatures: {}", self.world.creatures.len());
        // println!("Processed creatures: {}", stats.0);
        // println!("Children: {}", stats.1);
        // println!("Men of age: {}", stats.2);
        // println!("Women of age: {}", stats.3);

        if stats.0 == 0 {
            println!("Dead world.");

            let mut map = (0, 0, 0);
            for creature in self.world.creatures.iter() {
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

    fn simulate_step_unit(&mut self, step: &WorldDate, now: &WorldDate, mut rng: Rng, unit_id: &UnitId, stats: &mut (usize, usize, usize, usize, usize)) {
        let mut unit = self.world.units.get_mut(unit_id);
        let mut side_effects = Vec::new();

        let mut resources = unit.resources.clone();

        let mut demographics = Demographics::new();

        for creature_id in unit.creatures.iter() {
            let mut creature = self.world.get_creature_mut(creature_id);
            demographics.count(now, &creature);

            let mut events = Vec::new();

            let side_effect = CreatureSimulation::simulate_step_creature(&self.world, step, now, &mut rng, &unit, creature_id, &mut creature, &mut events);
            side_effects.push((*creature_id, side_effect));

            let age = (*now - creature.birth).year() as f32;

            stats.0 += 1;
            if age < 18. {
                stats.1 += 1;
            } else {
                if creature.gender == CreatureGender::Male {
                    stats.2 += 1;
                } else {
                    stats.3 += 1;
                }
            }
            stats.4 += std::mem::size_of_val(&creature.species) +
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

        demographics.print_console();

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
                        let mut creature = self.world.get_creature_mut(&creature_id);
                        // CreatureSimulation::kill_creature(&self.world, now, &mut creature, &cause_of_death);
                        creature.death = Some((now.clone(), cause_of_death));
                        if let Some(spouse_id) = creature.spouse {
                            let mut spouse = self.world.get_creature_mut(&spouse_id);
                            spouse.spouse = None;
                        }
                        let mut unit = self.world.units.get_mut(unit_id);
                        let i = unit.creatures.iter().position(|id| *id == creature_id).unwrap();
                        let id = unit.creatures.remove(i);
                        unit.cemetery.push(id);

                        let mut inheritor = None;
                        let mut has_possession = false;

                        if let Some(details) = &creature.details {
                            if details.inventory.len() > 0 {
                                has_possession = true;
                                for candidate_id in creature.offspring.iter() {
                                    let candidate = self.world.get_creature(&candidate_id);
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
                                let mut inheritor = self.world.get_creature_mut(&inheritor_id);
                                let mut creature = self.world.get_creature_mut(&creature_id);
                                creature.details().inventory.clear();
                                inheritor.details().inventory.append(&mut inventory.clone());
                                drop(creature);
                                drop(inheritor);
                                for item in inventory.iter() {
                                    self.world.events.push(Event::InheritedArtifact { date: now.clone(), creature_id: inheritor_id, from: creature_id, item: *item });
                                }
                            } else {
                                self.world.events.push(Event::BurriedWithPosessions { date: now.clone(), creature_id });
                            }
                        }

                        // TODO: Inherit leadership

                    }
                    self.world.events.push(Event::CreatureDeath { date: now.clone(), creature_id: creature_id, cause_of_death: cause_of_death });
                },
                CreatureSideEffect::HaveChild => {

                    let mut creature = self.world.get_creature_mut(&creature_id);
                    let child = CreatureSimulation::have_child_with_spouse(now, &mut rng, &creature_id, &mut creature);
                    drop(creature);
                    if let Some(child) = child {
                        let father = child.father;
                        let mother = child.mother;
                        // TODO: TODO what???
                        let creature_id = self.world.add_creature(child);
                        let mut unit = self.world.units.get_mut(unit_id);
                        unit.creatures.push(creature_id);
                        self.world.events.push(Event::CreatureBirth { date: now.clone(), creature_id });
                        {
                            let mut father = self.world.get_creature_mut(&father);
                            father.offspring.push(creature_id);
                        }
                        {
                            let mut mother = self.world.get_creature_mut(&mother);
                            mother.offspring.push(creature_id);
                        }
                    }
                },
                CreatureSideEffect::LookForMarriage => {
                    let creature = self.world.get_creature_mut(&creature_id);
                    marriage_pool.push((creature_id, creature.gender));
                },
                CreatureSideEffect::LookForNewJob => {
                    change_job_pool.push(creature_id);
                },
                CreatureSideEffect::MakeArtifact => {
                    let item = ArtifactFactory::create_artifact(&mut rng, &self.params.resources, &self.params.resources.materials.id_of("mat:steel"));
                    let id = self.world.add_artifact(item);
                    {
                        let mut creature = self.world.get_creature_mut(&creature_id);             
                        creature.details().inventory.push(id);
                    }
                    self.world.events.push(Event::ArtifactCreated { date: *now, artifact: id, creator: creature_id });
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
            let mut unit = self.world.units.get_mut(unit_id);
            let need_election = match unit.leader {
                None => true,
                Some(creature_id) => {
                    let creature = self.world.get_creature(&creature_id);
                    creature.death.is_some()
                }
            } && unit.creatures.len() > 0;
            if need_election {
                let mut candidates_pool = Vec::new();
                for creature_id in unit.creatures.iter() {
                    let creature = self.world.get_creature(creature_id);
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
                    let mut leader = self.world.get_creature_mut(&new_leader);
                    leader.profession = Profession::Ruler;

                    // TODO: Spouse / children of leader being peasant is weird

                }
                unit.leader = Some(new_leader);
                self.world.events.push(Event::NewLeaderElected { date: now.clone(), unit_id: *unit_id, creature_id: new_leader });
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
                        let mut creature_a = self.world.get_creature_mut(&candidate_a.0);
                        let mut creature_b = self.world.get_creature_mut(&candidate_b.0);
                        creature_a.spouse = Some(candidate_b.0);
                        creature_b.spouse = Some(candidate_a.0);
                    }
                    self.world.events.push(Event::CreatureMarriage { date: now.clone(), creature_id: candidate_a.0, spouse_id: candidate_b.0 });
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
            let artisan = self.world.get_creature(&artisan_id);
            let item = match artisan.profession {
                Profession::Blacksmith => {
                    Some(ArtifactFactory::create_artifact(&mut rng, &self.params.resources, &self.params.resources.materials.id_of("mat:steel")))
                },
                Profession::Sculptor => {
                    Some(ArtifactFactory::create_statue(&mut rng, &self.params.resources, &self.params.resources.materials.id_of("mat:steel"), comission_creature_id, &self.world))
                },
                _ => None
            };
            drop(artisan);
            if let Some(item) = item {
                let id = self.world.add_artifact(item.clone());
                {
                    let mut creature = self.world.get_creature_mut(&comission_creature_id);
                    // TODO: Actually a statue is not an item. It will be place in the city.
                    match &item {
                        Item::Statue { material: _, scene: _ } => {
                            let mut unit = self.world.units.get_mut(unit_id);
                            unit.artifacts.push(id);
                        },
                        Item::Sword(_) | Item::Mace(_) => {
                            creature.details().inventory.push(id);
                        },
                    }
                }
                self.world.events.push(Event::ArtifactComission { date: now.clone(), creature_id: comission_creature_id, creator_id: artisan_id, item_id: id });
            }
        }

        for creature_id in change_job_pool {
            let mut creature = self.world.get_creature_mut(&creature_id);
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
            self.world.events.push(Event::CreatureProfessionChange { date: now.clone(), creature_id: creature_id, new_profession: profession });
        }

        // return deferred_side_effects
    }

    pub(crate) fn dump_events(&self, filename: &str) {
        let mut f = File::create(filename).unwrap();
        println!("{:?} events", self.world.events.len());
        for event in self.world.events.iter() {
            match event {
                Event::CreatureBirth { date, creature_id } => {
                    let creature = self.world.get_creature(creature_id);
                    let name = self.creature_desc(creature_id, date);
                    let father = self.creature_desc(&creature.father, date);
                    let mother = self.creature_desc(&creature.mother, date);
                    writeln!(&mut f, "{}, {} was born. Father: {:?}, Mother: {:?}", self.date_desc(date), name, father, mother).unwrap();
                },
                Event::CreatureDeath { date, creature_id, cause_of_death } => {
                    let name = self.creature_desc(creature_id, date);
                    writeln!(&mut f, "{}, {} died of {:?}", self.date_desc(date), name, cause_of_death).unwrap();
                },
                Event::CreatureMarriage { date, creature_id, spouse_id } => {
                    let name_a = self.creature_desc(creature_id, date);
                    let name_b = self.creature_desc(spouse_id, date);
                    writeln!(&mut f, "{}, {} and {} married", self.date_desc(date), name_a, name_b).unwrap();
                },
                Event::CreatureProfessionChange { date, creature_id, new_profession } => {
                    let name = self.creature_desc(creature_id, date);
                    writeln!(&mut f, "{}, {} became a {:?}", self.date_desc(date), name, new_profession).unwrap();
                },
                Event::ArtifactCreated { date, artifact, creator } => {
                    let name = self.creature_desc(creator, date);
                    let artifact = self.world.artifacts.get(artifact);
                    writeln!(&mut f, "{}, {} created {:?}", self.date_desc(date), name, artifact.name(&self.params.resources.materials)).unwrap();
                },
                Event::BurriedWithPosessions { date, creature_id } => {
                    let name = self.creature_desc(creature_id, date);
                    writeln!(&mut f, "{}, {} was buried with their possessions", self.date_desc(date), name).unwrap();
                },
                Event::InheritedArtifact { date, creature_id, from, item } => {
                    let name = self.creature_desc(creature_id, date);
                    let name_b = self.creature_desc(from, date);
                    let artifact = self.world.artifacts.get(item);
                    writeln!(&mut f, "{}, {} inherited {} from {:?}", self.date_desc(date), name, artifact.name(&self.params.resources.materials), name_b).unwrap();
                },
                Event::ArtifactComission { date, creature_id, creator_id, item_id } => {
                    let name = self.creature_desc(creature_id, date);
                    let name_b = self.creature_desc(creator_id, date);
                    let artifact = self.world.artifacts.get(item_id);
                    let creature = self.world.get_creature(creature_id);
                    let age = (*date - creature.birth).year();
                    writeln!(&mut f, "{}, {} commissioned {} from {:?} for his {}th birthday", self.date_desc(date), name, artifact.name(&self.params.resources.materials), name_b, age).unwrap();
                },
                Event::NewLeaderElected { date, unit_id, creature_id } => {
                    let name = self.creature_desc(creature_id, date);
                    writeln!(&mut f, "{}, {} was elected new leader of {:?}", self.date_desc(date), name, *unit_id).unwrap();
                }
            }
            
        }
    }

    fn creature_desc(&self, creature_id: &CreatureId, date: &WorldDate) -> String {
        let creature = self.world.get_creature(creature_id);
        let age = (*date - creature.birth).year();
        let mut gender = "M";
        if creature.gender.is_female() {
            gender = "F";
        }
        return String::from(format!("[{:?}, {:?} {:?}]", creature_id, age, gender))
    }


    fn date_desc(&self, date: &WorldDate) -> String {
        return String::from(format!("{}-{}-{}", date.year(), date.month(), date.day()))
    }

}