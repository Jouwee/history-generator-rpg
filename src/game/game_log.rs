use crate::{world::world::World, Actor, Color, GameContext, RenderContext, Resources};

use super::actor::damage_resolver::DamageOutput;

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

    pub fn render(&mut self, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        let last_entries = &self.entries[..10.min(self.entries.len())];
        let mut y = ctx.layout_rect[3] as i32 - 64;
        for entry in last_entries.iter() {
            let mut x = 16;
            for part in entry.parts.iter() {
                // Shadow effect
                ctx.text(part.text(), game_ctx.assets.font_standard(), [x, y + 1], &Color::from_hex("000000"));
                // Actual text
                ctx.text(part.text(), game_ctx.assets.font_standard(), [x, y], &part.color());
                x += game_ctx.assets.font_standard().width(&part.text()) as i32;
            }
            y -= 10;
        }
    }
}

pub(crate) struct GameLogEntry {
    parts: Vec<GameLogPart>
}

impl GameLogEntry {

    pub(crate) fn from_parts(parts: Vec<GameLogPart>) -> Self {
        return Self {
            parts
        }
    }

    pub(crate) fn damage(target: &Actor, is_player: bool, damage: &DamageOutput, world: &World, resources: &Resources) -> Self {
        match damage {
            DamageOutput::Dodged => GameLogEntry::from_parts(vec!(
                GameLogPart::Actor(Self::actor_name(target, world, resources), is_player),
                GameLogPart::Text(String::from(" dodged the attack")),
            )),
            DamageOutput::Hit(damage) => GameLogEntry::from_parts(vec!(
                GameLogPart::Actor(Self::actor_name(target, world, resources), is_player),
                GameLogPart::Text(String::from(" takes ")),
                GameLogPart::Damage(format!("{:.2}", damage)),
                GameLogPart::Text(String::from(" damage")),
            )),
            DamageOutput::CriticalHit(damage) => GameLogEntry::from_parts(vec!(
                GameLogPart::Actor(Self::actor_name(target, world, resources), is_player),
                GameLogPart::Text(String::from(" takes ")),
                GameLogPart::Damage(format!("{:.2}", damage)),
                GameLogPart::Text(String::from(" damage (crit)")),
            ))
        }
    }

    pub(crate) fn actor_name(actor: &Actor, world: &World, resources: &Resources) -> String {
        if let Some(creature_id) = &actor.creature_id {
            let creature = world.creatures.get(creature_id);
            return creature.name(creature_id, world, resources)
        }
        let species = resources.species.get(&actor.species);
        return species.name.clone();
    }

}

pub(crate) enum GameLogPart {
    Text(String),
    Damage(String),
    Actor(String, bool)
}

impl GameLogPart {

    fn text(&self) -> &str {
        match self {
            Self::Text(str) => &str,
            Self::Damage(str) => &str,
            Self::Actor(str, is_player) => {
                match is_player {
                    true => "player",
                    false => &str,
                }
            },
        }   
    }

    fn color(&self) -> Color {
        match self {
            Self::Text(_) => Color::from_hex("ffffff"),
            Self::Damage(_) => Color::from_hex("ff0000"),
            Self::Actor(_, is_player) => {
                match is_player {
                    true => Color::from_hex("00ccff"),
                    false => Color::from_hex("ff8800"),
                }
            },
        }   
    }

}