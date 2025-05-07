use std::collections::HashMap;

use crate::commons::rng::Rng;

#[derive(Clone)]
pub(crate) struct HealthComponent {
    current_hp: f32,
    body_parts: HashMap<BodyPart, BodyPartCondition>
}

const NON_CRITICAL_HIT_BODY_PART_DAMAGE_MULT: f32 = 0.25;

impl HealthComponent {

    pub(crate) fn new() -> Self {
        let mut instance = Self {
            current_hp: 100.,
            body_parts: HashMap::new()
        };
        instance.body_parts.insert(BodyPart::Head, BodyPartCondition::new(25.));
        instance.body_parts.insert(BodyPart::Torso, BodyPartCondition::new(40.));
        instance.body_parts.insert(BodyPart::LeftArm, BodyPartCondition::new(30.));
        instance.body_parts.insert(BodyPart::RightArm, BodyPartCondition::new(30.));
        instance.body_parts.insert(BodyPart::LeftLeg, BodyPartCondition::new(30.));
        instance.body_parts.insert(BodyPart::RightLeg, BodyPartCondition::new(30.));
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
        return overall_condition.0 / overall_condition.1 * 100.;
    }

    pub(crate) fn hit(&mut self, body_part: BodyPart, damage: f32) {
        self.current_hp -= damage;
        let body_part = self.body_parts.get_mut(&body_part).expect("Creature doesn't have bodypart");
        body_part.health -= damage * NON_CRITICAL_HIT_BODY_PART_DAMAGE_MULT;
    }

    pub(crate) fn critical_hit(&mut self, body_part: BodyPart, damage: f32) {
        self.current_hp -= damage;
        let body_part = self.body_parts.get_mut(&body_part).expect("Creature doesn't have bodypart");
        body_part.health -= damage;
    }

}

#[derive(Clone, Hash, PartialEq, Eq)]
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
struct BodyPartCondition {
    health: f32,
    max_health: f32,
}

impl BodyPartCondition {

    fn new(health: f32) -> Self {
        BodyPartCondition { health, max_health: health }
    }

}

#[cfg(test)]
mod test_health_component {
    use super::*;

    #[test]
    fn test_overall_health() {
        let mut health = HealthComponent::new();
        assert_eq!(health.health_points(), 100.);
        assert_eq!(health.max_health_points(), 100.);

        health.hit(BodyPart::Torso, 5.);
        assert_eq!(health.health_points(), 95.);
        assert_eq!(health.max_health_points(), 97.2973);
        
        
    }

}