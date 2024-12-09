use std::{borrow::BorrowMut, collections::HashMap, time::Instant};

use graphics::rectangle::{square, Border};
use piston::{Button, Key};

use crate::{commons::{history_vec::{HistoryVec, Id}, rng::Rng, strings::Strings}, engine::{geometry::{Coord2, Size2D}, render::RenderContext, scene::Scene, Color, Point2D}, game::InputEvent, world::{faction::{Faction, FactionRelation}, person::{Importance, NextOfKin, Person, PersonSex, Relative}, topology::{WorldTopology, WorldTopologyGenerationParameters}, world::People}, BattleResult, MarriageEvent, NewSettlementLeaderEvent, PeaceDeclaredEvent, SettlementFoundedEvent, SiegeEvent, SimplePersonEvent, WarDeclaredEvent, WorldEventDate, WorldEventEnum, WorldEvents};

use super::{culture::Culture, person::CivilizedComponent, region::Region, settlement::{Settlement, SettlementBuilder}, species::{Species, SpeciesIntelligence}, world::World};

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
        map.insert(Id(1), Species::new(Id(1), "leshen").intelligence(SpeciesIntelligence::Instinctive).lifetime(200).fertility(0.));
        map
    }

    pub fn simulate_year(&mut self) {
        self.year = self.year + 1;
        let year = self.year;
        let event_date = WorldEventDate { year };
        println!("Year {}, {} people to process", self.year, self.world.people.len());

        let mut new_people: Vec<Person> = Vec::new();

        for (person_id, person) in self.world.people.iter() {
            let mut person = person.borrow_mut();
            let species = self.world.species.get(&person.species).unwrap();
            let age = (year - person.birth) as f32;
            if self.rng.rand_chance(f32::min(1.0, (age / species.lifetime.max_age as f32).powf(5.0))) {
                person.death = year;
                self.world.events.push(event_date, WorldEventEnum::PersonDeath(SimplePersonEvent { person_id: person.id }));
                if let Some(civ) = &person.civ {
                    if let Some(settlement_id) = civ.leader_of_settlement {
                        let heirs_by_order = person.sorted_heirs();
                        let settlement = self.world.settlements.get(&settlement_id);
                    
                        let mut valid_heir = false;
                        for heir in heirs_by_order {
                            let mut heir = self.world.people.get_mut(&heir.person_id).unwrap();
                            if heir.alive() {
                                if let Some(civ2) = &mut heir.civ {
                                    civ2.leader_of_settlement = Some(settlement_id);
                                    if civ.faction_relation == FactionRelation::Leader {
                                        civ2.faction_relation = FactionRelation::Leader;
                                    }
                                    let mut faction = self.world.factions.get_mut(&civ2.faction);
                                    faction.leader = heir.id;
                                    valid_heir = true;
                                }
                                if valid_heir {
                                    heir.importance = Importance::Important;
                                    heir.position = settlement.xy.to_coord();
                                    self.world.events.push(event_date, WorldEventEnum::NewSettlementLeader(NewSettlementLeaderEvent { new_leader_id: heir.id, settlement_id }));
                                    break;
                                }
                            }
                        }
                        if !valid_heir {
                            self.rng.next();
                            let species = self.world.species.get(&person.species).unwrap();
                            let new_leader = Person::new(self.next_person_id.next(), &species, Importance::Important, year, settlement.xy.to_coord())
                                .civilization(&Some(civ.clone()));
                            let mut new_leader = self.name_person(new_leader, &None);
                            if let Some(civ2) = &mut new_leader.civ {
                                civ2.leader_of_settlement = Some(settlement_id);
                                if civ.faction_relation == FactionRelation::Leader {
                                    civ2.faction_relation = FactionRelation::Leader;
                                }
                            }
                            self.world.events.push(event_date, WorldEventEnum::NewSettlementLeader(NewSettlementLeaderEvent { new_leader_id: new_leader.id, settlement_id }));
                            new_people.push(new_leader);
                        }
                    }
                }

                continue
            }

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
                    let settlement = generate_settlement(&self.rng, year, culture, civ.faction, &self.world, &self.world.map, &self.parameters.regions).clone();
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

            if let Some(civ) = &person.civ {
                if civ.faction_relation == FactionRelation::Leader {
                    let faction_id = civ.faction;
                    let mut faction = self.world.factions.get_mut(&faction_id);

                    if faction_id != civ.faction {
                        panic!("{:?} {:?}", faction_id, civ.faction);
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