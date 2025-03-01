use std::{borrow::BorrowMut, cell::RefMut, collections::HashMap, time::Instant};

use crate::{commons::{history_vec::{HistoryVec, Id}, id_vec::IdVec, resource_map::ResourceMap, rng::Rng, strings::Strings}, engine::{geometry::{Coord2, Size2D}, Point2D}, resources::resources::Resources, world::{faction::{Faction, FactionRelation}, item::{Mace, Sword}, person::{Importance, NextOfKin, Person, PersonSex, Relative}, topology::{WorldTopology, WorldTopologyGenerationParameters}, world::People}, ArtifactPossesionEvent, CauseOfDeath, MarriageEvent, NewSettlementLeaderEvent, PeaceDeclaredEvent, SettlementFoundedEvent, SimplePersonEvent, WarDeclaredEvent, WorldEventDate, WorldEventEnum, WorldEvents};

use super::{attributes::Attributes, battle_simulator::{BattleForce, BattleResult}, culture::Culture, item::{Item, ItemQuality}, material::MaterialId, person::CivilizedComponent, region::Region, settlement::{Settlement, SettlementBuilder}, species::{Species, SpeciesApearance, SpeciesIntelligence}, world::{ArtifactId, SpeciesId, World}};


pub struct WorldGenerationParameters {
    pub seed: u32,
    pub cultures: Vec<Culture>,
    pub regions: Vec<Region>,
    pub great_beasts_yearly_spawn_chance: f32,
    pub legendary_artifact_comission_chance: f32
}

pub struct WorldHistoryGenerator {
    rng: Rng,
    pub year: u32,
    parameters: WorldGenerationParameters,
    pub world: World,
    next_person_id: Id,
    resources: Resources
}

impl WorldHistoryGenerator {

    pub fn seed_world(parameters: WorldGenerationParameters, resources: &Resources) -> WorldHistoryGenerator {
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

        let mut regions = HashMap::new();
        for region in parameters.regions.iter() {
            regions.insert(Id(region.id), region.clone());
        }

        let mut world = World {
            map: world_map,
            cultures: HashMap::new(),
            species: Self::load_species(resources),
            regions,
            artifacts: IdVec::new(),
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
            resources: resources.clone(),
            world,
            year: 1,
            next_person_id: person_id
        };

        // Generate starter people
        for _ in 0..16 {
            generator.rng.next();
            let id = person_id.next();
            let culture = generator.world.cultures.get(&Id(generator.rng.randu_range(0, culture_id.seq()))).unwrap();
            let faction = Faction::new(&generator.rng, id);
            let faction_id = generator.world.factions.insert(faction);
            // TODO: Position
            let position = Coord2::xy(generator.rng.randu_range(0, generator.world.map.size.x()) as i32, generator.rng.randu_range(0, generator.world.map.size.y()) as i32);
            let species = generator.world.species.id_of("species:human");
            let person = Person::new(id, &species, Importance::Important, 1, position)
                .civilization(&Some(CivilizedComponent {
                    culture: culture.id,
                    faction: faction_id,
                    faction_relation: FactionRelation::Leader,
                    leader_of_settlement: None
                }));
            let person = generator.name_person(person, &None);
            generator.world.events.push(event_date, position, WorldEventEnum::PersonBorn(SimplePersonEvent { person_id: person.id }));
            generator.world.people.insert(person);
        }
        generator.next_person_id = person_id;
        return generator;
    }

    fn load_species(resources: &Resources) -> ResourceMap<SpeciesId, Species> {
        // TODO: Move to resources
        let mut map = ResourceMap::new();
        map.add("species:human", Species::new("human", SpeciesApearance::composite(
            vec!(
                ("base", vec!(
                    ("male_light", "species/human/base_male_light.png"),
                    ("female_light", "species/human/base_female_light.png")
                )),
                ("hair", vec!(
                    ("bun", "species/human/hair_bun.png"),
                    ("short", "species/human/hair_short.png"),
                    ("shaved", "species/human/hair_shaved.png"),
                    ("bald", "system/transparent.png"),
                )),
                ("clothes", vec!(("default", "species/human/armor_placeholder.png"))),
            )
        )));
        map.add("species:leshen", Species::new("leshen", SpeciesApearance::single_sprite("leshen.png"))
            .intelligence(SpeciesIntelligence::Instinctive)
            .attributes(Attributes { strength: 45, agility: 15, constitution: 45, unallocated: 0 })
            .lifetime(300)
            .fertility(0.)
            .drops(vec!((resources.materials.id_of("mat:bone_leshen"), 1)))
        );
        map.add("species:fiend", Species::new("fiend", SpeciesApearance::single_sprite("fiend.png"))
            .intelligence(SpeciesIntelligence::Instinctive)
            .attributes(Attributes { strength: 35, agility: 25, constitution: 35, unallocated: 0 })
            .lifetime(200)
            .fertility(0.)
            .drops(vec!((resources.materials.id_of("mat:bone_fiend"), 1)))
        );
        map.add("species:spider", Species::new("spider", SpeciesApearance::single_sprite("spider.png"))
            .intelligence(SpeciesIntelligence::Instinctive)
            .attributes(Attributes { strength: 5, agility: 12, constitution: 10, unallocated: 0 })
        );
        map
    }

    pub fn simulate_year(&mut self) {
        self.year = self.year + 1;
        let year = self.year;
        let event_date = WorldEventDate { year };

        let mut new_people: Vec<Person> = Vec::new();

        let ids: Vec<Id> = self.world.people.ids();
        for id in ids {
            let action = self.choose_person_action(id, event_date);
            match action {
                ActionToSimulate::None => {},
                ActionToSimulate::Death(id) => { let _ = self.kill_person(event_date, id, CauseOfDeath::NaturalCauses); },
                ActionToSimulate::GreatBeastHunt(id) => self.beast_hunt_nearby(event_date, &id),
                ActionToSimulate::ComissionArtifact(id) => self.create_simple_artifact(event_date, &id),
                ActionToSimulate::MarryRandomPerson(id) => self.marry_random_person(event_date, &id),
                ActionToSimulate::HaveChildWith(id_father, id_mother) => self.have_child_with(event_date, id_father, id_mother),
                ActionToSimulate::ColonizeNewSettlement(id) => self.colonize_new_settlement(event_date, id)
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
                    // TODO: Location
                    self.world.events.push(event_date, Coord2::xy(-64, -64), WorldEventEnum::PeaceDeclared(PeaceDeclaredEvent { faction1_id: faction_id, faction2_id: other_faction_id }));
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
                        // TODO: Location
                        self.world.events.push(event_date, Coord2::xy(-64, -64), WorldEventEnum::WarDeclared(WarDeclaredEvent { faction1_id: faction_id, faction2_id: other_faction_id }));

                        break
                    }
                }
            }
        }
        
        let ids = self.world.settlements.ids();
        for id in ids {
            let mut settlement = self.world.settlements.get_mut(&id);
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
                                    self.world.events.push(event_date, settlement.xy.to_coord(), WorldEventEnum::NewSettlementLeader(NewSettlementLeaderEvent { new_leader_id: heir.id, settlement_id: id }));
                                    break;
                                }
                            }
                        }
                    }
                    if !valid_heir {
                        self.rng.next();
                        let new_leader = Person::new(self.next_person_id.next(), &leader.species, Importance::Important, year, settlement.xy.to_coord())
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
                        self.world.events.push(event_date, settlement.xy.to_coord(), WorldEventEnum::NewSettlementLeader(NewSettlementLeaderEvent { new_leader_id: new_leader.id, settlement_id: id }));
                        new_people.push(new_leader);
                    }
                }
            }
            drop(leader);

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
            let faction = self.world.factions.get_mut(&faction_id);
            let at_war = faction.relations.iter().find(|v| *v.1 <= -0.8);
            let mut battle = None;
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

                if let Some((enemy_settlement_id, enemy_settlement)) = attack {
                    let mut attacker_force = BattleForce::from_attacking_settlement(&self.world, id, &settlement);
                    let mut defender_force = BattleForce::from_defending_settlement(&self.world, enemy_settlement_id, &enemy_settlement);
                    let result = attacker_force.battle(&mut defender_force, &mut self.rng.derive("battle"), enemy_settlement.xy.to_coord(), enemy_settlement_id);
                    battle = Some(result);
                }
            }
            drop(settlement);
            drop(faction);
            if let Some(result) = battle {
                self.apply_battle_result(event_date, result);
            }
        }


        for new_person in new_people {
            self.world.people.insert(new_person);
        }
    }

    fn choose_person_action(&self, id: Id, date: WorldEventDate) -> ActionToSimulate {
        let person = self.world.people.get(&id).unwrap();

        if !person.alive() {
            return ActionToSimulate::None
        }

        let mut rng = self.rng.derive(id);
        let species = self.world.species.get(&person.species);
        let age = (date.year - person.birth) as f32;

        // Random death chance
        if rng.rand_chance(f32::min(1.0, (age / species.lifetime.max_age as f32).powf(5.0))) {
            return ActionToSimulate::Death(id)
        }
        // Don't track anything else besides death for people that aren't remarkable
        if person.importance == Importance::Unknown {
            return ActionToSimulate::None
        }

        if species.lifetime.is_adult(age) {
            if species.intelligence == SpeciesIntelligence::Instinctive {
                return ActionToSimulate::GreatBeastHunt(id)
            }
            if species.intelligence == SpeciesIntelligence::Civilized {
                if person.spouse().is_none() && rng.rand_chance(0.1) {
                    return ActionToSimulate::MarryRandomPerson(id)
                }
                if person.spouse().is_some() {
                    let spouse = self.world.people.get_mut(person.spouse().unwrap()).unwrap();
                    let couple_fertility = species.fertility.male_drop.powf(age - 18.) * species.fertility.female_drop.powf(age - 18.);
                    if rng.rand_chance(couple_fertility) {
                        return ActionToSimulate::HaveChildWith(id, spouse.id)
                    }
                }
                if let Some(civ) = &person.civ {
                    match civ.leader_of_settlement {
                        Some(sett_id) => {
                            let settlement = self.world.settlements.get(&sett_id);
                            if settlement.demographics.population > 100 && rng.rand_chance(self.parameters.legendary_artifact_comission_chance) {
                                return ActionToSimulate::ComissionArtifact(id)
                            }
                        },
                        None => {
                            if rng.rand_chance(0.02) {
                                return ActionToSimulate::ColonizeNewSettlement(id)
                            }
                        }
                    }
                }
            }
        }
        return ActionToSimulate::None
    }

    fn kill_person(&mut self, date: WorldEventDate, id: Id, cause_of_death: CauseOfDeath) -> Option<MaterialId> {
        let mut person = self.world.people.get_mut(&id).unwrap();
        person.death = date.year;
        let mut artifact_material = None;
        {
            let species = self.world.species.get(&person.species);
            if species.drops.len() > 0 {
                let (drop_to_use, _) = species.drops.get(self.rng.randu_range(0, species.drops.len())).unwrap();
                artifact_material = Some(drop_to_use.clone());
            }
        }
        if person.possesions.len() > 0 {
            'possession_handling: {
                if let CauseOfDeath::KilledInBattle(killer, _weapon) = &cause_of_death {
                    if let Some(killer_id) = killer {
                        let mut killer = self.world.people.get_mut(&killer_id).unwrap();
                        let species = self.world.species.get(&killer.species);
                        // TODO: They might not be marked as dead yet
                        if killer.alive() && species.intelligence == SpeciesIntelligence::Civilized {
                            for item in person.possesions.iter() {
                                self.world.events.push(date, killer.position, WorldEventEnum::ArtifactPossession(ArtifactPossesionEvent { item: *item, person: killer.id }));
                                killer.possesions.push(item.clone());
                            }
                            killer.importance = killer.importance.at_least(&Importance::Unimportant);
                            person.possesions.clear();
                            break 'possession_handling;
                        }
                    }
                }
                let mut heirs: Vec<RefMut<Person>> = person.sorted_heirs().iter()
                    .map(|n| self.world.people.get_mut(&n.person_id).unwrap())
                    .filter(|p| p.alive())
                    .collect();
                let heir_count = heirs.len();
                if heir_count > 0 {
                    for (i, item) in person.possesions.iter().enumerate() {
                        let heir = heirs.get_mut(i % heir_count).unwrap();
                        heir.possesions.push(*item);
                        heir.importance = heir.importance.at_least(&Importance::Unimportant);
                        self.world.events.push(date, heir.position, WorldEventEnum::ArtifactPossession(ArtifactPossesionEvent { item: *item, person: heir.id }))
                    }
                    person.possesions.clear();
                }
            }
        }

        self.world.events.push(date, person.position, WorldEventEnum::PersonDeath(crate::CreatureDeathEvent { creature: id, cause_of_death }));

        return artifact_material
    }

    fn create_child(&self, id: Id, birth: u32, father: &Person, mother: &Person) -> Person {
        let mut figure = Person::new(id, &father.species, father.importance.lower(), birth, mother.position);
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

    fn create_artifact(&mut self, date: WorldEventDate, location: Coord2, material_id: &MaterialId) -> ArtifactId {
        let material_id = material_id.clone();
        let item;
        match self.rng.randu_range(0, 2) {
            0 => {
                let mut blade = self.resources.materials.id_of("mat:steel");
                let mut handle = self.resources.materials.id_of("mat:oak");
                let mut guard = self.resources.materials.id_of("mat:bronze");
                let mut pommel = self.resources.materials.id_of("mat:bronze");
                match self.rng.randu_range(0, 4) {
                    1 => blade = material_id,
                    2 => guard = material_id,
                    3 => handle = material_id,
                    _ => pommel = material_id,
                }
                let mut sword = Sword::new(ItemQuality::Legendary, handle, blade, pommel, guard, &self.resources.materials);
                sword.name = Some(self.artifact_name(self.rng.derive("name"), vec!(
                    "sword", "blade", "slash", "fang", "tongue", "kiss", "wing", "edge", "talon"
                )));
                item = Item::Sword(sword)
            },
            _ => {
                let mut head = self.resources.materials.id_of("mat:steel");
                let mut handle = self.resources.materials.id_of("mat:oak");
                let mut pommel = self.resources.materials.id_of("mat:bronze");
                match self.rng.randu_range(0, 3) {
                    1 => head = material_id,
                    2 => handle = material_id,
                    _ => pommel = material_id,
                }
                let mut mace = Mace::new(ItemQuality::Legendary, handle, head, pommel, &self.resources.materials);
                mace.name = Some(self.artifact_name(self.rng.derive("name"), vec!(
                    "breaker", "kiss", "fist", "touch"
                )));
                item = Item::Mace(mace)
            }
        }
        let id = self.world.artifacts.add(item);
        self.world.events.push(date, location, WorldEventEnum::ArtifactCreated(crate::ArtifactEvent { item: id }));
        return id
    }

    fn artifact_name(&self, mut rng: Rng, suffixes: Vec<&str>) -> String {
        let preffixes = [
            "whisper", "storm", "fire", "moon", "sun", "ice", "raven", "thunder", "flame", "frost", "ember"
        ];
        let prefix = preffixes[rng.randu_range(0, preffixes.len())];
        let suffix = suffixes[rng.randu_range(0, suffixes.len())];
        return Strings::capitalize(format!("{prefix}{suffix}").as_str());
    }

    fn name_person(&self, mut figure: Person, surname: &Option<String>) -> Person {
        if let Some(civ) = &figure.civ {
            let culture = self.world.cultures.get(&civ.culture).unwrap();
            let first_name;
            match figure.sex() {
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
        let mut species = "species:fiend";
        if self.rng.rand_chance(0.3) {
            species = "species:leshen";
        }
        let species = self.world.species.id_of(species);
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
            self.world.people.insert(Person::new(id, &species, Importance::Important, year, xy));
            self.world.events.push(WorldEventDate { year }, xy, WorldEventEnum::PersonBorn(SimplePersonEvent { person_id: id }))
        }
    }

    fn beast_hunt_nearby(&mut self, date: WorldEventDate, person_id: &Id) {
        let beast = self.world.people.get(&person_id).unwrap();
        let mut rng = self.rng.derive("beast_attack");
        let xy = beast.position + Coord2::xy(rng.randi_range(-15, 15), rng.randi_range(-15, 15));
        let mut result = None;
        if let Some((sett_id, settlement)) = self.world.settlements.iter().find(|(_, sett)| sett.borrow().xy.to_coord() == xy) {
            let mut creature_force = BattleForce::from_creatures(&self.world, vec!(&beast));
            let mut settlement_force = BattleForce::from_defending_settlement(&self.world, sett_id, &settlement.borrow());
            let battle = creature_force.battle(&mut settlement_force, &mut rng, settlement.borrow().xy.to_coord(), sett_id);
            result = Some(battle);
        }
        drop(beast);
        if let Some(battle) = result {
            self.apply_battle_result(date, battle);
        }
    }

    fn create_simple_artifact(&mut self, date: WorldEventDate, creature_id: &Id) {
        let position = self.world.people.get(&creature_id).unwrap().position.clone();
        let artifact_id = self.create_artifact(date, position, &self.resources.materials.id_of("mat:steel"));
        let mut creature = self.world.people.get_mut(&creature_id).unwrap();
        creature.possesions.push(artifact_id);
        creature.importance = creature.importance.at_least(&Importance::Unimportant);
        self.world.events.push(date, position, WorldEventEnum::ArtifactPossession(ArtifactPossesionEvent { item: artifact_id, person: *creature_id }));

    }

    fn marry_random_person(&mut self, date: WorldEventDate, person_id: &Id) {
        let mut person = self.world.people.get_mut(&person_id).unwrap();
        let id = self.next_person_id.next();
        let spouse = Person::new(id, &person.species, person.importance.lower(), date.year, person.position)
            .civilization(&person.civ);
        let mut spouse = self.name_person(spouse, &None);
        spouse.set_sex(person.sex().opposite());
        spouse.last_name = person.last_name.clone();
        spouse.next_of_kin.push(NextOfKin {
            person_id: person.id,
            relative: Relative::Spouse
        });
        person.next_of_kin.push(NextOfKin {
            person_id: spouse.id,
            relative: Relative::Spouse
        });
        drop(person);
        self.world.events.push(date, spouse.position, WorldEventEnum::Marriage(MarriageEvent { person1_id: *person_id, person2_id: spouse.id }));
        self.world.people.insert(spouse);
    }

    fn have_child_with(&mut self, date: WorldEventDate, father_id: Id, mother_id: Id) {
        let mut father = self.world.people.get_mut(&father_id).unwrap();
        let mut mother = self.world.people.get_mut(&mother_id).unwrap();
        let id = self.next_person_id.next();
        let child = self.create_child(id, date.year, &father, &mother);
        self.world.events.push(date, child.position, WorldEventEnum::PersonBorn(SimplePersonEvent { person_id: child.id }));
        father.next_of_kin.push(NextOfKin { 
            person_id: child.id,
            relative: Relative::Child
        });
        mother.next_of_kin.push(NextOfKin { 
            person_id: child.id,
            relative: Relative::Child
        });
        drop(father);
        drop(mother);
        self.world.people.insert(child);
    }

    fn colonize_new_settlement(&mut self, date: WorldEventDate, id: Id) {
        let mut person = self.world.people.get_mut(&id).unwrap();
        let xy = person.position.clone();
        if let Some(civ) = &mut person.civ {
            let culture = self.world.cultures.get(&civ.culture).unwrap();
            let settlement = generate_settlement(&self.rng, date.year, xy, id.clone(), culture, civ.faction, &self.world, &self.world.map, &self.parameters.regions).clone();
            if let Some(settlement) = settlement {
                let position = settlement.xy;
                let id = self.world.settlements.insert(settlement);
                self.world.events.push(date, position.to_coord(), WorldEventEnum::SettlementFounded(SettlementFoundedEvent { settlement_id: id, founder_id: id }));
                let mut faction = self.world.factions.get_mut(&civ.faction);
                faction.settlements.insert(id);
                civ.leader_of_settlement = Some(id);
                if let Some(spouse) = person.spouse() {
                    let mut spouse = self.world.people.get_mut(spouse).unwrap();
                    let spouse = spouse.borrow_mut();
                    (*spouse).position = position.to_coord();
                }
                person.position = position.to_coord();
            }
        }
    }

    fn apply_battle_result(&mut self, date: WorldEventDate, battle: (BattleResult, BattleResult)) {
        for (killed, killer_id) in battle.0.creature_casualties.iter() {
            let killer = killer_id.and_then(|id| Some(self.world.people.get(&id).unwrap()));
            let weapon = killer.and_then(|killer| killer.possesions.get(0).and_then(|id| Some(id.clone())));
            let artifact_material = self.kill_person(date, *killed, CauseOfDeath::KilledInBattle(killer_id.clone(), weapon));
            if let Some(artifact_material) = artifact_material {
                if let Some(killer_id) = killer_id {
                    let artifact_id = self.create_artifact(date, battle.0.location, &artifact_material);
                    let mut killer = self.world.people.get_mut(killer_id).unwrap();
                    killer.possesions.push(artifact_id);
                    killer.importance = killer.importance.at_least(&Importance::Unimportant);
                    self.world.events.push(date, battle.0.location, WorldEventEnum::ArtifactPossession(ArtifactPossesionEvent { item: artifact_id, person: *killer_id }));
                    // Else, who would get the artifact?
                }
            }
        }
        if let Some(settlement_id) = battle.0.belligerent_settlement {
            let mut settlement = self.world.settlements.get_mut(&settlement_id);
            settlement.kill_military(battle.0.army_casualties, &self.rng);
            settlement.kill_civilians(battle.0.civilian_casualties);
        }
        for (killed, killer_id) in battle.1.creature_casualties.iter() {
            let killer = killer_id.and_then(|id| Some(self.world.people.get(&id).unwrap()));
            let weapon = killer.and_then(|killer| killer.possesions.get(0).and_then(|id| Some(id.clone())));
            let artifact_material = self.kill_person(date, *killed, CauseOfDeath::KilledInBattle(killer_id.clone(), weapon));
            if let Some(artifact_material) = artifact_material {
                let artifact_id = self.create_artifact(date, battle.1.location, &artifact_material);
                if let Some(killer_id) = killer_id {
                    let mut killer = self.world.people.get_mut(killer_id).unwrap();
                    killer.possesions.push(artifact_id);
                    killer.importance = killer.importance.at_least(&Importance::Unimportant);
                    self.world.events.push(date, battle.1.location, WorldEventEnum::ArtifactPossession(ArtifactPossesionEvent { item: artifact_id, person: *killer_id }));
                    // Else, who would get the artifact?
                }
            }
        }
        if let Some(settlement_id) = battle.1.belligerent_settlement {
            let mut settlement = self.world.settlements.get_mut(&settlement_id);
            settlement.kill_military(battle.1.army_casualties, &self.rng);
            settlement.kill_civilians(battle.1.civilian_casualties);
        }
        self.world.events.push(date, battle.0.location, WorldEventEnum::Battle(crate::BattleEvent { battle_result: battle }));
    }

}

fn generate_settlement(rng: &Rng, founding_year: u32, seed_pos: Coord2, leader: Id, culture: &Culture, faction: Id, world_graph: &World, world_map: &WorldTopology, regions: &Vec<Region>) -> Option<Settlement> {
    let mut rng = rng.derive("settlement");
    let mut xy = None;
    let dist = 25;
    let x = ((seed_pos.x - dist).clamp(0, world_map.size.x() as i32 - 1))..((seed_pos.x + dist).clamp(0, world_map.size.x() as i32 - 1));
    let y = ((seed_pos.y - dist).clamp(0, world_map.size.y() as i32 - 1))..((seed_pos.y + dist).clamp(0, world_map.size.y() as i32 - 1));
    'candidates: for _ in 1..20 {
        let txy = Point2D(rng.randu_range(x.start as usize, x.end as usize), rng.randu_range(y.start as usize, y.end as usize));
        let tile = world_graph.map.tile(txy.0, txy.1);
        if tile.region_id == 0 {// Ocean
            continue;
        }
        for (_, settlement) in world_graph.settlements.iter() {
            if settlement.borrow().xy.dist_squared(&txy) <= 2_f32.powi(2) {
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
enum ActionToSimulate {
    None,
    Death(Id),
    GreatBeastHunt(Id),
    ComissionArtifact(Id),
    MarryRandomPerson(Id),
    HaveChildWith(Id, Id),
    ColonizeNewSettlement(Id)
}