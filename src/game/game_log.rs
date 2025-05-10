use graphics::CharacterCache;

use crate::{commons::damage_model::DamageOutput, world::world::World, Actor, Color, GameContext, RenderContext, Resources};

use super::actor::actor::ActorType;

pub(crate) struct GameLog {
    entries: Vec<GameLogEntry>
}

impl GameLog {
    pub fn new() -> Self {
        GameLog {
            entries: Vec::new()
        }
    }

    pub fn log(&mut self, message: GameLogEntry) {
        self.entries.insert(0, message);
        // println!("{}", message.text);0
    }

    pub fn render(&mut self, ctx: &mut RenderContext, _game_ctx: &mut GameContext) {
        let last_entries = &self.entries[..10.min(self.entries.len())];
        let mut y = ctx.layout_rect[3] - 64.;
        for entry in last_entries.iter() {
            let mut x = 16.;
            for part in entry.parts.iter() {
                // Shadow effect
                ctx.text_small(part.text(), 5, [x, y + 1.], Color::from_hex("000000"));
                // Actual text
                ctx.text_small(part.text(), 5, [x, y], part.color());
                x += ctx.small_font.width(5, &part.text()).unwrap_or(0.);
            }
            y -= 8.;
        }
    }
}

pub(crate) struct GameLogEntry {
    parts: Vec<Part>
}

impl GameLogEntry {

    fn from_parts(parts: Vec<Part>) -> Self {
        return Self {
            parts
        }
    }

    pub(crate) fn damage(actor: &Actor, target: &Actor, damage: &DamageOutput, world: &World, resources: &Resources) -> Self {
        match damage {
            DamageOutput::Dodged => GameLogEntry::from_parts(vec!(
                Part::Actor(Self::actor_name(target, world, resources), target.actor_type),
                Part::Text(String::from(" doged ")),
                Part::Actor(Self::actor_name(actor, world, resources), actor.actor_type),
                Part::Text(String::from("'s attack")),
            )),
            DamageOutput::Hit(damage) => GameLogEntry::from_parts(vec!(
                Part::Actor(Self::actor_name(actor, world, resources), actor.actor_type),
                Part::Text(String::from(" hit ")),
                Part::Actor(Self::actor_name(target, world, resources), target.actor_type),
                Part::Text(String::from(" for ")),
                Part::Damage(format!("{:.2}", damage)),
                Part::Text(String::from(" damage")),
            )),
            DamageOutput::CriticalHit(damage) => GameLogEntry::from_parts(vec!(
                Part::Actor(Self::actor_name(actor, world, resources), actor.actor_type),
                Part::Text(String::from(" critically hit ")),
                Part::Actor(Self::actor_name(target, world, resources), target.actor_type),
                Part::Text(String::from(" for ")),
                Part::Damage(format!("{:.2}", damage)),
                Part::Text(String::from(" damage")),
            ))
        }
    }

    fn actor_name(actor: &Actor, world: &World, resources: &Resources) -> String {
        if let Some(creature_id) = &actor.creature_id {
            let creature = world.creatures.get(creature_id);
            return creature.name(creature_id, world, resources)
        }
        let species = resources.species.get(&actor.species);
        return species.name.clone();
    }

}

enum Part {
    Text(String),
    Damage(String),
    Actor(String, ActorType)
}

impl Part {

    fn text(&self) -> &str {
        match self {
            Self::Text(str) => &str,
            Self::Damage(str) => &str,
            Self::Actor(str, actor_type) => {
                match actor_type {
                    ActorType::Player => "player",
                    _ => &str,
                }
            },
        }   
    }

    fn color(&self) -> Color {
        match self {
            Self::Text(_) => Color::from_hex("ffffff"),
            Self::Damage(_) => Color::from_hex("ff0000"),
            Self::Actor(_, actor_type) => {
                match actor_type {
                    ActorType::Player => Color::from_hex("00ccff"),
                    _ => Color::from_hex("ff8800"),
                }
            },
        }   
    }

}