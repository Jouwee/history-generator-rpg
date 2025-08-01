use crate::{commons::interpolate::lerp, world::{history_generator::WorldGenerationParameters, unit::{Unit, UnitType}}};

pub(crate) struct Storyteller {
}

impl Storyteller {

    pub(crate) fn new() -> Self {
        Self {  }
    }

    pub(crate) fn story_teller_unit_chances(&self, params: &WorldGenerationParameters, unit: &Unit) -> UnitChances {
        let mut chances = BASE_CHANCES.clone();
        
        if unit.unit_type == UnitType::Village {
            // Balances unit population
            let population_divergence = unit.creatures.len() as f32 / params.st_city_population as f32;
            if population_divergence < 0.8 {
                chances.have_child = chances.have_child * 1.5;
                chances.disease_death = chances.disease_death * 0.1;
                chances.leave_for_bandits = chances.leave_for_bandits * 0.1;
            } else if population_divergence > 1.5 {
                chances.have_child = chances.have_child * 0.2;
                chances.disease_death = chances.disease_death * 1.2;
                chances.leave_for_bandits = chances.leave_for_bandits * 1.0;
            } else if population_divergence > 1.2 {
                chances.have_child = chances.have_child * 0.5;
                chances.disease_death = chances.disease_death * 1.2;
                chances.leave_for_bandits = chances.leave_for_bandits * 1.1;
            }
        }

        return lerp_chances(&BASE_CHANCES, &chances, params.st_strength)
    }

}

#[derive(Clone)]
pub(crate) struct UnitChances {
    pub(crate) disease_death: f32,
    pub(crate) leave_for_bandits: f32,
    pub(crate) have_child: f32
}

const BASE_CHANCES: UnitChances = UnitChances {
    disease_death: 0.0015,
    have_child: 1.,
    leave_for_bandits: 0.001
};

fn lerp_chances(a: &UnitChances, b: &UnitChances, strength: f32) -> UnitChances {
    UnitChances {
        disease_death: lerp(a.disease_death as f64, b.disease_death as f64, strength as f64) as f32,
        have_child: lerp(a.have_child as f64, b.have_child as f64, strength as f64) as f32,
        leave_for_bandits: lerp(a.leave_for_bandits as f64, b.leave_for_bandits as f64, strength as f64) as f32,
    }
}
