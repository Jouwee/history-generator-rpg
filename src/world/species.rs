use std::collections::{BTreeMap, HashMap};

use image::ImageReader;
use opengl_graphics::{Filter, Texture, TextureSettings};

use crate::{commons::rng::Rng, game::action::ActionId};

use super::{attributes::Attributes, material::MaterialId};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub(crate) struct SpeciesId(usize);
impl crate::commons::id_vec::Id for SpeciesId {
    fn new(id: usize) -> Self {
        SpeciesId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Species {
    // pub(crate) name: String,
    pub(crate) appearance: SpeciesApearance,
    pub(crate) lifetime: SpeciesLifetime,
    pub(crate) intelligence: SpeciesIntelligence,
    pub(crate) fertility: SpeciesFertility,
    pub(crate) attributes: Attributes,
    pub(crate) innate_actions: Vec<ActionId>,
    pub(crate) drops: Vec<(MaterialId, usize)>
}

impl Species {

    pub(crate) fn new(_name: &str, appearance: SpeciesApearance) -> Species {
        Species {
            // name: String::from(name),
            appearance,
            intelligence: SpeciesIntelligence::Civilized,
            lifetime: SpeciesLifetime::new(120),
            fertility: SpeciesFertility { male_drop: 0.96, female_drop: 0.92 },
            attributes: Attributes { strength: 13, agility: 13, constitution: 13, unallocated: 13 },
            innate_actions: Vec::new(),
            drops: Vec::new()
        }
    }

    pub(crate) fn innate_actions(mut self, innate_actions: Vec<ActionId>) -> Self {
        self.innate_actions = innate_actions;
        return self
    }

    pub(crate) fn intelligence(mut self, intelligence: SpeciesIntelligence) -> Self {
        self.intelligence = intelligence;
        self
    }

    pub(crate) fn attributes(mut self, attributes: Attributes) -> Self {
        self.attributes = attributes;
        self
    }

    pub(crate) fn lifetime(mut self, max_age: u32) -> Self {
        self.lifetime = SpeciesLifetime::new(max_age);
        self
    }

    pub(crate) fn fertility(mut self, fertility: f32) -> Self {
        self.fertility = SpeciesFertility { male_drop: fertility, female_drop: fertility };
        self
    }

    pub(crate) fn drops(mut self, drops: Vec<(MaterialId, usize)>) -> Self {
        self.drops = drops;
        self
    }

}

#[derive(Debug, Clone)]
pub(crate) struct SpeciesLifetime {
    pub(crate) max_age: u32,
    pub(crate) adult_age: f32
}

impl SpeciesLifetime {
    pub(crate) fn new(max_age: u32) -> SpeciesLifetime {
        SpeciesLifetime {
            max_age,
            adult_age: max_age as f32 * 0.15
        }
    }

    pub(crate) fn is_adult(&self, age: f32) -> bool {
        return age > self.adult_age;
    }

}

#[derive(Debug, Clone, Hash, PartialEq)]
pub(crate) enum SpeciesIntelligence {
    Instinctive,
    Civilized
}

#[derive(Debug, Clone)]
pub(crate) struct SpeciesFertility {
    pub(crate) male_drop: f32,
    pub(crate) female_drop: f32
}

#[derive(Debug, Clone)]
pub(crate) struct SpeciesApearance {
    map: BTreeMap<String, HashMap<String, String>>
}

impl SpeciesApearance {

    pub(crate) fn single_sprite(path: &str) -> SpeciesApearance {
        let mut map = BTreeMap::new();
        let mut var = HashMap::new();
        var.insert(String::from("default"), String::from(path));
        map.insert(String::from("base"), var);
        Self { map }
    }

    pub(crate) fn composite(parts: Vec<(&str, Vec<(&str, &str)>)>) -> SpeciesApearance {
        let mut map = BTreeMap::new();
        for part in parts {
            let mut var = HashMap::new();
            for variation in part.1 {
                var.insert(String::from(variation.0), String::from(variation.1));
            }
            map.insert(String::from(part.0), var);
        }
        Self { map }
    }

    pub(crate) fn collapse(&self, rng: &Rng, hints: &HashMap<String, String>) -> CreatureAppearance {
        let mut collapsed = CreatureAppearance {
            map: BTreeMap::new()
        };
        for (k, v) in self.map.iter() {
            let hint = hints.get(k);
            if let Some(hint) = hint {
                collapsed.map.insert(k.clone(), (hint.to_string(), v.get(hint).unwrap().clone()));    
            } else {
                let mut rng = rng.derive(k);
                let variations: Vec<(&String, &String)> = v.iter().collect();
                let variation = variations[rng.randu_range(0, variations.len())];
                collapsed.map.insert(k.clone(), (variation.0.clone(), variation.1.clone()));
            }
        }
        collapsed
    }

}

#[derive(Debug, Clone)]
pub(crate) struct CreatureAppearance {
    pub(crate) map: BTreeMap<String, (String, String)>
}

impl CreatureAppearance {
    pub(crate) fn texture(&self) -> Vec<Texture> {
        let mut vec = Vec::new();
        for (_k, v) in self.map.iter() {
            // TODO: Don't load everytime
            let image = ImageReader::open(format!("./assets/sprites/{}", v.1)).unwrap().decode().unwrap();
            let settings = TextureSettings::new().filter(Filter::Nearest);
            vec.push(Texture::from_image(&image.to_rgba8(), &settings));
        }
        return vec;
    }
}