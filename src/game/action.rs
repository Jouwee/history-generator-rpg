use std::collections::HashMap;

use crate::engine::{geometry::Coord2, Color};

use super::{actor::Actor, log::LogEntry};

#[derive(Hash, Eq, PartialEq)]
pub enum ActionEnum {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Attack
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

    pub fn try_use_on_target(&self, action: ActionEnum, actor: &mut dyn Actor, target: &mut dyn Actor) -> Result<Option<LogEntry>, ()> {
        if let Some(action) = self.map.get(&action) {
            if action.action.can_run_on_target(actor, target) && actor.ap().can_use(action.base_ap_cost) {
                let log = action.action.run_on_target(actor, target);
                actor.ap().consume(action.base_ap_cost);
                return Ok(log)
            }
        }
        Err(())
    }

    pub fn try_use_on_self(&self, action: ActionEnum, actor: &mut dyn Actor) -> Result<Option<LogEntry>, ()> {
        if let Some(action) = self.map.get(&action) {
            if action.action.can_run_on_self(actor) && actor.ap().can_use(action.base_ap_cost) {
                let log = action.action.run_on_self(actor);
                actor.ap().consume(action.base_ap_cost);
                return Ok(log)
            }
        }
        Err(())
    }
}


pub trait ActionTrait {
    fn final_ap_cost(&self) {}
    fn can_run_on_self(&self, _actor: &dyn Actor) -> bool { false }
    fn run_on_self(&self, _actor: &mut dyn Actor) -> Option<LogEntry> { None }
    fn can_run_on_target(&self, _actor: &dyn Actor, _target: &dyn Actor) -> bool { false }
    fn run_on_target(&self, _actor: &mut dyn Actor, _target: &mut dyn Actor) -> Option<LogEntry> { None }
}

pub struct AttackAction {}
impl ActionTrait for AttackAction {

    fn can_run_on_target(&self, _actor: &dyn Actor, _target: &dyn Actor) -> bool { true }
    fn run_on_target(&self, actor: &mut dyn Actor, target: &mut dyn Actor) -> Option<LogEntry> {
        let damage = actor.damage().resolve(&target.defence());
        target.hp().damage(damage);
        Some(LogEntry::new(format!("X attacks Y for {damage}"), Color::from_hex("eb9661")))
    }

}

pub struct MoveAction {
    direction: Coord2
}
impl ActionTrait for MoveAction {

    fn can_run_on_self(&self, _actor: &dyn Actor) -> bool { true }
    fn run_on_self(&self, actor: &mut dyn Actor) -> Option<LogEntry> {
        actor.set_xy(actor.xy() + self.direction);
        None
    }

}