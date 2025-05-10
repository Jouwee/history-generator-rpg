use crate::Actor;

use super::health_component::BodyPart;

pub(crate) struct ActorStats<'a> {
    actor: &'a Actor
}

impl<'a> ActorStats<'a> {

    pub(crate) fn new(actor: &'a Actor) -> Self {
        return Self { actor }
    }

    pub(crate) fn walk_ap_multiplier(&self) -> f32 {
        let left_leg = self.actor.hp.body_part_condition(&BodyPart::LeftLeg);
        let right_leg = self.actor.hp.body_part_condition(&BodyPart::RightLeg);
        match (left_leg, right_leg) {
            (Some(left_leg), Some(right_leg)) => {
                let avg_condition = (left_leg.condition() + right_leg.condition()) / 2.;
                return 1. + (1. - avg_condition);
            },
            _ => panic!("Body part not found"),
        }
    }

    pub(crate) fn dodge_chance(&self) -> f32 {
        return self.actor.attributes.agility as f32 * 0.01
    }

    pub(crate) fn enemy_dodge_bonus(&self) -> f32 {
        let left_arm = self.actor.hp.body_part_condition(&BodyPart::LeftArm);
        let right_arm = self.actor.hp.body_part_condition(&BodyPart::RightArm);
        match (left_arm, right_arm) {
            (Some(left_arm), Some(right_arm)) => {
                let avg_condition = (left_arm.condition() + right_arm.condition()) / 2.;
                return (1. - avg_condition) * 0.2;
            },
            _ => panic!("Body part not found"),
        }
    }

    pub(crate) fn critical_hit_chance(&self) -> f32 {
        return 0.05
    }

    pub(crate) fn critical_hit_multiplier(&self) -> f32 {
        return 2.
    }

}

