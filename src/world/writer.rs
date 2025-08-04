use crate::{game::actor::actor::Actor, resources::resources::Resources, world::{creature::{self, CreatureId}, date::WorldDate, item::{ArtworkScene, Item}, world::World}};

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
        let creature = self.world.creatures.get(creature_id);
        let (death_date, death_reason) = creature.death.unwrap();
        self.add_text(&format!("A grave.\nThe headstone reads: {}, {:?}.\nBirthed {}, dead {}", creature.name(creature_id, self.world, self.resources), death_reason, format_date(&creature.birth), format_date(&death_date)));
    }

    fn add_text(&mut self, text: &str) {
        if self.final_text.len() > 0 {
            self.final_text = self.final_text.clone() + " " + text;
        } else {
            self.final_text = String::from(text);
        }
    }
    
} 

fn format_date(date: &WorldDate) -> String {
    return format!("{}-{}-{}", date.year(), date.month(), date.day())
}