use crate::{resources::resources::Resources, world::world::World};

use super::{creature::{CauseOfDeath, CreatureId, Profession}, date::WorldDate, item::ItemId, unit::UnitId};

pub(crate) enum Event {
    CreatureDeath { date: WorldDate, creature_id: CreatureId, cause_of_death: CauseOfDeath },
    CreatureBirth { date: WorldDate, creature_id: CreatureId },
    CreatureMarriage { date: WorldDate, creature_id: CreatureId, spouse_id: CreatureId },
    CreatureProfessionChange { date: WorldDate, creature_id: CreatureId, new_profession: Profession },
    ArtifactCreated { date: WorldDate, artifact: ItemId, creator: CreatureId },
    InheritedArtifact { date: WorldDate, creature_id: CreatureId, from: CreatureId, item: ItemId },
    BurriedWithPosessions { date: WorldDate, creature_id: CreatureId, items_ids: Vec<ItemId> },
    ArtifactComission { date: WorldDate, creature_id: CreatureId, creator_id: CreatureId, item_id: ItemId },
    NewLeaderElected { date: WorldDate, unit_id: UnitId, creature_id: CreatureId },
    JoinBanditCamp { date: WorldDate, creature_id: CreatureId, unit_id: UnitId, new_unit_id: UnitId },
    CreateBanditCamp { date: WorldDate, creature_id: CreatureId, unit_id: UnitId, new_unit_id: UnitId },
}

impl Event {

    pub(crate) fn related_creatures(&self) -> Vec<CreatureId> {
        match self {
            Self::CreatureDeath { date: _, creature_id, cause_of_death: _ } => vec!(*creature_id),
            Self::CreatureBirth { date: _, creature_id } => vec!(*creature_id),
            Self::CreatureMarriage { date: _, creature_id, spouse_id } => vec!(*creature_id, *spouse_id),
            Self::CreatureProfessionChange { date: _, creature_id, new_profession: _ } => vec!(*creature_id),
            Self::ArtifactCreated { date: _, artifact: _, creator } => vec!(*creator),
            Self::InheritedArtifact { date: _, creature_id, from, item: _ } => vec!(*creature_id, *from),
            Self::BurriedWithPosessions { date: _, creature_id, items_ids: _ } => vec!(*creature_id),
            Self::ArtifactComission { date: _, creature_id, creator_id, item_id: _ } => vec!(*creature_id, *creator_id),
            Self::NewLeaderElected { date: _, unit_id: _, creature_id } => vec!(*creature_id),
            Self::JoinBanditCamp { date: _, creature_id, unit_id: _, new_unit_id: _ } => vec!(*creature_id),
            Self::CreateBanditCamp { date: _, creature_id, unit_id: _, new_unit_id: _ } => vec!(*creature_id),
        }
    }

    pub(crate) fn relates_to_creature(&self, creature_id: &CreatureId) -> bool {
        return self.related_creatures().iter().any(|id| id == creature_id);
    }

    pub(crate) fn related_artifacts(&self) -> Vec<ItemId> {
        match self {
            Self::CreatureDeath { date: _, creature_id: _, cause_of_death: _ } => vec!(),
            Self::CreatureBirth { date: _, creature_id: _ } => vec!(),
            Self::CreatureMarriage { date: _, creature_id: _, spouse_id: _ } => vec!(),
            Self::CreatureProfessionChange { date: _, creature_id: _, new_profession: _ } => vec!(),
            Self::ArtifactCreated { date: _, artifact, creator: _ } => vec!(*artifact),
            Self::InheritedArtifact { date: _, creature_id: _, from: _, item } => vec!(*item),
            Self::BurriedWithPosessions { date: _, creature_id: _, items_ids } => items_ids.clone(),
            Self::ArtifactComission { date: _, creature_id: _, creator_id: _, item_id } => vec!(*item_id),
            Self::NewLeaderElected { date: _, unit_id: _, creature_id: _ } => vec!(),
            Self::JoinBanditCamp { date: _, creature_id: _, unit_id: _, new_unit_id: _ } => vec!(),
            Self::CreateBanditCamp { date: _, creature_id: _, unit_id: _, new_unit_id: _ } => vec!(),
        }
    }

    pub(crate) fn relates_to_artifact(&self, artifact_id: &ItemId) -> bool {
        return self.related_artifacts().iter().any(|id| id == artifact_id);
    }

    pub(crate) fn event_text(&self, resources: &Resources, world: &World) -> String {
        match self {
            Event::CreatureBirth { date, creature_id } => {
                let creature = world.creatures.get(creature_id);
                let name = world.creature_desc(creature_id, resources);
                let father = world.creature_desc(&creature.father, resources);
                let mother = world.creature_desc(&creature.mother, resources);
                return format!("> {}, {} was born. Father: {}, Mother: {}", world.date_desc(date), name, father, mother);
            },
            Event::CreatureDeath { date, creature_id, cause_of_death } => {
                let name = world.creature_desc(creature_id, resources);
                return format!("> {}, {} died of {:?}", world.date_desc(date), name, cause_of_death);
            },
            Event::CreatureMarriage { date, creature_id, spouse_id } => {
                let name_a = world.creature_desc(creature_id, resources);
                let name_b = world.creature_desc(spouse_id, resources);
                return format!("> {}, {} and {} married", world.date_desc(date), name_a, name_b);
            },
            Event::CreatureProfessionChange { date, creature_id, new_profession } => {
                let name = world.creature_desc(creature_id, resources);
                return format!("> {}, {} became a {:?}", world.date_desc(date), name, new_profession);
            },
            Event::ArtifactCreated { date, artifact, creator } => {
                let name = world.creature_desc(creator, resources);
                let artifact = world.artifacts.get(artifact);
                return format!("> {}, {} created {:?}", world.date_desc(date), name, artifact.name(&resources.materials));
            },
            Event::BurriedWithPosessions { date, creature_id, items_ids: _ } => {
                let name = world.creature_desc(creature_id, resources);
                return format!("> {}, {} was buried with their possessions", world.date_desc(date), name);
            },
            Event::InheritedArtifact { date, creature_id, from, item } => {
                let name = world.creature_desc(creature_id, resources);
                let name_b = world.creature_desc(from, resources);
                let artifact = world.artifacts.get(item);
                return format!("> {}, {} inherited {} from {}", world.date_desc(date), name, artifact.name(&resources.materials), name_b);
            },
            Event::ArtifactComission { date, creature_id, creator_id, item_id } => {
                let name = world.creature_desc(creature_id, resources);
                let name_b = world.creature_desc(creator_id, resources);
                let artifact = world.artifacts.get(item_id);
                let creature = world.creatures.get(creature_id);
                let age = (*date - creature.birth).year();
                return format!("> {}, {} commissioned {} from {} for his {}th birthday", world.date_desc(date), name, artifact.name(&resources.materials), name_b, age);
            },
            Event::NewLeaderElected { date, unit_id, creature_id } => {
                let name = world.creature_desc(creature_id, resources);
                return format!("> {}, {} was elected new leader of {:?}", world.date_desc(date), name, *unit_id);
            },
            Event::JoinBanditCamp { date, creature_id, unit_id, new_unit_id } => {
                let name = world.creature_desc(creature_id, resources);
                return format!("> {}, {} left {:?} and joined the bandits at {:?}", world.date_desc(date), name, *unit_id, *new_unit_id);
            },
            Event::CreateBanditCamp { date, creature_id, unit_id, new_unit_id } => {
                let name = world.creature_desc(creature_id, resources);
                return format!("> {}, {} left {:?} and started a bandit camp at {:?}", world.date_desc(date), name, *unit_id, *new_unit_id);
            },
        }
            

    }

}