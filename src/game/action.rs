use std::collections::HashMap;

use crate::{commons::damage_model::DamageComponent, engine::{geometry::Coord2, Color}, world::item::Item};

use super::{actor::Actor, chunk::ChunkMap, log::LogEntry};

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum ActionEnum {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    UnarmedAttack,
    Attack,
    Talk,
    PickUp,
    Sleep
}

impl ActionEnum {

    pub fn name(&self) -> &str {
        match self {
            Self::MoveLeft => "MoveLeft",
            Self::MoveRight => "MoveRight",
            Self::MoveUp => "MoveUp",
            Self::MoveDown => "MoveDown",
            Self::UnarmedAttack => "UnarmedAttack",
            Self::Attack => "Attack",
            Self::Talk => "Talk",
            Self::PickUp => "PickUp",
            Self::Sleep => "Sleep"
        }
    }

}

pub struct ActionDefinition {
    base_ap_cost: u16,
    action: Box<dyn ActionTrait>,
}

pub struct ActionMap {
    map: HashMap<ActionEnum, ActionDefinition>
}

impl Default for ActionMap {
    fn default() -> Self {
        let mut map = ActionMap { map: HashMap::new() };
        map.register(ActionEnum::UnarmedAttack, 40, Box::new(UnarmedAttack {}));
        map.register(ActionEnum::Attack, 40, Box::new(AttackAction {}));
        map.register(ActionEnum::Talk, 0, Box::new(TalkAction {}));
        map.register(ActionEnum::PickUp, 20, Box::new(PickUpAction {}));
        map.register(ActionEnum::MoveLeft, 20, Box::new(MoveAction { direction: Coord2::xy(-1, 0) }));
        map.register(ActionEnum::MoveRight, 20, Box::new(MoveAction { direction: Coord2::xy(1, 0) }));
        map.register(ActionEnum::MoveUp, 20, Box::new(MoveAction { direction: Coord2::xy(0, -1) }));
        map.register(ActionEnum::MoveDown, 20, Box::new(MoveAction { direction: Coord2::xy(0, 1) }));
        map.register(ActionEnum::Sleep, 0, Box::new(SleepAction {}));
        return map
    }
}

impl ActionMap {

    pub fn register(&mut self, action: ActionEnum, ap_cost: u16, action_impl: Box<dyn ActionTrait>) {
        self.map.insert(action, ActionDefinition {
            base_ap_cost: ap_cost,
            action: action_impl
        });
    }

    pub fn try_use_on_target(&self, action: ActionEnum, actor: &mut Actor, target: &mut Actor) -> Result<Option<LogEntry>, ()> {
        if let Some(action) = self.map.get(&action) {
            if action.action.can_run_on_target(actor, target) && actor.ap.can_use(action.base_ap_cost) {
                let log = action.action.run_on_target(actor, target);
                actor.ap.consume(action.base_ap_cost);
                return Ok(log)
            }
        }
        Err(())
    }

    pub fn try_use_on_item(&self, action: ActionEnum, actor: &mut Actor, item: &mut Item) -> Result<Option<LogEntry>, ()> {
        if let Some(action) = self.map.get(&action) {
            if action.action.can_run_on_item(actor, item) && actor.ap.can_use(action.base_ap_cost) {
                let log = action.action.run_on_item(actor, item);
                actor.ap.consume(action.base_ap_cost);
                return Ok(log)
            }
        }
        Err(())
    }

    pub fn try_use_on_self(&self, action: ActionEnum, actor: &mut Actor) -> Result<Option<LogEntry>, ()> {
        if let Some(action) = self.map.get(&action) {
            if action.action.can_run_on_self(actor) && actor.ap.can_use(action.base_ap_cost) {
                let log = action.action.run_on_self(actor);
                actor.ap.consume(action.base_ap_cost);
                return Ok(log)
            }
        }
        Err(())
    }

    pub fn try_use_on_tile(&self, action: ActionEnum, actor: &mut Actor, chunk: &mut ChunkMap, pos: &Coord2) -> Result<Option<LogEntry>, ()> {
        if let Some(action) = self.map.get(&action) {
            if action.action.can_run_on_tile(actor, chunk, pos) && actor.ap.can_use(action.base_ap_cost) {
                let log = action.action.run_on_tile(actor, chunk, pos);
                actor.ap.consume(action.base_ap_cost);
                return Ok(log)
            }
        }
        Err(())
    }

}


pub trait ActionTrait {
    fn final_ap_cost(&self) {}
    fn can_run_on_self(&self, _actor: &Actor) -> bool { false }
    fn run_on_self(&self, _actor: &mut Actor) -> Option<LogEntry> { None }
    fn can_run_on_target(&self, _actor: &Actor, _target: &Actor) -> bool { false }
    fn run_on_target(&self, _actor: &mut Actor, _target: &mut Actor) -> Option<LogEntry> { None }
    fn can_run_on_item(&self, _actor: &Actor, _target: &Item) -> bool { false }
    fn run_on_item(&self, _actor: &mut Actor, _target: &mut Item) -> Option<LogEntry> { None }
    fn can_run_on_tile(&self, _actor: &Actor, _chunk: &ChunkMap, _pos: &Coord2) -> bool { false }
    fn run_on_tile(&self, _actor: &mut Actor, _chunk: &mut ChunkMap, _pos: &Coord2) -> Option<LogEntry> { None }
}

pub struct UnarmedAttack {}
impl ActionTrait for UnarmedAttack {

    fn can_run_on_target(&self, _actor: &Actor, _target: &Actor) -> bool { true }
    fn run_on_target(&self, actor: &mut Actor, target: &mut Actor) -> Option<LogEntry> {
        let str_mult = actor.attributes.strength_attack_damage_mult();
        let damage_model = DamageComponent::new(0., 0., 1.).multiply(str_mult);
        let damage = damage_model.resolve(&target.defence);
        target.hp.damage(damage);
        Some(LogEntry::new(format!("X attacks Y for {damage}"), Color::from_hex("eb9661")))
    }

}

pub struct AttackAction {}
impl ActionTrait for AttackAction {

    fn can_run_on_target(&self, _actor: &Actor, _target: &Actor) -> bool { true }
    fn run_on_target(&self, actor: &mut Actor, target: &mut Actor) -> Option<LogEntry> {
        let damage_model;
        let str_mult = actor.attributes.strength_attack_damage_mult();
        if let Some(item) = actor.inventory.equipped() {
            damage_model = item.damage_model().multiply(str_mult);
        } else {
            // TODO: Maybe if the creature bites instead of punching, this should change
            damage_model = DamageComponent::new(0., 0., 1.).multiply(str_mult);
        }
        let damage = damage_model.resolve(&target.defence);
        target.hp.damage(damage);
        Some(LogEntry::new(format!("X attacks Y for {damage}"), Color::from_hex("eb9661")))
    }

}

pub struct TalkAction {}
impl ActionTrait for TalkAction {

    fn can_run_on_target(&self, _actor: &Actor, _target: &Actor) -> bool { true }
    fn run_on_target(&self, _actor: &mut Actor, _target: &mut Actor) -> Option<LogEntry> {
        Some(LogEntry::new(format!("Hello!"), Color::from_hex("eb9661")))
    }

}

pub struct PickUpAction {}
impl ActionTrait for PickUpAction {

    fn can_run_on_item(&self, _actor: &Actor, _item: &Item) -> bool { true }
    fn run_on_item(&self, actor: &mut Actor, item: &mut Item) -> Option<LogEntry> {
        actor.inventory.add(item.clone());
        Some(LogEntry::new(format!("You picked up ..."), Color::from_hex("eb9661")))
    }

}

pub struct MoveAction {
    direction: Coord2
}
impl ActionTrait for MoveAction {

    fn can_run_on_tile(&self, _actor: &Actor, chunk: &ChunkMap, pos: &Coord2) -> bool {
        let pos = *pos + self.direction;
        return !chunk.blocks_movement(pos);
    }
    fn run_on_tile(&self, actor: &mut Actor, _chunk: &mut ChunkMap, _pos: &Coord2) -> Option<LogEntry> {
        actor.xy = actor.xy + self.direction;
        None
    }

}

pub struct SleepAction {
}
impl ActionTrait for SleepAction {

    fn can_run_on_tile(&self, _actor: &Actor, chunk: &ChunkMap, pos: &Coord2) -> bool {
        return chunk.get_object_idx(*pos) == 3
    }
    fn run_on_tile(&self, actor: &mut Actor, _chunk: &mut ChunkMap, _pos: &Coord2) -> Option<LogEntry> {
        actor.hp.refill();
        Some(LogEntry { string: String::from("You rest and recover health"), color: Color::from_hex("44702d") })
    }

}

