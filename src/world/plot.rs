use std::usize;

use serde::{Deserialize, Serialize};

use crate::{commons::{id_vec::IdVec, xp_table::xp_to_level}, world::{creature::{Creature, CreatureId}, world::World}};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize, Deserialize)]
pub(crate) struct PlotId(usize);

impl crate::commons::id_vec::Id for PlotId {
    fn new(id: usize) -> Self {
        PlotId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

pub(crate) type Plots = IdVec<Plot>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Plot {
    pub(crate) goal: PlotGoal,
    plotter: CreatureId,
    plot_difficulty: f64,
    supporters: Vec<CreatureId>,
    plot_strength: f64,
    status: PlotStatus,
}

impl Plot {

    pub(crate) fn new(goal: PlotGoal, plotter: CreatureId, world: &World) -> Self {
        let plot_difficulty = goal.difficulty(world);
        let mut plot = Self {
            goal,
            plotter,
            plot_difficulty,
            supporters: Vec::new(),
            plot_strength: 0.,
            status: PlotStatus::Ongoing,
        };
        let plotter = world.creatures.get(&plotter);
        plot.plot_strength = plot.supporter_strength(&plotter);
        return plot;
    }

    pub(crate) fn add_supporter(&mut self, plot_id: PlotId, creature_id: CreatureId, creature: &mut Creature) {
        self.supporters.push(creature_id);
        self.plot_strength += self.supporter_strength(&creature);
        creature.supports_plot = Some(plot_id);
    }

    pub(crate) fn remove_supporter(&mut self, creature_id: CreatureId, creature: &mut Creature) {
        let i = self.supporters.iter().position(|id| id == &creature_id);
        if let Some(i) = i {
            self.supporters.remove(i);
            self.plot_strength -= self.supporter_strength(&creature);
            creature.supports_plot = None;
        }
    }

    fn supporter_strength(&self, supporter: &Creature) -> f64 {
        return xp_to_level(supporter.experience) as f64
    }

    pub(crate) fn success_chance(&self) -> f32 {
        if self.plot_strength > self.plot_difficulty {
            return 1.
        } else {
            let pct_of_strength = self.plot_strength / self.plot_difficulty;
            return pct_of_strength.powi(3) as f32
        }
    }

    pub(crate) fn verify_success(&mut self, world: &World) {
        if self.status != PlotStatus::Ongoing {
            return;
        }

        match self.goal {
            PlotGoal::KillBeast(creature_id) => {
                let creature = world.creatures.get(&creature_id);
                if creature.death.is_some() {
                    self.status = PlotStatus::Succeeded
                }
            }
        };

        let mut plotter = world.creatures.get_mut(&self.plotter);

        if self.status == PlotStatus::Ongoing && plotter.death.is_some() {
            self.status = PlotStatus::Failed;
        }

        // If failed or suceeded
        if self.status != PlotStatus::Ongoing {
            plotter.supports_plot = None;

            for supporter in self.supporters.iter() {
                let mut supporter = world.creatures.get_mut(supporter);
                supporter.supports_plot = None;
            }

            // Purges from the memory
            self.supporters.clear();
        }
    }

}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum PlotGoal {
    KillBeast(CreatureId),
}

impl PlotGoal {

    fn difficulty(&self, world: &World) -> f64 {
        match self {
            Self::KillBeast(creature_id) => {
                let creature = world.creatures.get(creature_id);
                return xp_to_level(creature.experience) as f64
            }
        }
    }

}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum PlotStatus {
    Ongoing,
    Failed,
    Succeeded,
}