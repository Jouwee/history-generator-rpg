use std::{cell::RefCell, fs::File, io::Write, time::Instant};

use crate::{commons::rng::Rng, engine::geometry::Coord2, resources::resources::Resources, world::history_sim::structs::CauseOfDeath};

use super::{creature_simulation::{CreatureSideEffect, CreatureSimulation, DeferredUnitSideEffect}, factories::CreatureFactory, structs::{Creature, CreatureGender, CreatureId, Demographics, Event, Profession, Unit, UnitType, World, WorldDate}};

pub struct HistorySimulation {
    world: World,
    date: WorldDate,
    params: HistorySimParams
}

pub struct HistorySimParams {
    pub rng: Rng,
    pub resources: Resources,
    pub number_of_seed_cities: u8,
    pub seed_cities_population: u32,
}

impl HistorySimulation {
    pub fn new(params: HistorySimParams) -> Self {
        HistorySimulation {
            world: World::new(),
            date: WorldDate::year(0),
            params
        }
    }

    pub fn seed(&mut self) {

        let mut factory = CreatureFactory::new(self.params.rng.derive("creature"));

        for _ in 0..self.params.number_of_seed_cities {
            let mut unit = Unit {
                // TODO:
                xy: Coord2::xy(0, 0),
                creatures: Vec::new(),
                unit_type: UnitType::City,
                resources: super::structs::UnitResources {
                    // Enough food for a year
                    food: self.params.seed_cities_population as f32
                }
            };

            while unit.creatures.len() < self.params.seed_cities_population as usize {
                
                let family = factory.make_family_or_single(&self.date, self.params.resources.species.id_of("species:human"), &mut self.world);
                for creature_id in family {
                //     let creature_id = self.world.add_creature(creature);
                    unit.creatures.push(creature_id);
                }

            }

            self.world.units.push(RefCell::new(unit));
        }
    }

    pub fn simulate_step(&mut self, step: WorldDate) {
        self.date = self.date.add(&step);
        let now = Instant::now();

        // let mut side_effects = Vec::new();


        let mut stats = (0, 0, 0, 0, 0);

        for i in 0..self.world.units.len() {
            // let mut unit = unit.borrow_mut();
            self.simulate_step_unit(&step, &self.date.clone(), self.params.rng.clone(), i, &mut stats);
            // for side_effect in local_side_effects.into_iter() {
            //     side_effects.push(side_effect);
            // }
            self.params.rng.next();
        }

        // for side_effect in side_effects.into_iter() {
        //     match side_effect {
        //         DeferredWorldSideEffect::None => (),
        //         DeferredWorldSideEffect::AddCreature(unit_index, creature) => {
        //             let creature_id = self.world.add_creature(creature);
        //             // TODO: Add to offspring of parents
        //             let mut unit = self.world.units[unit_index].borrow_mut();
        //             unit.creatures.push(creature_id);
        //         },
        //     }
        // }

        println!("");
        println!("Elapsed: {:.2?}", now.elapsed());
        println!("Memory: {:?}b", stats.4);
        println!("Year: {}", self.date.year);
        println!("Units: {}", self.world.units.len());
        println!("Creatures: {}", self.world.creatures.len());
        println!("Processed creatures: {}", stats.0);
        println!("Children: {}", stats.1);
        println!("Men of age: {}", stats.2);
        println!("Women of age: {}", stats.3);

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

    fn simulate_step_unit(&mut self, step: &WorldDate, now: &WorldDate, mut rng: Rng, unit_index: usize, stats: &mut (usize, usize, usize, usize, usize)) {
        let mut unit = self.world.units.get(unit_index).unwrap().borrow_mut();
        let mut side_effects = Vec::new();

        let mut resources = unit.resources.clone();

        let mut demographics = Demographics::new();

        for creature_id in unit.creatures.iter() {
            let mut creature = self.world.get_creature_mut(creature_id);
            demographics.count(now, &creature);

            let mut events = Vec::new();

            let side_effect = CreatureSimulation::simulate_step_creature(&self.world, step, now, &mut rng, &unit, creature_id, &mut creature, &mut events);
            side_effects.push((*creature_id, side_effect));

            let age = now.subtract(&creature.birth).year as f32;

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
        for (creature_id, side_effect) in side_effects.into_iter() {
            match side_effect {
                CreatureSideEffect::None => (),
                CreatureSideEffect::Death(cause_of_death) => {
                    {
                        let mut creature = self.world.get_creature_mut(&creature_id);
                        CreatureSimulation::kill_creature(&self.world, now, &mut creature, &cause_of_death);
                        let mut unit = self.world.units.get(unit_index).unwrap().borrow_mut();
                        let i = unit.creatures.iter().position(|id| *id == creature_id).unwrap();
                        unit.creatures.remove(i);
                    }
                    self.world.events.push(Event::CreatureDeath { date: now.clone(), creature_id: creature_id, cause_of_death: cause_of_death });
                },
                CreatureSideEffect::HaveChild => {
                    let mut creature = self.world.get_creature_mut(&creature_id);
                    let child = CreatureSimulation::have_child_with_spouse(now, &mut rng, &creature_id, &mut creature);
                    drop(creature);
                    if let Some(child) = child {
                        // TODO:
                        let creature_id = self.world.add_creature(child);
                        let mut unit = self.world.units.get(unit_index).unwrap().borrow_mut();
                        unit.creatures.push(creature_id);
                        // deferred_side_effects.push(DeferredWorldSideEffect::AddCreature(unit_index, child));
                        self.world.events.push(Event::CreatureBirth { date: now.clone(), creature_id: creature_id });
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
                    // TODO:
                }
            }



            // match side_effect {
            //     DeferredUnitSideEffect::None => (),
            //     DeferredUnitSideEffect::RemoveCreature(creature_id) => unit.creatures.retain(|id| *id != creature_id),
            //     DeferredUnitSideEffect::AddCreature(creature) => {
            //         deferred_side_effects.push(DeferredWorldSideEffect::AddCreature(unit_index, creature));
            //     },
            //     DeferredUnitSideEffect::LookForMarriage(creature, gender) => marriage_pool.push((creature, gender)),
            // }
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

        for creature_id in change_job_pool {
            let mut creature = self.world.get_creature_mut(&creature_id);
            // Ideally this would look at what the city needs
            let rand_job = rng.randf();
            if rand_job < 0.8 {
                creature.profession = Profession::Peasant;
            } else if rand_job < 0.9 {
                creature.profession = Profession::Farmer;
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

    pub fn dump_events(&self, filename: &str) {
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
            }
            
        }
    }

    fn creature_desc(&self, creature_id: &CreatureId, date: &WorldDate) -> String {
        let creature = self.world.get_creature(creature_id);
        let age = date.subtract(&creature.birth).year;
        let mut gender = "M";
        if creature.gender.is_female() {
            gender = "F";
        }
        return String::from(format!("[{:?}, {:?} {:?}]", creature_id, age, gender))
    }


    fn date_desc(&self, date: &WorldDate) -> String {
        return String::from(format!("{:?}", date.year))
    }

}

enum DeferredWorldSideEffect {
    None,
    AddCreature(/* Unit index */ usize, Creature),
}
