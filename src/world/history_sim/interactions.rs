use crate::{commons::rng::Rng, world::{creature::{Creature, CreatureId}}};

pub(crate) fn simplified_interaction(creature_id: &CreatureId, creature: &mut Creature, other_creature_id: &CreatureId, other_creature: &mut Creature, rng: &mut Rng) {
    
    let chitchat_chance = 1.;
    let mut insult_chance = 0.1;
    let good_talk_chance = 0.1;
    let awkward_talk_chance = 0.2;

    let relationship_a = creature.relationship_find_mut_or_insert(creature_id, *other_creature_id, &other_creature);

    if relationship_a.rival_or_worse() {
        insult_chance *= 2.;
    }
    if relationship_a.friend_or_better() {
        insult_chance /= 2.;
    }

    let max = chitchat_chance + insult_chance + good_talk_chance + awkward_talk_chance;

    let f_interaction = rng.randf_range(0., max);
    if f_interaction < chitchat_chance {
        // println!("chit chat");
        relationship_a.add_opinion(1);
        let relationship_b = other_creature.relationship_find_mut_or_insert(other_creature_id, *creature_id, &creature);
        relationship_b.add_opinion(1);
    } else if f_interaction < chitchat_chance + good_talk_chance {
        // println!("good talk");
        relationship_a.add_opinion(5);
        let relationship_b = other_creature.relationship_find_mut_or_insert(other_creature_id, *creature_id, &creature);
        relationship_b.add_opinion(5);
    } else if f_interaction < chitchat_chance + good_talk_chance + awkward_talk_chance {
        // println!("awkward talk");
        relationship_a.add_opinion(-1);
        let relationship_b = other_creature.relationship_find_mut_or_insert(other_creature_id, *creature_id, &creature);
        relationship_b.add_opinion(-1);
    } else { // Insult
        // println!("insult");
        relationship_a.add_opinion(-1);
        let relationship_b = other_creature.relationship_find_mut_or_insert(other_creature_id, *creature_id, &creature);
        relationship_b.add_opinion(-5);
    }
}