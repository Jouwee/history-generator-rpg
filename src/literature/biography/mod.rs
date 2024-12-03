use crate::{commons::history_vec::Id, Relative, WorldEvent, WorldEventEnum, WorldGraph, WorldTileData};

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
                    return format!("In {}, {} fathered {}", date, parent.name(), person.name())
                } else {
                    return format!("In {}, {} was born", date, person.name())
                }
            },
            WorldEventEnum::PersonDeath(event) => {
                return format!("In {}, {} died", date, self.world.people.get(&event.person_id).unwrap().name())
            },
            WorldEventEnum::SettlementFounded(event) => {
                let settlement = self.world.settlements.get(&event.settlement_id);
                return format!("In {}, {} found the city of {}", date, self.world.people.get(&event.founder_id).unwrap().name(), settlement.name)
            },
            WorldEventEnum::NewSettlementLeader(event) => {
                return format!("In {}, {} became the new leader of {}", date, self.world.people.get(&event.new_leader_id).unwrap().name(), self.world.settlements.get(&event.settlement_id).name)
            },
            WorldEventEnum::Marriage(event) => {
                return format!("In {}, {} and {} married", date, self.world.people.get(&event.person1_id).unwrap().name(), self.world.people.get(&event.person2_id).unwrap().birth_name())
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
            WorldEventEnum::Siege(event) => {
                let settlement_attacker = self.world.settlements.get(&event.settlement1_id);
                let settlement_defender = self.world.settlements.get(&event.settlement2_id);
                let mut suffix = "had to retreat";
                if event.battle_result.defender_captured {
                    suffix = "captured the settlement"
                }
                let deaths = event.battle_result.attacker_deaths + event.battle_result.defender_deaths;
                if event.battle_result.attacker_victor {
                    return format!("In {}, {} sucessfully sieged {} and {suffix}. {deaths} people died", date, settlement_attacker.name, settlement_defender.name)
                } else {
                    return format!("In {}, {} attempted to siege {} and {suffix}. {deaths} people died", date, settlement_attacker.name, settlement_defender.name)
                }
            }
        }
    }

}

