use serde::{Deserialize, Serialize};

use crate::{commons::{resource_map::ResourceMap, rng::Rng}, engine::{assets::{assets, GetSprite, ImageSheetSprite}, audio::SoundEffect, geometry::Size2D}, resources::material::MaterialId, world::{attributes::Attributes, creature::CreatureGender}};

use super::action::ActionId;

// TODO(ROO4JcDl): Should serialize the string id, not the internal id
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq, Serialize, Deserialize)]
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
    pub(crate) appearance: SpeciesAppearance,
    pub(crate) max_hp: f32,
    pub(crate) intelligence: SpeciesIntelligence,
    pub(crate) attributes: Attributes,
    pub(crate) innate_actions: Vec<ActionId>,
    pub(crate) drops: Vec<MaterialId>,
    pub(crate) hurt_sound: Option<SoundEffect>
}

impl Species {

    pub(crate) fn new(name: &str, appearance: SpeciesAppearance) -> Species {
        Species {
            name: String::from(name),
            appearance,
            max_hp: 100.,
            intelligence: SpeciesIntelligence::Civilized,
            attributes: Attributes { strength: 13, agility: 13, constitution: 13, unallocated: 13 },
            innate_actions: Vec::new(),
            drops: Vec::new(),
            hurt_sound: None,
        }
    }

    pub(crate) fn max_hp(mut self, max_hp: f32) -> Self {
        self.max_hp = max_hp;
        return self
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

    pub(crate) fn hurt_sound(mut self, hurt_sound: SoundEffect) -> Self {
        self.hurt_sound = Some(hurt_sound);
        self
    }

    pub(crate) fn drops(mut self, drops: Vec<MaterialId>) -> Self {
        self.drops = drops;
        self
    }

}

#[derive(Debug, Clone, Hash, PartialEq)]
pub(crate) enum SpeciesIntelligence {
    Instinctive,
    Civilized
}

pub(crate) const SPECIES_SPRITE_SIZE: Size2D = Size2D(48, 48);

#[derive(Debug, Clone)]
pub(crate) enum SpeciesAppearance {
    Single(String),
    Composite {
        base: Vec<String>,
        top: Vec<String>,
    }
}

impl SpeciesAppearance {

    pub(crate) fn collapse(&self, gender: &CreatureGender) -> CreatureAppearance {
        let mut rng = Rng::rand();
        match self {
            Self::Single(path) => {
                let len = assets().image_sheet(path, SPECIES_SPRITE_SIZE).len();
                return CreatureAppearance::Single(path.clone(), rng.randu_range(0, len))
            },
            Self::Composite { base, top } => {
                return CreatureAppearance::Composite {
                    index: match gender {
                        CreatureGender::Male => 0,
                        CreatureGender::Female => 1,
                    },
                    base: rng.item(base).unwrap().clone(),
                    top: rng.item(top).unwrap().clone(),
                }
            }
        }
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum CreatureAppearance {
    Single(String, usize),
    Composite {
        index: usize,
        base: String,
        top: String,
    }
}

impl CreatureAppearance {

    pub(crate) fn textures(&self) -> Vec<ImageSheetSprite> {
        let mut assets = assets();
        match self {
            Self::Single(path, i) => vec!(assets.image_sheet(path, SPECIES_SPRITE_SIZE).sprite(*i).unwrap()),
            Self::Composite { index, base, top } => {
                vec!(
                    assets.image_sheet(&base, SPECIES_SPRITE_SIZE).sprite(*index).unwrap(),
                    assets.image_sheet(&top, SPECIES_SPRITE_SIZE).sprite(*index).unwrap(),
                )
            }
        }
    }

}