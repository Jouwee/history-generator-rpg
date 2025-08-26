use crate::{commons::{interpolate::lerp, rng::Rng}, world::{date::Duration, history_generator::WorldGenerationParameters, site::{SiteId, SiteType}, world::World}};

pub(crate) struct Storyteller {
    params: WorldGenerationParameters,
    selected_for_cities: Vec<SiteId>
}

impl Storyteller {

    pub(crate) fn new(params: WorldGenerationParameters) -> Self {
        Self {
            params,
            selected_for_cities: Vec::new(),
        }
    }

    pub(crate) fn global_chances(&mut self, rng: &mut Rng, world: &World, delta_time: &Duration) -> GlobalChances {
        let mut chances = BASE_GLOBAL_CHANCES.clone();

        // Check city count, village count, and maybe promote village to city
        let mut villages = 0;
        for site_id in world.sites.iter_ids::<SiteId>() {
            let site = world.sites.get(&site_id);
            if let SiteType::Village = site.site_type {
                if site.creatures.len() == 0 {
                    self.selected_for_cities.retain(|id| id != &site_id);
                    continue;
                }
                if self.selected_for_cities.len() < self.params.st_city_count as usize && !self.selected_for_cities.contains(&site_id) && rng.rand_chance(0.3) {
                    self.selected_for_cities.push(site_id);
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

        let factor = delta_time.percentage_of_year();
        return lerp_global_chances(&BASE_GLOBAL_CHANCES, &chances, self.params.st_strength).scale(factor)
    }

    pub(crate) fn story_teller_site_chances(&self, site_id: &SiteId, world: &World, delta_time: &Duration) -> SiteChances {
        let mut chances = BASE_SITE_CHANCES.clone();
        
        let site = world.sites.get(site_id);

        if site.site_type == SiteType::Village {

            let pop_goal = match self.selected_for_cities.contains(site_id) {
                true => self.params.st_city_population,
                false => self.params.st_village_population,
            };
            let adults = site.creatures.iter().filter(|id| {
                let creature = world.creatures.get(*id);
                (world.date - creature.birth).year() > 18
            }).count();

            // Balances site population
            let population_divergence = site.creatures.len() as f32 / pop_goal as f32;
            if population_divergence < 0.8 {
                chances.have_child = chances.have_child * 1.5;
            } else if population_divergence > 1.5 {
                chances.have_child = chances.have_child * 0.;
            } else if population_divergence > 1.2 {
                chances.have_child = chances.have_child * 0.5;
            }

            // Balances site population
            let population_divergence = adults as f32 / pop_goal as f32;
            if population_divergence < 0.8 {
                chances.disease_death = chances.disease_death * 0.;
                chances.leave_for_bandits = chances.leave_for_bandits * 0.2;
            } else if population_divergence > 1.5 {
                chances.disease_death = chances.disease_death * 1.2;
                chances.leave_for_bandits = chances.leave_for_bandits * 1.1;
            } else if population_divergence > 1.2 {
                chances.disease_death = chances.disease_death * 1.2;
                chances.leave_for_bandits = chances.leave_for_bandits * 1.0;
            }
        }

        let factor = delta_time.percentage_of_year();
        return lerp_site_chances(&BASE_SITE_CHANCES, &chances, self.params.st_strength).scale(factor)
    }

}

#[derive(Clone)]
pub(crate) struct SiteChances {
    /// Just a base multiplier to be used in more complex rules
    pub(crate) base_multiplier: f32,
    pub(crate) disease_death: f32,
    pub(crate) leave_for_bandits: f32,
    pub(crate) have_child: f32,
    /// Look for a marriage candidate
    pub(crate) marry: f32,
    /// Look for a new job
    pub(crate) change_job: f32,
    /// Be inspired and make an artifac
    pub(crate) make_inspired_artifact: f32,
    /// Chance that a creature with a plottable goal will start a plot
    pub(crate) start_plot: f32,
    /// Chance that a creature will work on an active plot
    pub(crate) work_on_plot: f32,
    /// Chance that a great beast will attack a nearby settlement
    pub(crate) great_beast_hunt: f32,
}

impl SiteChances {
    fn scale(&self, factor: f32) -> Self {
        Self {
            base_multiplier: self.base_multiplier * factor,
            disease_death: self.disease_death * factor,
            leave_for_bandits: self.leave_for_bandits * factor,
            have_child: self.have_child * factor,
            marry: self.marry * factor,
            change_job: self.change_job * factor,
            make_inspired_artifact: self.make_inspired_artifact * factor,
            start_plot: self.start_plot * factor,
            work_on_plot: self.work_on_plot * factor,
            great_beast_hunt: self.great_beast_hunt * factor,
        }
    }
}

const BASE_SITE_CHANCES: SiteChances = SiteChances {
    base_multiplier: 1.,
    disease_death: 0.0015,
    have_child: 0.6,
    marry: 0.8,
    leave_for_bandits: 0.001,
    change_job: 0.005,
    make_inspired_artifact: 0.005,
    start_plot: 0.3,
    work_on_plot: 0.9,
    great_beast_hunt: 0.01
};

fn lerp_site_chances(a: &SiteChances, b: &SiteChances, strength: f32) -> SiteChances {
    SiteChances {
        base_multiplier: lerp(a.base_multiplier as f64, b.base_multiplier as f64, strength as f64) as f32,
        disease_death: lerp(a.disease_death as f64, b.disease_death as f64, strength as f64) as f32,
        have_child: lerp(a.have_child as f64, b.have_child as f64, strength as f64) as f32,
        marry: lerp(a.marry as f64, b.marry as f64, strength as f64) as f32,
        change_job: lerp(a.change_job as f64, b.change_job as f64, strength as f64) as f32,
        make_inspired_artifact: lerp(a.make_inspired_artifact as f64, b.make_inspired_artifact as f64, strength as f64) as f32,
        leave_for_bandits: lerp(a.leave_for_bandits as f64, b.leave_for_bandits as f64, strength as f64) as f32,
        start_plot: lerp(a.start_plot as f64, b.start_plot as f64, strength as f64) as f32,
        work_on_plot: lerp(a.work_on_plot as f64, b.work_on_plot as f64, strength as f64) as f32,
        great_beast_hunt: lerp(a.great_beast_hunt as f64, b.great_beast_hunt as f64, strength as f64) as f32,
    }
}


#[derive(Clone)]
pub(crate) struct GlobalChances {
    pub(crate) spawn_varningr: f32,
    pub(crate) spawn_wolf_pack: f32,
    pub(crate) spawn_village: f32,
}

impl GlobalChances {
    fn scale(&self, factor: f32) -> Self {
        Self {
            spawn_varningr: self.spawn_varningr * factor,
            spawn_wolf_pack: self.spawn_wolf_pack * factor,
            spawn_village: self.spawn_village * factor
        }
    }
}

const BASE_GLOBAL_CHANCES: GlobalChances = GlobalChances {
    spawn_varningr: 0.05,
    spawn_wolf_pack: 0.2,
    spawn_village: 0.01,
};

fn lerp_global_chances(a: &GlobalChances, b: &GlobalChances, strength: f32) -> GlobalChances {
    GlobalChances {
        spawn_varningr: lerp(a.spawn_varningr as f64, b.spawn_varningr as f64, strength as f64) as f32,
        spawn_wolf_pack: lerp(a.spawn_wolf_pack as f64, b.spawn_wolf_pack as f64, strength as f64) as f32,
        spawn_village: lerp(a.spawn_village as f64, b.spawn_village as f64, strength as f64) as f32,
    }
}
