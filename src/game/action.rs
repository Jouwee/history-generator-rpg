use std::collections::HashMap;

use crate::{commons::damage_model::DamageComponent, engine::{geometry::Coord2, Color}, world::item::Item};

use super::{actor::Actor, log::LogEntry};

#[derive(Hash, Eq, PartialEq)]
pub enum ActionEnum {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Attack,
    Talk,
    PickUp,
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
        map.register(ActionEnum::Attack, 40, Box::new(AttackAction {}));
        map.register(ActionEnum::Talk, 0, Box::new(TalkAction {}));
        map.register(ActionEnum::PickUp, 20, Box::new(PickUpAction {}));
        map.register(ActionEnum::MoveLeft, 20, Box::new(MoveAction { direction: Coord2::xy(-1, 0) }));
        map.register(ActionEnum::MoveRight, 20, Box::new(MoveAction { direction: Coord2::xy(1, 0) }));
        map.register(ActionEnum::MoveUp, 20, Box::new(MoveAction { direction: Coord2::xy(0, -1) }));
        map.register(ActionEnum::MoveDown, 20, Box::new(MoveAction { direction: Coord2::xy(0, 1) }));
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
}


pub trait ActionTrait {
    fn final_ap_cost(&self) {}
    fn can_run_on_self(&self, _actor: &Actor) -> bool { false }
    fn run_on_self(&self, _actor: &mut Actor) -> Option<LogEntry> { None }
    fn can_run_on_target(&self, _actor: &Actor, _target: &Actor) -> bool { false }
    fn run_on_target(&self, _actor: &mut Actor, _target: &mut Actor) -> Option<LogEntry> { None }
    fn can_run_on_item(&self, _actor: &Actor, _target: &Item) -> bool { false }
    fn run_on_item(&self, _actor: &mut Actor, _target: &mut Item) -> Option<LogEntry> { None }
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

    fn can_run_on_self(&self, _actor: &Actor) -> bool { true }
    fn run_on_self(&self, actor: &mut Actor) -> Option<LogEntry> {
        actor.xy = actor.xy + self.direction;
        None
    }

}