use crate::{commons::{rng::Rng, strings::Strings}, engine::{Id, Point2D}, CulturePrefab, RegionPrefab};

#[derive(Clone)]
pub struct Settlement {
    pub xy: Point2D,
    pub name: String,
    pub founding_year: u32,
    pub culture_id: Id,
    pub faction_id: Id,
    pub region_id: usize,
    pub gold: i32,
    pub military: Military,
    pub demographics: Demographics
}

impl Settlement {
    pub fn military_siege_power(&self) -> f32 {
        return (
            (self.military.trained_soldiers * 2) + (self.military.conscripts * 1)
        ) as f32;
    }

    pub fn military_defence_power(&self) -> f32 {
        return (
            (self.military.trained_soldiers * 2) + (self.military.conscripts * 1)
        ) as f32 * 1.2;
    }

    pub fn kill_military(&mut self, total_kills: u32, rng: &Rng) {
        let trained_ratio = rng.derive("kill").randf();
        let trained_kills = (total_kills as f32 * trained_ratio).floor() as u32;
        let trained_kills = trained_kills.min(self.military.trained_soldiers);
        self.military.trained_soldiers = self.military.trained_soldiers - trained_kills;
        let conscript_kills = total_kills - trained_kills;
        if conscript_kills > self.military.conscripts {
            // TODO:
            // panic!("Tried to kill {total_kills} military, but there's not enough military")
        } else {
            self.military.conscripts = self.military.conscripts - conscript_kills;
        }
        self.demographics.change_population(-(total_kills as i32));
    }

}

#[derive(Clone)]
pub struct Military {
    pub trained_soldiers: u32,
    pub conscripts: u32,
}

#[derive(Clone)]
pub struct Demographics {
    pub population: u32,
}

impl Demographics {
    pub fn change_population(&mut self, population_delta: i32) {
        self.population = (self.population as i32 + population_delta).max(0) as u32;
    }
}

pub struct SettlementBuilder<'a> {
    rng: Rng,
    xy: Point2D,
    founding_year: u32,
    culture: &'a CulturePrefab,
    faction_id: Id,
    region: &'a RegionPrefab,
    population: u32,
}

impl<'a> SettlementBuilder<'a> {

    pub fn colony(rng: &Rng, xy: Point2D, founding_year: u32, culture: &'a CulturePrefab, faction_id: Id, region: &'a RegionPrefab) -> SettlementBuilder<'a> {
        let mut rng = rng.derive("colony");
        let population = rng.randu_range(2, 10) as u32;
        return SettlementBuilder {
            rng,
            founding_year,
            faction_id,
            xy,
            culture,
            region,
            population
        }
    }

    pub fn create(&self) -> Settlement {
        return Settlement {
            xy: self.xy,
            name: Self::generate_location_name(&self.rng.derive("name"), self.culture, self.region),
            founding_year: self.founding_year,
            culture_id: self.culture.id,
            faction_id: self.faction_id,
            region_id: self.region.id,
            gold: 0,
            military: Military { trained_soldiers: 0, conscripts: 0 },
            demographics: Self::derive_demographics(&self.rng.derive("demographics"), self.population)
        }
    }

    fn generate_location_name(rng: &Rng, culture: &CulturePrefab, region: &RegionPrefab) -> String {
        let mut rng = rng.derive("name");

        let mut landmarks = Vec::new();
        landmarks.extend(&region.fauna);
        landmarks.extend(&region.flora);
        if let Some(landmark) = landmarks.get(rng.randu_range(0, landmarks.len())) {

            // TODO: Based on location
            let place_types = [String::from("fortress"), String::from("port")];
            if let Some(place_type) = place_types.get(rng.randu_range(0, place_types.len())) {

                // TODO: Fallback to something
                let landmark_tr = culture.language.dictionary.get(*landmark).unwrap_or(landmark);
                let placetype_tr = culture.language.dictionary.get(&*place_type).unwrap_or(place_type);
                return Strings::capitalize((landmark_tr.to_owned() + placetype_tr).as_str());

            }
        }
        // TODO: Fallback to something
        return Strings::capitalize("Settlement")
    }

    fn derive_demographics(rng: &Rng, population: u32) -> Demographics {
        return Demographics {
            population
        }
    }

}