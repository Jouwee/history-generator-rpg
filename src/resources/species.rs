use crate::{commons::{resource_map::ResourceMap, rng::Rng}, engine::{assets::{assets, GetSprite, ImageSheetSprite}, geometry::Size2D}, resources::material::MaterialId, world::attributes::Attributes};

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
    pub(crate) appearance: SpeciesAppearance,
    pub(crate) intelligence: SpeciesIntelligence,
    pub(crate) attributes: Attributes,
    pub(crate) innate_actions: Vec<ActionId>,
    pub(crate) drops: Vec<MaterialId>,
}

impl Species {

    pub(crate) fn new(name: &str, appearance: SpeciesAppearance) -> Species {
        Species {
            name: String::from(name),
            appearance,
            intelligence: SpeciesIntelligence::Civilized,
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

const SPECIES_SPRITE_SIZE: Size2D = Size2D(48, 48);

#[derive(Debug, Clone)]
pub(crate) enum SpeciesAppearance {
    Single(String),
    Composite {
        base: Vec<String>,
        top: Vec<String>,
    }
}

impl SpeciesAppearance {

    pub(crate) fn collapse(&self) -> CreatureAppearance {
        let mut rng = Rng::rand();
        match self {
            Self::Single(path) => {
                let len = assets().image_sheet(path, SPECIES_SPRITE_SIZE).len();
                return CreatureAppearance::Single(path.clone(), rng.randu_range(0, len))
            },
            Self::Composite { base, top } => {
                return CreatureAppearance::Composite {
                    base: (rng.item(base).unwrap().clone(), 0),
                    top: (rng.item(top).unwrap().clone(), 0)
                }
            }
        }
    }

}

#[derive(Debug, Clone)]
pub(crate) enum CreatureAppearance {
    Single(String, usize),
    Composite {
        base: (String, usize),
        top: (String, usize),
    }
}

impl CreatureAppearance {

    pub(crate) fn textures(&self) -> Vec<ImageSheetSprite> {
        let mut assets = assets();
        match self {
            Self::Single(path, i) => vec!(assets.image_sheet(path, SPECIES_SPRITE_SIZE).sprite(*i).unwrap()),
            Self::Composite { base, top } => {
                vec!(
                    assets.image_sheet(&base.0, SPECIES_SPRITE_SIZE).sprite(base.1).unwrap(),
                    assets.image_sheet(&top.0, SPECIES_SPRITE_SIZE).sprite(top.1).unwrap(),
                )
            }
        }
    }

}