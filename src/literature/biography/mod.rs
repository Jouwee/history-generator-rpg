use crate::{commons::history_vec::Id, world::{battle_simulator::FinalResult, person::Person, topology::WorldTileData}, Relative, World, WorldEvent, WorldEventEnum};

pub struct BiographyWriter<'a> { 
    world: &'a World   
}

impl<'a> BiographyWriter<'a> {

    pub fn new(world: &'a World) -> BiographyWriter {
        return BiographyWriter {
            world
        }
    }

    pub fn tile(&self, tile: &WorldTileData) -> String {
        return format!("{:?}, el {}, {}, growth: {}", tile.xy, tile.elevation, tile.region_id, (tile.soil_fertility - 0.5) * 0.01);
    }

    pub fn settlement(&self, id: &Id) -> String {
        let settlement = self.world.settlements.get(id);
        let faction = self.world.factions.get(&settlement.faction_id);
        let mut description = format!(
            "{} {:?}\nPart of the {}\nPopulation: {}\nFounded in: {}\nMilitary: {}\nGold: {}",
            settlement.name,
            id,
            faction.name,
            settlement.demographics.population,
            settlement.founding_year,
            settlement.military.conscripts + settlement.military.trained_soldiers,
            settlement.gold
        );
        description.push_str("\n\nHistory:\n");
        for event in self.world.events.iter_settlement(id) {
            description.push_str(&self.event(event));        
            description.push_str("\n");
        }
        return description;
    }

    pub fn event(&self, event: &WorldEvent) -> String {
        let date = &event.date;
        match &event.event {
            WorldEventEnum::PersonBorn(event) => {
                let person = self.world.people.get(&event.person_id).unwrap();
                if let Some(person_id) = person.find_next_of_kin(Relative::Parent) {
                    let parent = self.world.people.get(person_id).unwrap();
                    return format!("In {}, {} fathered {}", date, self.name(&parent), self.name(&person))
                } else {
                    return format!("In {}, {} was born", date, self.name(&person))
                }
            },
            WorldEventEnum::PersonDeath(event) => {
                return format!("In {}, {} died", date, self.name(&self.world.people.get(&event.person_id).unwrap()))
            },
            WorldEventEnum::SettlementFounded(event) => {
                let settlement = self.world.settlements.get(&event.settlement_id);
                return format!("In {}, {} found the city of {}", date, self.name(&self.world.people.get(&event.founder_id).unwrap()), settlement.name)
            },
            WorldEventEnum::NewSettlementLeader(event) => {
                return format!("In {}, {} became the new leader of {}", date, self.name(&self.world.people.get(&event.new_leader_id).unwrap()), self.world.settlements.get(&event.settlement_id).name)
            },
            WorldEventEnum::Marriage(event) => {
                return format!("In {}, {} and {} married", date, self.name(&self.world.people.get(&event.person1_id).unwrap()), self.birth_name(&self.world.people.get(&event.person2_id).unwrap()))
            },
            WorldEventEnum::WarDeclared(event) => {
                let faction = self.world.factions.get(&event.faction1_id);
                let faction2 = self.world.factions.get(&event.faction2_id);
                return format!("In {}, a war between the {} and the {} started", date, faction.name, faction2.name)
            }
            WorldEventEnum::PeaceDeclared(event) => {
                let faction = self.world.factions.get(&event.faction1_id);
                let faction2 = self.world.factions.get(&event.faction2_id);
                return format!("In {}, the war between the {} and the {} ended", date, faction.name, faction2.name)
            }
            WorldEventEnum::ArtifactCreated(event) => {
                let artifact = self.world.artifacts.get(&event.item);
                return format!("In {}, an artifact was made. {}", date, artifact.description(self.world))
            }
            WorldEventEnum::Battle(event) => {
                let (attacker, defender) = &event.battle_result;

                let attacker_name;
                if let Some(faction) = attacker.belligerent_faction {
                    let faction = self.world.factions.get(&faction);
                    attacker_name = faction.name.clone();
                } else {
                    if attacker.creature_participants.len() == 1 {
                        let creature = self.world.people.get(attacker.creature_participants.get(0).unwrap()).unwrap();
                        attacker_name = self.name(&creature);
                    } else {
                        attacker_name = format!("{}", attacker.creature_participants.len()).to_string();
                    }
                }

                let defender_name;
                if let Some(faction) = defender.belligerent_faction {
                    let faction = self.world.factions.get(&faction);
                    defender_name = faction.name.clone();
                } else {
                    if defender.creature_participants.len() == 1 {
                        let creature = self.world.people.get(defender.creature_participants.get(0).unwrap()).unwrap();
                        defender_name = self.name(&creature);
                    } else {
                        defender_name = String::from("UNKNOWN");
                    }
                }

                let settlement = self.world.settlements.get(&attacker.location_settlement);
                let location_name = settlement.name.clone();

                let battle_result;

                match (&attacker.result, &defender.result) {
                    (FinalResult::Defeat, FinalResult::Victory) => battle_result = "but was defeated",
                    (FinalResult::Flee, FinalResult::Victory) => battle_result = "but had to flee",
                    (FinalResult::Victory, FinalResult::Flee) => battle_result = "and made them flee",
                    (FinalResult::Victory, FinalResult::Defeat) => battle_result = "and emerged vitorious",
                    _ => battle_result = "and it was a stalemate",
                }

                let mut attacker_kill_description = String::from("");
                for (creature, _) in attacker.creature_casualties.iter() {
                    let creature = self.world.people.get(creature).unwrap();
                    attacker_kill_description.push_str(&self.name(&creature));
                    attacker_kill_description.push_str("\n");
                }
                if attacker.army_casualties > 0 {
                    attacker_kill_description.push_str(&format!("{} soldiers", attacker.army_casualties));
                    attacker_kill_description.push_str("\n");
                }
                if attacker.civilian_casualties > 0 {
                    attacker_kill_description.push_str(&format!("{} civilians", attacker.civilian_casualties));
                    attacker_kill_description.push_str("\n");
                }
                if attacker_kill_description.len() == 0 {
                    attacker_kill_description = "suffered no casualties. ".to_string();
                } else {
                    attacker_kill_description = format!("lost: \n{attacker_kill_description}");
                }

                let mut defender_kill_description = String::from("");
                for (creature, _) in defender.creature_casualties.iter() {
                    let creature = self.world.people.get(creature).unwrap();
                    defender_kill_description.push_str(&self.name(&creature));
                    defender_kill_description.push_str("\n");
                }
                if defender.army_casualties > 0 {
                    defender_kill_description.push_str(&format!("{} soldiers", defender.army_casualties));
                    defender_kill_description.push_str("\n");
                }
                if defender.civilian_casualties > 0 {
                    defender_kill_description.push_str(&format!("{} civilians", defender.civilian_casualties));
                    defender_kill_description.push_str("\n");
                }
                if defender_kill_description.len() == 0 {
                    defender_kill_description = "suffered no casualties. ".to_string();
                } else {
                    defender_kill_description = format!("lost: \n{defender_kill_description}");
                }

                let kill_description = format!("In the end, the attackers {attacker_kill_description}While the defenders {defender_kill_description}");

                return String::from(format!("In {date}, {attacker_name} attacked {defender_name} at {location_name}, {battle_result}.\n{kill_description}"));
            }
            WorldEventEnum::ArtifactPossession(evt) => {
                let person = self.world.people.get(&evt.person).unwrap();
                let artifact = self.world.artifacts.get(&evt.item);
                return format!("In {}, {} became the wielder of {}", date, self.name(&person), artifact.name(self.world))
            }
        }
    }

    pub fn name(&self, figure: &Person) -> String {
        match figure.name() {
            Some(name) => format!("{} ({})", name, figure.id.0),
            None => format!("{} ({})", self.descriptor(figure), figure.id.0)
        }
    }

    pub fn birth_name(&self, figure: &Person) -> String {
        match figure.birth_name() {
            Some(name) => format!("{} ({})", name, figure.id.0),
            None => format!("{} ({})", self.descriptor(figure), figure.id.0)
        }
    }

    pub fn descriptor(&self, figure: &Person) -> String {
        let species = self.world.species.get(&figure.species).unwrap();
        return format!("a {}", species.name)
    }

}

