// TODO: Break into files

use crate::world::{creature::{CauseOfDeath, Creature, CreatureId, Profession}, date::WorldDate, unit::UnitId, world::ArtifactId};

// ----------------------

pub(crate) enum Event {
    CreatureDeath { date: WorldDate, creature_id: CreatureId, cause_of_death: CauseOfDeath },
    CreatureBirth { date: WorldDate, creature_id: CreatureId },
    CreatureMarriage { date: WorldDate, creature_id: CreatureId, spouse_id: CreatureId },
    CreatureProfessionChange { date: WorldDate, creature_id: CreatureId, new_profession: Profession },
    ArtifactCreated { date: WorldDate, artifact: ArtifactId, creator: CreatureId },
    InheritedArtifact { date: WorldDate, creature_id: CreatureId, from: CreatureId, item: ArtifactId },
    BurriedWithPosessions { date: WorldDate, creature_id: CreatureId },
    ArtifactComission { date: WorldDate, creature_id: CreatureId, creator_id: CreatureId, item_id: ArtifactId },
    NewLeaderElected { date: WorldDate, unit_id: UnitId, creature_id: CreatureId },
}


// ----------------

pub(crate) struct Demographics {
    total: u16,
    children_male: u16,
    children_female: u16,
    adult_male: u16,
    adult_female: u16,
    adult_singles: u16,
    adult_married: u16,
    employed: u16,
    army: u16,
    peasants: u16,
    farmers: u16,
    artisans: u16,
    politicians: u16,
}

impl Demographics {

    pub(crate) fn new() -> Demographics {
        return Demographics {
            total: 0,
            children_male: 0,
            children_female: 0,
            adult_male: 0,
            adult_female: 0,
            adult_singles: 0,
            adult_married: 0, 
            employed: 0,
            army: 0,
            peasants: 0,
            farmers: 0,
            artisans: 0,
            politicians: 0,
        }
    }

    pub(crate) fn count(&mut self, reference: &WorldDate, creature: &Creature) {
        let age = (*reference - creature.birth).year();
        self.total += 1;
        if age < 18 {
            if creature.gender.is_male() {
                self.children_male += 1;
            } else {
                self.children_female += 1;
            }
        } else {
            if creature.gender.is_male() {
                self.adult_male += 1;
            } else {
                self.adult_female += 1;
            }
            if creature.spouse.is_none() {
                self.adult_singles += 1;
            } else {
                self.adult_married += 1;
            }
        }
        self.employed += 1;
        match creature.profession {
            Profession::None => self.employed -= 1,
            Profession::Peasant => self.peasants += 1,
            Profession::Farmer => self.farmers += 1,
            Profession::Blacksmith | Profession::Sculptor => self.artisans += 1,
            Profession::Guard => self.army += 1,
            Profession::Ruler => self.politicians += 1,
        }
    }

    pub(crate) fn print_console(&self) {
        println!("total: {}", self.total);
        println!("children_male: {} ({:.2?}%)", self.children_male, Self::pct(self.total, self.children_male));
        println!("children_female: {} ({:.2?}%)", self.children_female, Self::pct(self.total, self.children_female));
        println!("adult_male: {} ({:.2?}%)", self.adult_male, Self::pct(self.total, self.adult_male));
        println!("adult_female: {} ({:.2?}%)", self.adult_female, Self::pct(self.total, self.adult_female));
        println!("adult_singles: {} ({:.2?}%)", self.adult_singles, Self::pct(self.total, self.adult_singles));
        println!("adult_married: {} ({:.2?}%)", self.adult_married, Self::pct(self.total, self.adult_married));
        println!("employed: {} ({:.2?}%)", self.employed, Self::pct(self.total, self.employed));
        println!("army: {} ({:.2?}%)", self.army, Self::pct(self.employed, self.army));
        println!("peasants: {} ({:.2?}%)", self.peasants, Self::pct(self.employed, self.peasants));
        println!("farmers: {} ({:.2?}%)", self.farmers, Self::pct(self.employed, self.farmers));
        println!("artisans: {} ({:.2?}%)", self.artisans, Self::pct(self.employed, self.artisans));
        println!("politicians: {} ({:.2?}%)", self.politicians, Self::pct(self.employed, self.politicians));
    }

    fn pct(total: u16, count: u16) -> f32 {
        return (count as f32 / total as f32) * 100.
    }

}