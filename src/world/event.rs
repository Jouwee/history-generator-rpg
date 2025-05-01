use super::{creature::{CauseOfDeath, CreatureId, Profession}, date::WorldDate, item::ItemId, unit::UnitId};

pub(crate) enum Event {
    CreatureDeath { date: WorldDate, creature_id: CreatureId, cause_of_death: CauseOfDeath },
    CreatureBirth { date: WorldDate, creature_id: CreatureId },
    CreatureMarriage { date: WorldDate, creature_id: CreatureId, spouse_id: CreatureId },
    CreatureProfessionChange { date: WorldDate, creature_id: CreatureId, new_profession: Profession },
    ArtifactCreated { date: WorldDate, artifact: ItemId, creator: CreatureId },
    InheritedArtifact { date: WorldDate, creature_id: CreatureId, from: CreatureId, item: ItemId },
    BurriedWithPosessions { date: WorldDate, creature_id: CreatureId },
    ArtifactComission { date: WorldDate, creature_id: CreatureId, creator_id: CreatureId, item_id: ItemId },
    NewLeaderElected { date: WorldDate, unit_id: UnitId, creature_id: CreatureId },
}