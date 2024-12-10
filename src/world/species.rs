use crate::commons::history_vec::Id;

pub struct Species {
    pub id: Id,
    pub name: String,
    pub lifetime: SpeciesLifetime,
    pub intelligence: SpeciesIntelligence,
    pub fertility: SpeciesFertility
}

impl Species {

    pub fn new(id: Id, name: &str) -> Species {
        Species {
            id,
            name: String::from(name),
            intelligence: SpeciesIntelligence::Civilized,
            lifetime: SpeciesLifetime { max_age: 120 },
            fertility: SpeciesFertility { male_drop: 0.96, female_drop: 0.92 },
        }
    }

    pub fn intelligence(mut self, intelligence: SpeciesIntelligence) -> Self {
        self.intelligence = intelligence;
        self
    }

    pub fn lifetime(mut self, max_age: u32) -> Self {
        self.lifetime = SpeciesLifetime { max_age };
        self
    }

    pub fn fertility(mut self, fertility: f32) -> Self {
        self.fertility = SpeciesFertility { male_drop: fertility, female_drop: fertility };
        self
    }

}

pub struct SpeciesLifetime {
    pub max_age: u32
}

#[derive(PartialEq)]
pub enum SpeciesIntelligence {
    Instinctive,
    Civilized
}

pub struct SpeciesFertility {
    pub male_drop: f32,
    pub female_drop: f32
}