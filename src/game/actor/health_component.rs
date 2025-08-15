use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::commons::rng::Rng;

#[derive(Clone)]
pub(crate) struct HealthComponent {
    max_hp: f32,
    current_hp: f32,
    body_parts: HashMap<BodyPart, BodyPartCondition>
}

const NON_CRITICAL_HIT_BODY_PART_DAMAGE_MULT: f32 = 0.25;

impl HealthComponent {

    pub(crate) fn new(max_hp: f32) -> Self {
        let mut instance = Self {
            max_hp,
            current_hp: max_hp,
            body_parts: HashMap::new()
        };
        instance.body_parts.insert(BodyPart::Head, BodyPartCondition::new(max_hp * 0.25));
        instance.body_parts.insert(BodyPart::Torso, BodyPartCondition::new(max_hp * 0.40));
        instance.body_parts.insert(BodyPart::LeftArm, BodyPartCondition::new(max_hp * 0.30));
        instance.body_parts.insert(BodyPart::RightArm, BodyPartCondition::new(max_hp * 0.30));
        instance.body_parts.insert(BodyPart::LeftLeg, BodyPartCondition::new(max_hp * 0.30));
        instance.body_parts.insert(BodyPart::RightLeg, BodyPartCondition::new(max_hp * 0.30));
        return instance
    }

    pub(crate) fn health_points(&self) -> f32 {
        return self.current_hp;
    }

    pub(crate) fn max_health_points(&self) -> f32 {
        let mut overall_condition = (0., 0.);
        for condition in self.body_parts.values() {
            overall_condition.0 += condition.health;
            overall_condition.1 += condition.max_health;
        }
        return overall_condition.0 / overall_condition.1 * self.max_hp;
    }

    pub(crate) fn hit(&mut self, body_part: BodyPart, damage: f32) {
        self.current_hp = (self.current_hp - damage).max(0.);
        let body_part = self.body_parts.get_mut(&body_part).expect("Creature doesn't have bodypart");
        body_part.health = (body_part.health - (damage * NON_CRITICAL_HIT_BODY_PART_DAMAGE_MULT)).max(0.);
    }

    pub(crate) fn critical_hit(&mut self, body_part: BodyPart, damage: f32) {
        self.current_hp = (self.current_hp - damage).max(0.);
        let body_part = self.body_parts.get_mut(&body_part).expect("Creature doesn't have bodypart");
        body_part.health = (body_part.health - damage).max(0.);
    }

    pub(crate) fn recover_full(&mut self) {
        self.current_hp = self.max_hp;
        for (_body_part, condition) in self.body_parts.iter_mut() {
            condition.health = condition.max_health;
        }
    }

    pub(crate) fn recover_turn(&mut self) {
        self.current_hp = (self.current_hp + 0.1).min(self.max_health_points());
        for (_body_part, condition) in self.body_parts.iter_mut() {
            condition.health = (condition.health + 0.01).min(condition.max_health);
        }
    }

    pub(crate) fn body_part_condition(&self, body_part: &BodyPart) -> Option<&BodyPartCondition> {
        return self.body_parts.get(body_part)
    }

}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum BodyPart {
    Head,
    Torso,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
}

impl BodyPart {

    pub(crate) fn random(rng: &mut Rng) -> Self {
        match rng.randu_range(0, 6) {
            0 => Self::Head,
            1 => Self::Torso,
            2 => Self::LeftArm,
            3 => Self::RightArm,
            4 => Self::LeftLeg,
            _ => Self::RightLeg,
        }
    }

}

#[derive(Clone)]
pub(crate) struct BodyPartCondition {
    health: f32,
    max_health: f32,
}

impl BodyPartCondition {

    fn new(health: f32) -> Self {
        BodyPartCondition { health, max_health: health }
    }

    pub(crate) fn condition(&self) -> f32 {
        return self.health / self.max_health;
    }

}

#[cfg(test)]
mod test_health_component {
    use super::*;

    #[test]
    fn test_overall_health() {
        let mut health = HealthComponent::new(100.);
        assert_eq!(health.health_points(), 100.);
        assert_eq!(health.max_health_points(), 100.);

        health.hit(BodyPart::Torso, 5.);
        assert_eq!(health.health_points(), 95.);
        assert_eq!(health.max_health_points(), 99.324326);
        
    }

}