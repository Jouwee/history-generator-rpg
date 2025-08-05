use crate::{game::{actor::actor::Actor, codex::{Quest, QuestObjective}}, resources::resources::Resources, world::{creature::{CauseOfDeath, CreatureGender, CreatureId}, date::WorldDate, item::{ArtworkScene, Item}, world::World}};

pub(crate) struct Writer<'a> {
    world: &'a World,
    resources: &'a Resources,
    final_text: String
}

impl<'a> Writer<'a> {

    pub(crate) fn new(world: &'a World, resources: &'a Resources) -> Self {
        Self { world, resources, final_text: String::new() }
    }

    pub(crate) fn take_text(&mut self) -> String {
        let text = self.final_text.clone();
        self.final_text = String::new();
        return text
    }

    pub(crate) fn describe_actor(&mut self, actor: &Actor) {
        let species = self.resources.species.get(&actor.species);
        self.add_text(&format!("A {}.", species.name));
    }

    pub(crate) fn describe_quest(&mut self, quest: &Quest) {
        match quest.objective {
            QuestObjective::KillCreature(creature_id) => {
                let creature = self.world.creatures.get(&creature_id);
                self.add_text(&format!("Kill {}", creature.name(&creature_id, self.world, self.resources)));
            }
        }
    }

    pub(crate) fn describe_item(&mut self, item: &Item) {

        let mut description = item.name.clone();

        if let Some(quality) = &item.quality {
            description = format!("{:?} {description}", quality.quality)
        }

        if let Some(material) = &item.material {
            let mut composition = Vec::new();
            
            let primary = self.resources.materials.get(&material.primary);
            composition.push(primary.name.clone());

            if let Some(secondary) = material.secondary {
                let primary = self.resources.materials.get(&secondary);
                composition.push(primary.name.clone());
            }

            if let Some(details) = material.details {
                let primary = self.resources.materials.get(&details);
                composition.push(primary.name.clone());
            }

            let composition = composition.join(", ");
            description = format!("{description} made of {composition}");
        }

        if let Some(scene) = &item.artwork_scene {
            match scene.scene {
                ArtworkScene::Bust { creature_id } => {
                    let creature = self.world.creatures.get(&creature_id);
                    description = format!("{description}. It depicts a bust of {}", creature.name(&creature_id, self.world, self.resources));
                },
                ArtworkScene::FullBody { creature_id, artifact_id } => {
                    let creature = self.world.creatures.get(&creature_id);
                    description = match artifact_id {
                        Some(artifact) => {
                            let artifact = self.world.artifacts.get(&artifact);
                            format!("{description}. It depicts a full-body image of {} holding {}", creature.name(&creature_id, self.world, self.resources), artifact.name(&self.resources.materials))
                        }
                        None => format!("{description}. It depicts a full-body image of {}", creature.name(&creature_id, self.world, self.resources))
                    };                    
                }
            }
        }

        self.add_text(&description);
    }

    pub(crate) fn describe_burial_place(&mut self, creature_id: &CreatureId) {
        self.add_text(&format!("A grave. The headstone reads:\n"));
        let creature = self.world.creatures.get(creature_id);
        let (death_date, death_reason) = creature.death.unwrap();
        self.add_text(&(creature.name(creature_id, self.world, self.resources) + "\n"));
        if creature.father != CreatureId::ancients() {
            let child_gendered = match &creature.gender {
                CreatureGender::Male => "Son",
                CreatureGender::Female => "Daugther",
            };
            self.add_text(&format!("{} of {} and {}\n", child_gendered, self.creature_name(&creature.father), self.creature_name(&creature.mother)));
        }
        self.add_text(&format!("Born in {}\n", format_date(&creature.birth)));
        self.add_text(&format!("Died in {} {}\n", format_date(&death_date), self.cause_of_death_description(&death_reason)));
    }

    pub(crate) fn chat_present_self(&mut self, actor: &Actor) {
        if let Some(creature_id) = actor.creature_id {
            let creature = self.world.creatures.get(&creature_id);
            let name = creature.name(&creature_id, self.world, self.resources);
            self.quote_actor(&format!("I'm {name}"), actor);
        } else {
            self.quote_actor("I'm nobody", actor);
        }
    }

    pub(crate) fn chat_explain_quest(&mut self, quest: &Quest, actor: &Actor) {
        match &quest.objective {
            QuestObjective::KillCreature(creature_id) => {
                let creature = self.world.creatures.get(creature_id);
                let species = self.resources.species.get(&creature.species);
                self.quote_actor(&format!("A {} has been terrorising us. I want you to go to it's lair and kill it. Here, I marked it on your map.", species.name), actor);
            }
        }
    }

    pub(crate) fn quote_actor(&mut self, sentence: &str, actor: &Actor) {
        // let species = self.resources.species.get(&actor.species);
        self.add_text(&format!("\n\"{sentence}\", he says"));
    }

    fn creature_name(&self, creature_id: &CreatureId) -> String {
        let creature = self.world.creatures.get(creature_id);
        return creature.name(creature_id, self.world, self.resources);
    }

    fn cause_of_death_description(&self, cause_of_death: &CauseOfDeath) -> String {
        match cause_of_death {
            CauseOfDeath::Disease => String::from("of a sudden illness"),
            CauseOfDeath::OldAge => String::from("peacefully in their sleep"),
            CauseOfDeath::Starvation => String::from("of malnutrition"),
            CauseOfDeath::KilledInBattle(killer_id, _) => format!("by the hand of {}", self.creature_name(killer_id))
        }
    }

    pub(crate) fn add_text(&mut self, text: &str) {
        if self.final_text.len() > 0 {
            self.final_text = self.final_text.clone() + " " + text;
        } else {
            self.final_text = String::from(text);
        }
    }
    
} 

fn format_date(date: &WorldDate) -> String {
    return format!("{}", date.year())
}