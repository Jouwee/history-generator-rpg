use std::{borrow::BorrowMut, collections::HashMap, time::Instant};

use graphics::rectangle::{square, Border};
use piston::{Button, Key};

use crate::{commons::{history_vec::{HistoryVec, Id}, rng::Rng, strings::Strings}, engine::{geometry::Size2D, render::RenderContext, scene::Scene, Color, Point2D}, game::InputEvent, world::{faction::{Faction, FactionRelation}, person::{Importance, NextOfKin, Person, PersonSex, Relative}, topology::{WorldTopology, WorldTopologyGenerationParameters}, world::People}, BattleResult, MarriageEvent, NewSettlementLeaderEvent, PeaceDeclaredEvent, SettlementFoundedEvent, SiegeEvent, SimplePersonEvent, WarDeclaredEvent, WorldEventDate, WorldEventEnum, WorldEvents};

use super::{culture::Culture, region::Region, settlement::{Settlement, SettlementBuilder}, world::World};

pub struct WorldGenScene {
    generator: WorldHistoryGenerator,
    view: WorldViewMode
}

impl WorldGenScene {
    pub fn new(params: WorldGenerationParameters) -> WorldGenScene {
        WorldGenScene {
            generator: WorldHistoryGenerator::seed_world(params),
            view: WorldViewMode::Normal
        }
    }

    pub fn into_world(self) -> World {
        return self.generator.world
    }
}

impl Scene for WorldGenScene {
    fn render(&self, ctx: RenderContext) {
        use graphics::*;

        // https://lospec.com/palette-list/31
        let gray = Color::from_hex("636663");
        // let XXX = Color::from_hex("87857c");
        // let XXX = Color::from_hex("bcad9f");
        let salmon = Color::from_hex("f2b888");
        let orange = Color::from_hex("eb9661");
        let red = Color::from_hex("b55945");
        // let XXX = Color::from_hex("734c44");
        // let XXX = Color::from_hex("3d3333");
        let wine = Color::from_hex("593e47");
        // let XXX = Color::from_hex("7a5859");
        // let XXX: Color = Color::from_hex("a57855");
        let yellow = Color::from_hex("de9f47");
        // let XXX = Color::from_hex("fdd179");
        let off_white = Color::from_hex("fee1b8");
        // let XXX = Color::from_hex("d4c692");
        // let XXX = Color::from_hex("a6b04f");
        let yellow_green = Color::from_hex("819447");
        // let XXX = Color::from_hex("44702d");
        let dark_green = Color::from_hex("2f4d2f");
        // let XXX = Color::from_hex("546756");
        // let XXX = Color::from_hex("89a477");
        // let XXX = Color::from_hex("a4c5af");
        let teal = Color::from_hex("cae6d9");
        let white = Color::from_hex("f1f6f0");
        // let XXX = Color::from_hex("d5d6db");
        // let XXX = Color::from_hex("bbc3d0");
        // let XXX = Color::from_hex("96a9c1");
        // let XXX = Color::from_hex("6c81a1");
        let blue = Color::from_hex("405273");
        // let XXX = Color::from_hex("303843");
        let black = Color::from_hex("14233a");

        let faction_colors = [red, black, blue, teal, yellow, yellow_green, wine, white, orange, gray];

        let world = &self.generator.world;

        let ts = 4.;
        if self.view == WorldViewMode::Normal {


            for x in 0..world.map.size.x() {
                for y in 0..world.map.size.y() {
                    let tile = world.map.tile(x, y);

                    let color;
                    match tile.region_id {
                        0 => color = blue,
                        1 => color = off_white,
                        2 => color = dark_green,
                        3 => color = salmon,
                        _ => color = black
                    }
                    rectangle(color.f32_arr(), rectangle::square(x as f64 * ts, y as f64 * ts, ts), ctx.context.transform, ctx.gl);

                    let mut height_diff = 0.0;
                    let mut height_count = 0;
                    if x > 0 {
                        height_diff += tile.elevation as f32 - world.map.tile(x - 1, y).elevation as f32;
                        height_count += 1;
                    }
                    if y > 0 {
                        height_diff += tile.elevation as f32 - world.map.tile(x, y - 1).elevation as f32;
                        height_count += 1;
                    }
                    if x < world.map.size.x() - 1 {
                        height_diff += world.map.tile(x + 1, y).elevation as f32 - tile.elevation as f32;
                        height_count += 1;
                    }
                    if y < world.map.size.y() - 1 {
                        height_diff += world.map.tile(x, y + 1).elevation as f32 - tile.elevation as f32;
                        height_count += 1;
                    }
                    height_diff = (height_diff / height_count as f32) / 256.0;
                    if height_diff < 0.0 {
                        let opacity = height_diff.abs();
                        rectangle(black.alpha(opacity).f32_arr(), rectangle::square(x as f64 * ts, y as f64 * ts, ts), ctx.context.transform, ctx.gl);
                    } else {
                        let opacity = height_diff;
                        rectangle(white.alpha(opacity).f32_arr(), rectangle::square(x as f64 * ts, y as f64 * ts, ts), ctx.context.transform, ctx.gl);
                    }

                }   
            }

            for (_, settlement) in world.settlements.iter() {
                let settlement = settlement.borrow();

                if settlement.demographics.population > 0 {
                    let color = faction_colors[settlement.faction_id.seq() % faction_colors.len()];
                    let mut transparent = color.f32_arr();
                    transparent[3] = 0.4;

                    let mut rectangle = Rectangle::new(transparent);
                    rectangle = rectangle.border(Border { color: color.f32_arr(), radius: 1.0 });
                    let dims = square(settlement.xy.0 as f64 * ts, settlement.xy.1 as f64 * ts, ts);
                    rectangle.draw(dims, &DrawState::default(), ctx.context.transform, ctx.gl);
                }

            }
        } else {
            for x in 0..world.map.size.x() {
                for y in 0..world.map.size.y() {
                    let tile = world.map.tile(x, y);
                    let mut color = white;
                    match self.view {
                        WorldViewMode::Normal => (), // Already checked
                        WorldViewMode::Elevation => {
                            color = white.alpha((tile.elevation as f32) / 256.0);
                        },
                        WorldViewMode::Precipitation => {
                            color = blue.alpha((tile.precipitation as f32) / 256.0);
                        }
                    }
                    rectangle(color.f32_arr(), rectangle::square(x as f64 * ts, y as f64 * ts, ts), ctx.context.transform, ctx.gl);
                }   
            }
        }

    }

    fn update(&mut self) {
        if self.generator.year < 500 {
            self.generator.simulate_year();
        }
    }

    fn input(&mut self, evt: &InputEvent) {
        match evt.button_args.button {
            Button::Keyboard(Key::V) => {
                match self.view {
                    WorldViewMode::Normal => self.view = WorldViewMode::Elevation,
                    WorldViewMode::Elevation => self.view = WorldViewMode::Precipitation,
                    WorldViewMode::Precipitation => self.view = WorldViewMode::Normal,
                }
            }
            _ => ()
        }
    }
}


#[derive(PartialEq)]
enum WorldViewMode {
    Normal,
    Elevation,
    Precipitation,
}

pub struct WorldGenerationParameters {
    pub seed: u32,
    pub cultures: Vec<Culture>,
    pub regions: Vec<Region>
}

struct WorldHistoryGenerator {
    rng: Rng,
    year: u32,
    parameters: WorldGenerationParameters,
    pub world: World,
    next_person_id: Id,
}

impl WorldHistoryGenerator {

    pub fn seed_world(parameters: WorldGenerationParameters) -> WorldHistoryGenerator {
        let mut rng = Rng::seeded(parameters.seed);
       
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

        // Generate starter people
        for _ in 0..10 {
            rng.next();
            let id = person_id.next();
            let culture = world.cultures.get(&Id(rng.randu_range(0, culture_id.seq()))).unwrap();
            let sex;
            if rng.rand_chance(0.5) {
                sex = PersonSex::Male;
            } else {
                sex = PersonSex::Female;
            }
            let faction = Faction::new(&rng, id);
            let faction_id = world.factions.insert(faction);
            // TODO: Position
            let mut person = generate_person(&rng, Importance::Important, id, 1, sex, Point2D(0, 0), &culture, &faction_id, None);
            person.faction_relation = FactionRelation::Leader;
            world.events.push(event_date, WorldEventEnum::PersonBorn(SimplePersonEvent { person_id: person.id }));
            world.people.insert(person);
        }
        return WorldHistoryGenerator {
            rng,
            parameters,
            world,
            year: 1,
            next_person_id: person_id
        };
    }

    pub fn simulate_year(&mut self) {
        self.year = self.year + 1;
        let year = self.year;
        let event_date = WorldEventDate { year };
        println!("Year {}, {} people to process", self.year, self.world.people.len());

        let mut new_people: Vec<Person> = Vec::new();

        for (_, person) in self.world.people.iter() {
            let mut person = person.borrow_mut();
            let age = (year - person.birth) as f32;
            if self.rng.rand_chance(f32::min(1.0, (age/120.0).powf(5.0))) {
                person.death = year;
                self.world.events.push(event_date, WorldEventEnum::PersonDeath(SimplePersonEvent { person_id: person.id }));
                if let Some(settlement_id) = person.leader_of_settlement {
                    let heirs_by_order = person.sorted_heirs();
                    let settlement = self.world.settlements.get(&settlement_id);
                
                    let mut valid_heir = false;
                    for heir in heirs_by_order {
                        let mut heir = self.world.people.get_mut(&heir.person_id).unwrap();
                        if heir.alive() {
                            heir.leader_of_settlement = Some(settlement_id);
                            heir.importance = Importance::Important;
                            if person.faction_relation == FactionRelation::Leader {
                                heir.faction_relation = FactionRelation::Leader;
                            }
                            heir.position = settlement.xy;
                            self.world.events.push(event_date, WorldEventEnum::NewSettlementLeader(NewSettlementLeaderEvent { new_leader_id: heir.id, settlement_id }));
                            let mut faction = self.world.factions.get_mut(&heir.faction_id);
                            faction.leader = heir.id;
                            valid_heir = true;
                            break
                        }
                    }
                    if !valid_heir {
                        let culture = self.world.cultures.get(&Id(self.rng.randu_range(0, self.world.cultures.len()))).unwrap();
                        let sex;
                        if self.rng.rand_chance(0.5) {
                            sex = PersonSex::Male;
                        } else {
                            sex = PersonSex::Female;
                        }
                        self.rng.next();
                        let mut new_leader = generate_person(&self.rng, Importance::Important, self.next_person_id.next(), year, sex, settlement.xy, &culture, &person.faction_id, None);
                        new_leader.leader_of_settlement = Some(settlement_id);
                        if person.faction_relation == FactionRelation::Leader {
                            new_leader.faction_relation = FactionRelation::Leader;
                        }
                        self.world.events.push(event_date, WorldEventEnum::NewSettlementLeader(NewSettlementLeaderEvent { new_leader_id: new_leader.id, settlement_id }));
                        new_people.push(new_leader);
                    }
                }

                continue
            }

            if age > 18.0 && person.spouse().is_none() && self.rng.rand_chance(0.1) {
                self.rng.next();
                let id = self.next_person_id.next();
                let spouse_age = self.rng.randu_range(18, age as usize + 10) as u32;
                let spouse_birth_year = year - u32::min(spouse_age, year);
                let culture = self.world.cultures.get(&person.culture_id).unwrap();
                let mut spouse = generate_person(&self.rng, person.importance.lower(), id, spouse_birth_year, person.sex.opposite(), person.position, culture, &person.faction_id, None);
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
                let couple_fertility = person.fertility(year) * spouse.fertility(year);

                if self.rng.rand_chance(couple_fertility * 0.5) {
                    let id = self.next_person_id.next();
                    self.rng.next();
                    let sex;
                    if self.rng.rand_chance(0.5) {
                        sex = PersonSex::Male;
                    } else {
                        sex = PersonSex::Female;
                    }
                    let culture = self.world.cultures.get(&person.culture_id).unwrap();
                    let mut child = generate_person(&self.rng, person.importance.lower(), id, year, sex, person.position, culture, &person.faction_id, Some(&person.last_name));
                    child.next_of_kin.push(NextOfKin { 
                        person_id: person.id,
                        relative: Relative::Parent
                    });
                    self.world.events.push(event_date, WorldEventEnum::PersonBorn(SimplePersonEvent { person_id: child.id }));
                    person.next_of_kin.push(NextOfKin { 
                        person_id: child.id,
                        relative: Relative::Child
                    });
                    new_people.push(child);
                    continue;
                }
            }

            if age > 18.0 && person.leader_of_settlement.is_none() && self.rng.rand_chance(1.0/50.0) {
                self.rng.next();
                let culture = self.world.cultures.get(&person.culture_id).unwrap();
                let settlement = generate_settlement(&self.rng, year, culture, person.faction_id, &self.world, &self.world.map, &self.parameters.regions).clone();
                if let Some(settlement) = settlement {
                    let position = settlement.xy;
                    let id = self.world.settlements.insert(settlement);
                    self.world.events.push(event_date, WorldEventEnum::SettlementFounded(SettlementFoundedEvent { settlement_id: id, founder_id: person.id }));
                    let mut faction = self.world.factions.get_mut(&person.faction_id);
                    faction.settlements.insert(id);
                    person.leader_of_settlement = Some(id);
                    if let Some(spouse) = person.spouse() {
                        let mut spouse = self.world.people.get_mut(spouse).unwrap();
                        let spouse = spouse.borrow_mut();
                        (*spouse).position = position;
                    }
                    person.position = position;
                    continue;
                }
            }

            if person.faction_relation == FactionRelation::Leader {
                let faction_id = person.faction_id;
                let mut faction = self.world.factions.get_mut(&faction_id);

                if faction_id != person.faction_id {
                    panic!("{:?} {:?}", faction_id, person.faction_id);
                }

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

        }


        for new_person in new_people {
            self.world.people.insert(new_person);
        }
        
        for (id, settlement) in self.world.settlements.iter() {
            let mut settlement = settlement.borrow_mut();
            if settlement.demographics.population <= 0 {
                continue
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

                    let battle_result = BattleResult {
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
    }

}

fn generate_person(rng: &Rng, importance: Importance, next_id: Id, birth_year: u32, sex: PersonSex, position: Point2D, culture: &Culture, faction: &Id, surname: Option<&str>) -> Person {
    let rng = rng.derive("person");
    let first_name;
    match sex {
        PersonSex::Male => first_name = culture.first_name_male_model.generate(&rng.derive("first_name"), 4, 15),
        PersonSex::Female => first_name = culture.first_name_female_model.generate(&rng.derive("first_name"), 4, 15)
    }
    let first_name = Strings::capitalize(&first_name);
    let last_name;
    match surname {
        Some(str) => last_name = String::from(str),
        None => last_name = Strings::capitalize(&culture.last_name_model.generate(&rng.derive("last_name"), 4, 15))
    }
    return Person {
        id: next_id,
        importance,
        position,
        first_name,
        last_name: last_name.clone(),
        birth_last_name: last_name.clone(),
        birth: birth_year,
        sex,
        culture_id: culture.id,
        faction_id: faction.clone(),
        faction_relation: FactionRelation::Member,
        death: 0,
        next_of_kin: Vec::new(),
        leader_of_settlement: None
    }
}


fn generate_settlement(rng: &Rng, founding_year: u32, culture: &Culture, faction: Id, world_graph: &World, world_map: &WorldTopology, regions: &Vec<Region>) -> Option<Settlement> {
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

        return Some(SettlementBuilder::colony(&rng, xy, founding_year, culture, faction, region).create())
    } else {
        None
    }
}