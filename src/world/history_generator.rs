use std::{borrow::BorrowMut, collections::HashMap, time::Instant};

use crate::{commons::{history_vec::{HistoryVec, Id}, rng::Rng, strings::Strings}, engine::{geometry::{Coord2, Size2D}, Point2D}, world::{faction::{Faction, FactionRelation}, person::{Importance, NextOfKin, Person, PersonSex, Relative}, topology::{WorldTopology, WorldTopologyGenerationParameters}, world::People}, BattleResult as BattleResult_old, MarriageEvent, NewSettlementLeaderEvent, PeaceDeclaredEvent, SettlementFoundedEvent, SiegeEvent, SimplePersonEvent, WarDeclaredEvent, WorldEventDate, WorldEventEnum, WorldEvents};

use super::{attributes::Attributes, battle_simulator::{BattleForce, BattleResult}, culture::Culture, person::CivilizedComponent, region::Region, settlement::{Settlement, SettlementBuilder}, species::{Species, SpeciesIntelligence}, world::World};


pub struct WorldGenerationParameters {
    pub seed: u32,
    pub cultures: Vec<Culture>,
    pub regions: Vec<Region>,
    pub great_beasts_yearly_spawn_chance: f32
}

pub struct WorldHistoryGenerator {
    rng: Rng,
    pub year: u32,
    parameters: WorldGenerationParameters,
    pub world: World,
    next_person_id: Id,
}

impl WorldHistoryGenerator {

    pub fn seed_world(parameters: WorldGenerationParameters) -> WorldHistoryGenerator {
        let rng = Rng::seeded(parameters.seed);
       
        let mut params = WorldTopologyGenerationParameters {
            rng: rng.derive("topology"),
            num_plate_tectonics: 25
        };

        let mut world_map = WorldTopology::new(Size2D(256, 256));
        let now = Instant::now();
        world_map.plate_tectonics(&mut params);
        println!("Plate tectonics in {:.2?}", now.elapsed());
        let now: Instant = Instant::now();
        world_map.precipitation(&mut params);
        println!("Precipitation {:.2?}", now.elapsed());
        // let now: Instant = Instant::now();
        // world_map.erosion(&mut params);
        // println!("Erosion {:.2?}", now.elapsed());
        world_map.noise(&rng, &parameters.regions);

        let mut world = World {
            map: world_map,
            cultures: HashMap::new(),
            species: Self::load_species(),
            factions: HistoryVec::new(),
            settlements: HistoryVec::new(),
            people: People::new(),
            events: WorldEvents::new()
        };


        let mut culture_id = Id(0);
        for culture in parameters.cultures.iter() {
            let mut culture = culture.clone();
            culture.id = culture_id.next();
            world.cultures.insert(culture.id, culture);
        }

        let mut person_id = Id(0);

        let event_date = WorldEventDate { year: 1 };

        let mut generator = WorldHistoryGenerator {
            rng,
            parameters,
            world,
            year: 1,
            next_person_id: person_id
        };

        // Generate starter people
        for _ in 0..10 {
            generator.rng.next();
            let id = person_id.next();
            let culture = generator.world.cultures.get(&Id(generator.rng.randu_range(0, culture_id.seq()))).unwrap();
            let faction = Faction::new(&generator.rng, id);
            let faction_id = generator.world.factions.insert(faction);
            // TODO: Position
            let species = generator.world.species.get(&Id(0)).unwrap(); // Human
            let person = Person::new(id, species, Importance::Important, 1, Coord2::xy(0, 0))
                .civilization(&Some(CivilizedComponent {
                    culture: culture.id,
                    faction: faction_id,
                    faction_relation: FactionRelation::Leader,
                    leader_of_settlement: None
                }));
            let person = generator.name_person(person, &None);
            generator.world.events.push(event_date, WorldEventEnum::PersonBorn(SimplePersonEvent { person_id: person.id }));
            generator.world.people.insert(person);
        }
        generator.next_person_id = person_id;
        return generator;
    }

    fn load_species() -> HashMap<Id, Species> {
        let mut map = HashMap::new();
        map.insert(Id(0), Species::new(Id(0), "human"));
        map.insert(Id(1), Species::new(Id(1), "leshen")
            .intelligence(SpeciesIntelligence::Instinctive)
            .attributes(Attributes { strength: 45 })
            .lifetime(300)
            .fertility(0.));
        map.insert(Id(2), Species::new(Id(2), "fiend")
            .intelligence(SpeciesIntelligence::Instinctive)
            .attributes(Attributes { strength: 35 })
            .lifetime(200)
            .fertility(0.));
        map
    }

    pub fn simulate_year(&mut self) {
        self.year = self.year + 1;
        let year = self.year;
        let event_date = WorldEventDate { year };

        let mut new_people: Vec<Person> = Vec::new();

        for (person_id, person) in self.world.people.iter() {
            let mut person = person.borrow_mut();
            let species = self.world.species.get(&person.species).unwrap();
            let age = (year - person.birth) as f32;
            if self.rng.rand_chance(f32::min(1.0, (age / species.lifetime.max_age as f32).powf(5.0))) {
                person.death = year;
                self.world.events.push(event_date, WorldEventEnum::PersonDeath(SimplePersonEvent { person_id: person.id }));
                continue
            }

            if species.lifetime.is_adult(age) {
                if species.intelligence == SpeciesIntelligence::Instinctive {
                    if let Some(battle) = self.beast_hunt_nearby(&mut person) {

                        // TODO: How can I take this out of here?

                        for id in battle.0.creature_casualties.iter() {
                            if id == person_id {
                                person.death = year;
                            } else {
                                let mut killed = self.world.people.get_mut(id).unwrap();
                                killed.death = year;
                            }
                            self.world.events.push(event_date, WorldEventEnum::PersonDeath(SimplePersonEvent { person_id: *id }));
                        }

                        if let Some(settlement_id) = battle.0.belligerent_settlement {
                            let mut settlement = self.world.settlements.get_mut(&settlement_id);
                            settlement.kill_military(battle.0.army_casualties, &self.rng);
                            settlement.kill_civilians(battle.0.civilian_casualties);
                        }
                       
                        for id in battle.1.creature_casualties.iter() {
                            if id == person_id {
                                person.death = year;
                            } else {
                                let mut killed = self.world.people.get_mut(id).unwrap();
                                killed.death = year;
                            }
                            self.world.events.push(event_date, WorldEventEnum::PersonDeath(SimplePersonEvent { person_id: *id }));
                        }

                        if let Some(settlement_id) = battle.1.belligerent_settlement {
                            let mut settlement = self.world.settlements.get_mut(&settlement_id);
                            settlement.kill_military(battle.1.army_casualties, &self.rng);
                            settlement.kill_civilians(battle.1.civilian_casualties);
                        }

                        self.world.events.push(event_date, WorldEventEnum::Battle(crate::BattleEvent { battle_result: battle }));
                    }
                    continue;
                }
            }
            if species.intelligence == SpeciesIntelligence::Civilized {

                if age > 18.0 && person.spouse().is_none() && self.rng.rand_chance(0.1) {
                    self.rng.next();
                    let id = self.next_person_id.next();
                    let species = self.world.species.get(&person.species).unwrap();
                    let spouse = Person::new(id, &species, person.importance.lower(), year, person.position)
                        .civilization(&person.civ);
                    let mut spouse = self.name_person(spouse, &None);
                    spouse.last_name = person.last_name.clone();
                    spouse.next_of_kin.push(NextOfKin {
                        person_id: person.id,
                        relative: Relative::Spouse
                    });
                    person.next_of_kin.push(NextOfKin {
                        person_id: spouse.id,
                        relative: Relative::Spouse
                    });
                    self.world.events.push(event_date, WorldEventEnum::Marriage(MarriageEvent { person1_id: person.id, person2_id: spouse.id }));
                    new_people.push(spouse.clone());
                    continue;
                }

                if age > 18.0 && person.spouse().is_some() {
                    let spouse = self.world.people.get_mut(person.spouse().unwrap()).unwrap();
                    let couple_fertility = species.fertility.male_drop.powf(age - 18.) * species.fertility.female_drop.powf(age - 18.);

                    if self.rng.rand_chance(couple_fertility) {
                        let id = self.next_person_id.next();
                        let child = self.create_child(id, year, &person, &spouse);
                        self.world.events.push(event_date, WorldEventEnum::PersonBorn(SimplePersonEvent { person_id: child.id }));
                        person.next_of_kin.push(NextOfKin { 
                            person_id: child.id,
                            relative: Relative::Child
                        });
                        new_people.push(child);
                        continue;
                    }
                }

                if let Some(civ) = &mut person.civ {
                    if age > 18.0 && civ.leader_of_settlement.is_none() && self.rng.rand_chance(1.0/50.0) {
                        self.rng.next();
                        let culture = self.world.cultures.get(&civ.culture).unwrap();
                        let settlement = generate_settlement(&self.rng, year, person_id.clone(), culture, civ.faction, &self.world, &self.world.map, &self.parameters.regions).clone();
                        if let Some(settlement) = settlement {
                            let position = settlement.xy;
                            let id = self.world.settlements.insert(settlement);
                            self.world.events.push(event_date, WorldEventEnum::SettlementFounded(SettlementFoundedEvent { settlement_id: id, founder_id: *person_id }));
                            let mut faction = self.world.factions.get_mut(&civ.faction);
                            faction.settlements.insert(id);
                            civ.leader_of_settlement = Some(id);
                            if let Some(spouse) = person.spouse() {
                                let mut spouse = self.world.people.get_mut(spouse).unwrap();
                                let spouse = spouse.borrow_mut();
                                (*spouse).position = position.to_coord();
                            }
                            person.position = position.to_coord();
                            continue;
                        }
                    }
                }
            }
        }

        if self.rng.rand_chance(self.parameters.great_beasts_yearly_spawn_chance) {
            self.spawn_great_beast(year)
        }

        for (faction_id, faction) in self.world.factions.iter() {
            let mut faction = faction.borrow_mut();

            let current_enemy = faction.relations.iter().find(|kv| *kv.1 < -0.8);

            if let Some(current_enemy) = current_enemy {
                let chance_for_peace = 0.05;
                if self.rng.rand_chance(chance_for_peace) {
                    let other_faction_id = current_enemy.0.clone();
                    let mut other_faction = self.world.factions.get_mut(&other_faction_id);

                    faction.relations.insert(other_faction_id, -0.2);
                    other_faction.relations.insert(faction_id, -0.2);

                    self.world.events.push(event_date, WorldEventEnum::PeaceDeclared(PeaceDeclaredEvent { faction1_id: faction_id, faction2_id: other_faction_id }));
                }
            } else {
                for (other_faction_id, other_faction) in self.world.factions.iter() {
                    if other_faction_id == faction_id {
                        continue
                    }
                    let opinion = faction.relations.get(&other_faction_id).unwrap_or(&0.0);
                    let chance_for_war = (*opinion * -1.0).max(0.0) * 0.001 + 0.001;
                    if self.rng.rand_chance(chance_for_war) {
                        let mut other_faction = other_faction.borrow_mut();

                        faction.relations.insert(other_faction_id, -1.0);
                        other_faction.relations.insert(faction_id, -1.0);

                        self.world.events.push(event_date, WorldEventEnum::WarDeclared(WarDeclaredEvent { faction1_id: faction_id, faction2_id: other_faction_id }));

                        break
                    }
                }
            }
        }
        
        for (id, settlement) in self.world.settlements.iter() {
            let mut settlement = settlement.borrow_mut();
            if settlement.demographics.population <= 0 {
                continue
            }

            let leader = self.world.people.get(&settlement.leader_id).unwrap();
            if !leader.alive() {

                if let Some(civ) = &leader.civ {

                    let heirs_by_order = leader.sorted_heirs();
                
                    let mut valid_heir = false;
                    for heir in heirs_by_order {
                        let heir = self.world.people.get_mut(&heir.person_id);
                        if let Some(mut heir) = heir {
                            if heir.alive() {
                                if let Some(civ2) = &mut heir.civ {
                                    civ2.leader_of_settlement = Some(id);
                                    // TODO:
                                    // if civ.faction_relation == FactionRelation::Leader {
                                    //     civ2.faction_relation = FactionRelation::Leader;
                                    // }
                                    // let mut faction = self.world.factions.get_mut(&civ2.faction);
                                    // faction.leader = heir.id;
                                    valid_heir = true;
                                }
                                if valid_heir {
                                    heir.importance = Importance::Important;
                                    heir.position = settlement.xy.to_coord();
                                    settlement.leader_id = heir.id;
                                    self.world.events.push(event_date, WorldEventEnum::NewSettlementLeader(NewSettlementLeaderEvent { new_leader_id: heir.id, settlement_id: id }));
                                    break;
                                }
                            }
                        }
                    }
                    if !valid_heir {
                        self.rng.next();
                        let species = self.world.species.get(&leader.species).unwrap();
                        let new_leader = Person::new(self.next_person_id.next(), &species, Importance::Important, year, settlement.xy.to_coord())
                            .civilization(&Some(civ.clone()));
                        let mut new_leader = self.name_person(new_leader, &None);
                        if let Some(civ2) = &mut new_leader.civ {
                            civ2.leader_of_settlement = Some(id);

                            // TODO:
                            // if civ.faction_relation == FactionRelation::Leader {
                            //     civ2.faction_relation = FactionRelation::Leader;
                            // }
                        }
                        settlement.leader_id = new_leader.id;
                        self.world.events.push(event_date, WorldEventEnum::NewSettlementLeader(NewSettlementLeaderEvent { new_leader_id: new_leader.id, settlement_id: id }));
                        new_people.push(new_leader);
                    }
                }
            }

            let settlement_tile = self.world.map.tile(settlement.xy.0, settlement.xy.1);

            // https://en.wikipedia.org/wiki/Estimates_of_historical_world_population
            let soil_fertility = settlement_tile.soil_fertility;
            let growth = self.rng.randf_range(-0.005, 0.03) + ((soil_fertility - 0.5) * 0.01);
            let child_chance = (settlement.demographics.population as f32) * growth;
            if child_chance < 0.0 {
                if child_chance > -1.0 && self.rng.rand_chance(child_chance.abs()) {
                    settlement.demographics.change_population(-1);
                } else {
                    settlement.demographics.change_population(child_chance as i32);
                }
            } else {
                if child_chance < 1.0 && self.rng.rand_chance(child_chance) {
                    settlement.demographics.population = settlement.demographics.population + 1;
                } else {
                    settlement.demographics.change_population(child_chance as i32);
                }
            }

            // Keeping an army unit posted costs 100 gold per year, for reference
            let tile_gold_range = self.parameters.regions.get(settlement_tile.region_id as usize).unwrap().gold_generation_range;
            let gold_generated = self.rng.randf_range(tile_gold_range.0, tile_gold_range.1) * settlement.demographics.population as f32;
            settlement.gold = settlement.gold + gold_generated as i32;

            // Pay current army
            let army_cost = (settlement.military.trained_soldiers * 100) + (settlement.military.conscripts * 50);
            settlement.gold = (settlement.gold - army_cost as i32).max(0);

            let army_size = settlement.military.trained_soldiers + settlement.military.conscripts;
            let army_ratio = army_size as f32 / settlement.demographics.population as f32;
            if army_ratio < 0.05 {
                let can_train = settlement.gold / 50;
                settlement.military.trained_soldiers = settlement.military.trained_soldiers + can_train  as u32;
                settlement.gold = settlement.gold - (50 * can_train);
            }
            let faction_id = settlement.faction_id;
            let mut faction = self.world.factions.get_mut(&faction_id);
            let at_war = faction.relations.iter().find(|v| *v.1 <= -0.8);
            if let Some(enemy) = at_war {
                if army_ratio < 0.05 {
                    let can_train = settlement.gold / 15;
                    settlement.military.conscripts = settlement.military.conscripts + can_train as u32;
                    settlement.gold = settlement.gold - (15 * can_train);
                }
                let siege_power = settlement.military_siege_power();
                let mut attack = None;
                if siege_power > 0.0 {
                    let enemy_faction = self.world.factions.get(enemy.0);
                    for enemy_settlement_id in enemy_faction.settlements.iter() {
                        let enemy_settlement = self.world.settlements.get_mut(enemy_settlement_id);
                        let defence_power = enemy_settlement.military_defence_power();
                        let power_diff = siege_power / (siege_power + defence_power);
                        let attack_chance = power_diff.powi(2);
                        if self.rng.rand_chance(attack_chance) {
                            attack = Some((enemy_settlement_id.clone(), enemy_settlement));
                        }
                    }
                }

                if let Some(enemy_settlement) = attack {
                    let battle_modifer = self.rng.randf();
                    let (enemy_settlement_id, mut enemy_settlement) = enemy_settlement;

                    let defence_power = enemy_settlement.military_defence_power();
                    let power_diff = siege_power / (siege_power + defence_power);

                    let battle_closeness = 1.0 - (battle_modifer - power_diff).abs();

                    let battle_result = BattleResult_old {
                        attacker_deaths: ((settlement.military.trained_soldiers + settlement.military.conscripts) as f32 * battle_closeness) as u32,
                        defender_deaths: ((enemy_settlement.military.trained_soldiers + enemy_settlement.military.conscripts) as f32 * battle_closeness) as u32,
                        attacker_victor: battle_modifer > power_diff,
                        defender_captured: battle_modifer > power_diff,
                    };

                    settlement.kill_military(battle_result.attacker_deaths, &self.rng);
                    enemy_settlement.kill_military(battle_result.defender_deaths, &self.rng);

                    let enemy_faction_id = *enemy.0;
                    let mut enemy_faction = self.world.factions.get_mut(&enemy_faction_id);

                    if battle_result.defender_captured {
                        enemy_settlement.faction_id = settlement.faction_id;
                        faction.settlements.insert(enemy_settlement_id);
                        enemy_faction.settlements.remove(&enemy_settlement_id);
                    }

                    self.world.events.push(event_date, WorldEventEnum::Siege(SiegeEvent { faction1_id: faction_id, faction2_id: enemy_faction_id, settlement1_id: id.clone(), settlement2_id: enemy_settlement_id.clone(), battle_result }));
                }

            }

        }


        for new_person in new_people {
            self.world.people.insert(new_person);
        }
    }

    fn create_child(&self, id: Id, birth: u32, father: &Person, mother: &Person) -> Person {
        let species = self.world.species.get(&father.species).unwrap();
        let mut figure = Person::new(id, species, father.importance.lower(), birth, mother.position);
        if let Some(civ) = &father.civ {
            figure.civ = Some(CivilizedComponent {
                culture: civ.culture,
                faction: civ.faction,
                faction_relation: FactionRelation::Member,
                leader_of_settlement: None
            });
            figure = self.name_person(figure, &father.last_name);
        }
        figure.next_of_kin.push(NextOfKin { 
            person_id: father.id,
            relative: Relative::Parent
        });
        figure.next_of_kin.push(NextOfKin { 
            person_id: mother.id,
            relative: Relative::Parent
        });
        return figure        
    }

    fn name_person(&self, mut figure: Person, surname: &Option<String>) -> Person {
        if let Some(civ) = &figure.civ {
            let culture = self.world.cultures.get(&civ.culture).unwrap();
            let first_name;
            match figure.sex {
                PersonSex::Male => first_name = culture.first_name_male_model.generate(&self.rng.derive("first_name"), 4, 15),
                PersonSex::Female => first_name = culture.first_name_female_model.generate(&self.rng.derive("first_name"), 4, 15)
            }
            let first_name = Strings::capitalize(&first_name);
            let last_name;
            match surname {
                Some(str) => last_name = String::from(str),
                None => last_name = Strings::capitalize(&culture.last_name_model.generate(&self.rng.derive("last_name"), 4, 15))
            }
            figure.first_name = Some(first_name);
            figure.birth_last_name = Some(last_name.clone());
            figure.last_name = Some(last_name.clone());
        }
        return figure
    }

    fn spawn_great_beast(&mut self, year: u32) {
        let mut species = Id(2); // Fiend
        if self.rng.rand_chance(0.3) {
            species = Id(1); // Leshen
        }
        let species = self.world.species.get(&species).unwrap();
        let mut suitable_location = None;
        'candidates: for _ in 1..10 {
            let txy = Coord2::xy(self.rng.randu_range(0, self.world.map.size.x()) as i32, self.rng.randu_range(0, self.world.map.size.y()) as i32);
            let tile = self.world.map.tile(txy.x as usize, txy.y as usize);
            if tile.region_id == 0 {// Ocean
                continue;
            }
            for (_, settlement) in self.world.settlements.iter() {
                if settlement.borrow().xy.to_coord().dist_squared(&txy) < 3.0_f32.powi(2) {
                    continue 'candidates;
                }
            }
            suitable_location = Some(txy);
            break;
        }
        if let Some(xy) = suitable_location {
            let id = self.next_person_id.next();
            self.world.people.insert(Person::new(id, species, Importance::Important, year, xy));
            println!("born {} {}", year, id.0);
            self.world.events.push(WorldEventDate { year }, WorldEventEnum::PersonBorn(SimplePersonEvent { person_id: id }))
        }
    }

    fn beast_hunt_nearby(&self, beast: &mut Person) -> Option<(BattleResult, BattleResult)> {
        let mut rng = self.rng.derive("beast_attack");
        let xy = beast.position + Coord2::xy(rng.randi_range(-15, 15), rng.randi_range(-15, 15));
        if let Some((sett_id, settlement)) = self.world.settlements.iter().find(|(_, sett)| sett.borrow().xy.to_coord() == xy) {
            let mut creature_force = BattleForce::from_creatures(&self.world, vec!(beast));
            let mut settlement_corce = BattleForce::from_defending_settlement(&self.world, sett_id, &settlement.borrow());
            let result = creature_force.battle(&mut settlement_corce, &mut rng, settlement.borrow().xy.to_coord(), sett_id);
            return Some(result)
        }
        None
    }

}

fn generate_settlement(rng: &Rng, founding_year: u32, leader: Id, culture: &Culture, faction: Id, world_graph: &World, world_map: &WorldTopology, regions: &Vec<Region>) -> Option<Settlement> {
    let mut rng = rng.derive("settlement");
    let mut xy = None;
    'candidates: for _ in 1..10 {
        let txy = Point2D(rng.randu_range(0, world_map.size.x()), rng.randu_range(0, world_map.size.y()));
        let tile = world_graph.map.tile(txy.0, txy.1);
        if tile.region_id == 0 {// Ocean
            continue;
        }
        for (_, settlement) in world_graph.settlements.iter() {
            if settlement.borrow().xy.dist_squared(&txy) < 3.0_f32.powi(2) {
                continue 'candidates;
            }
        }
        xy = Some(txy);
        break;
    }
    if let Some(xy) = xy {
        let region_id = world_map.tile(xy.0, xy.1).region_id as usize;
        let region = regions.get(region_id).unwrap();

        return Some(SettlementBuilder::colony(&rng, xy, founding_year, leader, culture, faction, region).create())
    } else {
        None
    }
}