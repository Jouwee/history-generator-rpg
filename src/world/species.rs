use crate::commons::history_vec::Id;

use super::attributes::Attributes;

pub struct Species {
    pub id: Id,
    pub name: String,
    pub texture: String,
    pub lifetime: SpeciesLifetime,
    pub intelligence: SpeciesIntelligence,
    pub fertility: SpeciesFertility,
    pub attributes: Attributes,
    pub drops: Vec<(Id, usize)>
}

impl Species {

    pub fn new(id: Id, name: &str, texture: &str) -> Species {
        Species {
            id,
            name: String::from(name),
            texture: String::from(texture),
            intelligence: SpeciesIntelligence::Civilized,
            lifetime: SpeciesLifetime::new(120),
            fertility: SpeciesFertility { male_drop: 0.96, female_drop: 0.92 },
            attributes: Attributes { strength: 13 },
            drops: Vec::new()
        }
    }

    pub fn intelligence(mut self, intelligence: SpeciesIntelligence) -> Self {
        self.intelligence = intelligence;
        self
    }

    pub fn attributes(mut self, attributes: Attributes) -> Self {
        self.attributes = attributes;
        self
    }

    pub fn lifetime(mut self, max_age: u32) -> Self {
        self.lifetime = SpeciesLifetime::new(max_age);
        self
    }

    pub fn fertility(mut self, fertility: f32) -> Self {
        self.fertility = SpeciesFertility { male_drop: fertility, female_drop: fertility };
        self
    }

    pub fn drops(mut self, drops: Vec<(Id, usize)>) -> Self {
        self.drops = drops;
        self
    }

}

pub struct SpeciesLifetime {
    pub max_age: u32,
    pub adult_age: f32
}

impl SpeciesLifetime {
    pub fn new(max_age: u32) -> SpeciesLifetime {
        SpeciesLifetime {
            max_age,
            adult_age: max_age as f32 * 0.15
        }
    }
}

impl SpeciesLifetime {

    pub fn is_adult(&self, age: f32) -> bool {
        return age > self.adult_age;
    }

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