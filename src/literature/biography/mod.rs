use crate::{engine::Id, world::settlement::{self, Settlement}, Event, Relative, WorldGraph, WorldTileData};

pub struct BiographyWriter<'a> { 
    world: &'a WorldGraph   
}

impl<'a> BiographyWriter<'a> {

    pub fn new(world: &'a WorldGraph) -> BiographyWriter {
        return BiographyWriter {
            world
        }
    }

    pub fn tile(&self, tile: &WorldTileData) -> String {
        return format!("{:?}, {}, growth: {}", tile.xy, tile.region_id, (tile.soil_fertility - 0.5) * 0.01);
    }

    pub fn settlement(&self, id: &Id) -> String {
        let settlement = self.world.settlements.get(id).unwrap();
        let faction = self.world.factions.get(&settlement.faction_id).unwrap();
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
        for event in self.world.events.iter() {
            match event {
                Event::SettlementFounded(year, settlement, person) => {
                    if settlement == id {
                        description.push_str(&self.event(event));        
                        description.push_str("\n");
                    }
                },
                // Event::Inheritance(year, person_a, person_b) => anals.push(format!("In {}, {} inherited everything from {}", year, world.people.get(person_b).unwrap().name(), world.people.get(person_a).unwrap().name())),
                // Event::RoseToPower(year, person) => anals.push(format!("In {}, {} rose to power", year, world.people.get(person).unwrap().name())),
                Event::Siege(year, _, _, settlement_attacker, settlement_defender, battle_result) => {
                    if settlement_attacker == id || settlement_defender == id {
                        description.push_str(&self.event(event));        
                        description.push_str("\n");
                    }
                }
                _ => {}
            }
        }
        return description;
    }

    pub fn event(&self, event: &Event) -> String {
        match event {
            Event::PersonBorn(year, person) => {
                let person = self.world.people.get(person).unwrap();
                if let Some(person_id) = person.find_next_of_kin(Relative::Parent) {
                    let parent = self.world.people.get(person_id).unwrap();
                    return(format!("In {}, {} fathered {}", year, parent.name(), person.name()))
                } else {
                    return(format!("In {}, {} was born", year, person.name()))
                }
            },
            Event::PersonDeath(year, person) => return(format!("In {}, {} died", year, self.world.people.get(person).unwrap().name())),
            Event::SettlementFounded(year, settlement, person) => {
                let settlement = self.world.settlements.get(settlement).unwrap();
                return(format!("In {}, {} found the city of {}", year, self.world.people.get(person).unwrap().name(), settlement.name))
            },
            Event::Marriage(year, person_a, person_b) => {
                return(format!("In {}, {} and {} married", year, self.world.people.get(person_a).unwrap().name(), self.world.people.get(person_b).unwrap().birth_name()))
            },
            Event::Inheritance(year, person_a, person_b) => return(format!("In {}, {} inherited everything from {}", year, self.world.people.get(person_b).unwrap().name(), self.world.people.get(person_a).unwrap().name())),
            Event::RoseToPower(year, person) => return(format!("In {}, {} rose to power", year, self.world.people.get(person).unwrap().name())),
            Event::WarDeclared(year, faction, faction2) => {
                let faction = self.world.factions.get(faction).unwrap();
                let faction2 = self.world.factions.get(faction2).unwrap();
                return(format!("In {}, a war between the {} and the {} started", year, faction.name, faction2.name))
            }
            Event::PeaceDeclared(year, faction, faction2) => {
                let faction = self.world.factions.get(faction).unwrap();
                let faction2 = self.world.factions.get(faction2).unwrap();
                return(format!("In {}, the war between the {} and the {} ended", year, faction.name, faction2.name))
            }
            Event::Siege(year, _, _, settlement_attacker, settlement_defender, battle_result) => {
                let settlement_attacker = self.world.settlements.get(settlement_attacker).unwrap();
                let settlement_defender = self.world.settlements.get(settlement_defender).unwrap();
                let mut suffix = "had to retreat";
                if battle_result.defender_captured {
                    suffix = "captured the settlement"
                }
                let deaths = battle_result.attacker_deaths + battle_result.defender_deaths;
                if battle_result.attacker_victor {
                    return(format!("In {}, {} sucessfully sieged {} and {suffix}. {deaths} people died", year, settlement_attacker.name, settlement_defender.name))
                } else {
                    return(format!("In {}, {} attempted to sieged {} and {suffix}. {deaths} people died", year, settlement_attacker.name, settlement_defender.name))
                }
            }
        }
    }

}

