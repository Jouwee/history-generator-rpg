use crate::{commons::{rng::Rng, strings::Strings}, engine::{Id, Point2D}, CulturePrefab, RegionPrefab};

#[derive(Clone)]
pub struct Settlement {
    pub xy: Point2D,
    pub name: String,
    pub founding_year: u32,
    pub culture_id: Id,
    pub region_id: usize,
    pub demographics: Demographics
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
    region: &'a RegionPrefab,
    population: u32,
}

impl<'a> SettlementBuilder<'a> {

    pub fn colony(rng: &Rng, xy: Point2D, founding_year: u32, culture: &'a CulturePrefab, region: &'a RegionPrefab) -> SettlementBuilder<'a> {
        let mut rng = rng.derive("colony");
        let population = rng.randu_range(2, 10) as u32;
        return SettlementBuilder {
            rng,
            founding_year,
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
            region_id: self.region.id,
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
                return landmark_tr.to_owned() + placetype_tr;

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