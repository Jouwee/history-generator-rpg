use std::collections::{HashMap, HashSet};

use crate::engine::geometry::Coord2;

pub(crate) struct WorldMapFeatures {
    features: HashMap<Coord2, HashSet<MapFeature>>,
    empty_set: HashSet<MapFeature>
}

impl WorldMapFeatures {
    
    pub(crate) fn new() -> WorldMapFeatures {
        WorldMapFeatures { 
            features: HashMap::new(),
            empty_set: HashSet::new()
        }
    }

    pub(crate) fn get_features(&self, coord: Coord2) -> &HashSet<MapFeature> {
        if let Some(features) = self.features.get(&coord) {
            return features
        }
        &self.empty_set
    }

    pub(crate) fn has_road(&self, coord: Coord2) -> bool {
        if let Some(features) = self.features.get(&coord) {
            return features.contains(&MapFeature::Road)
        }
        return false
    }
    
}

#[derive(Hash, PartialEq, Eq)]
pub(crate) enum MapFeature {
    Road
}