use std::collections::{BTreeMap, HashMap};

use image::ImageReader;
use opengl_graphics::{Filter, Texture, TextureSettings};

use crate::{commons::{resource_map::ResourceMap, rng::Rng}, world::attributes::Attributes};

use super::action::ActionId;

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

pub(crate) type SpeciesMap = ResourceMap<SpeciesId, Species>;

#[derive(Debug, Clone)]
pub(crate) struct Species {
    pub(crate) name: String,
    pub(crate) appearance: SpeciesApearance,
    pub(crate) intelligence: SpeciesIntelligence,
    pub(crate) attributes: Attributes,
    pub(crate) innate_actions: Vec<ActionId>,
}

impl Species {

    pub(crate) fn new(name: &str, appearance: SpeciesApearance) -> Species {
        Species {
            name: String::from(name),
            appearance,
            intelligence: SpeciesIntelligence::Civilized,
            attributes: Attributes { strength: 13, agility: 13, constitution: 13, unallocated: 13 },
            innate_actions: Vec::new(),
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

}

#[derive(Debug, Clone, Hash, PartialEq)]
pub(crate) enum SpeciesIntelligence {
    Instinctive,
    Civilized
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
    pub(crate) fn texture(&self) -> Vec<(String, Texture)> {
        let mut vec = Vec::new();
        for (k, v) in self.map.iter() {
            // TODO: Don't load everytime
            let image = ImageReader::open(format!("./assets/sprites/{}", v.1)).unwrap().decode().unwrap();
            let settings = TextureSettings::new().filter(Filter::Nearest);
            vec.push((k.clone(), Texture::from_image(&image.to_rgba8(), &settings)));
        }
        return vec;
    }
}