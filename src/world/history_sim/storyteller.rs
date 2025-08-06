use crate::{commons::{interpolate::lerp, rng::Rng}, world::{history_generator::WorldGenerationParameters, unit::{UnitId, UnitType}, world::World}};

pub(crate) struct Storyteller {
    params: WorldGenerationParameters,
    selected_for_cities: Vec<UnitId>
}

impl Storyteller {

    pub(crate) fn new(params: WorldGenerationParameters) -> Self {
        Self {
            params,
            selected_for_cities: Vec::new(),
        }
    }

    pub(crate) fn global_chances(&mut self, rng: &mut Rng, world: &World) -> GlobalChances {
        let mut chances = BASE_GLOBAL_CHANCES.clone();

        // Check city count, village count, and maybe promote village to city
        let mut villages = 0;
        for unit_id in world.units.iter_ids::<UnitId>() {
            let unit = world.units.get(&unit_id);
            if let UnitType::Village = unit.unit_type {
                if unit.creatures.len() == 0 {
                    self.selected_for_cities.retain(|id| id != &unit_id);
                    continue;
                }
                if self.selected_for_cities.len() < self.params.st_city_count as usize && !self.selected_for_cities.contains(&unit_id) && rng.rand_chance(0.3) {
                    self.selected_for_cities.push(unit_id);
                } else {
                    villages += 1;
                }
            }
        }


        let villages_divergence = villages as f32 / self.params.st_village_count as f32;
        if villages_divergence > 1.2 {
            chances.spawn_village *= 0.;
        } else if villages_divergence < 0.8 {
            chances.spawn_village *= 2.;
        }

        return lerp_global_chances(&BASE_GLOBAL_CHANCES, &chances, self.params.st_strength)
    }

    pub(crate) fn story_teller_unit_chances(&self, unit_id: &UnitId, world: &World) -> UnitChances {
        let mut chances = BASE_UNIT_CHANCES.clone();
        
        let unit = world.units.get(unit_id);

        if unit.unit_type == UnitType::Village {

            let pop_goal = match self.selected_for_cities.contains(unit_id) {
                true => self.params.st_city_population,
                false => self.params.st_village_population,
            };
            let adults = unit.creatures.iter().filter(|id| {
                let creature = world.creatures.get(*id);
                (world.date - creature.birth).year() > 18
            }).count();

            // Balances unit population
            let population_divergence = unit.creatures.len() as f32 / pop_goal as f32;
            if population_divergence < 0.8 {
                chances.have_child = chances.have_child * 1.5;
            } else if population_divergence > 1.5 {
                chances.have_child = chances.have_child * 0.;
            } else if population_divergence > 1.2 {
                chances.have_child = chances.have_child * 0.5;
            }

            // Balances unit population
            let population_divergence = adults as f32 / pop_goal as f32;
            if population_divergence < 0.8 {
                chances.disease_death = chances.disease_death * 0.;
                chances.leave_for_bandits = chances.leave_for_bandits * 0.;
            } else if population_divergence > 1.5 {
                chances.disease_death = chances.disease_death * 1.2;
                chances.leave_for_bandits = chances.leave_for_bandits * 1.1;
            } else if population_divergence > 1.2 {
                chances.disease_death = chances.disease_death * 1.2;
                chances.leave_for_bandits = chances.leave_for_bandits * 1.0;
            }
        }

        return lerp_unit_chances(&BASE_UNIT_CHANCES, &chances, self.params.st_strength)
    }

}

#[derive(Clone)]
pub(crate) struct UnitChances {
    pub(crate) disease_death: f32,
    pub(crate) leave_for_bandits: f32,
    pub(crate) have_child: f32
}

const BASE_UNIT_CHANCES: UnitChances = UnitChances {
    disease_death: 0.0015,
    have_child: 0.6,
    leave_for_bandits: 0.001
};

fn lerp_unit_chances(a: &UnitChances, b: &UnitChances, strength: f32) -> UnitChances {
    UnitChances {
        disease_death: lerp(a.disease_death as f64, b.disease_death as f64, strength as f64) as f32,
        have_child: lerp(a.have_child as f64, b.have_child as f64, strength as f64) as f32,
        leave_for_bandits: lerp(a.leave_for_bandits as f64, b.leave_for_bandits as f64, strength as f64) as f32,
    }
}


#[derive(Clone)]
pub(crate) struct GlobalChances {
    pub(crate) spawn_varningr: f32,
    pub(crate) spawn_wolf_pack: f32,
    pub(crate) spawn_village: f32,
}

const BASE_GLOBAL_CHANCES: GlobalChances = GlobalChances {
    spawn_varningr: 0.01,
    spawn_wolf_pack: 0.1,
    spawn_village: 0.01,
};

fn lerp_global_chances(a: &GlobalChances, b: &GlobalChances, strength: f32) -> GlobalChances {
    GlobalChances {
        spawn_varningr: lerp(a.spawn_varningr as f64, b.spawn_varningr as f64, strength as f64) as f32,
        spawn_wolf_pack: lerp(a.spawn_wolf_pack as f64, b.spawn_wolf_pack as f64, strength as f64) as f32,
        spawn_village: lerp(a.spawn_village as f64, b.spawn_village as f64, strength as f64) as f32,
    }
}
