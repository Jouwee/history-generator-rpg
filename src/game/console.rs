use std::ops::ControlFlow;

use piston::Key;

use crate::{chunk_gen::chunk_generator::ChunkGenerator, commons::rng::Rng, engine::{geometry::Coord2, input::InputEvent, render::RenderContext, COLOR_BLACK, COLOR_WHITE}, game::{actor::actor::Actor, chunk::Chunk}, resources::species::{SpeciesId, SpeciesMap}, GameContext};

pub(crate) struct Console {
    visible: bool,
    output: String,
    command: String
}

impl Console {

    pub(crate) fn new() -> Self {
        Self {
            visible: false,
            output: String::new(),
            command: String::new()
        }
    }

    pub(crate) fn render(&mut self, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        if !self.visible {
            return
        }

        ctx.rectangle_fill([ctx.layout_rect[0], ctx.layout_rect[1], ctx.layout_rect[2], 32.], COLOR_BLACK.alpha(0.5));
        ctx.text(&format!("> {}", self.command), game_ctx.assets.font_standard(), [8, 8], &COLOR_WHITE);
        ctx.text(&format!("  {}", self.output), game_ctx.assets.font_standard(), [8, 24], &COLOR_WHITE);
    }

    pub(crate) fn input(&mut self, chunk: &mut Chunk, evt: &InputEvent, ctx: &mut GameContext) -> ControlFlow<()> {
        if let InputEvent::Key { key: Key::Quote } = evt {
            self.visible = !self.visible;
            self.command = String::new();
            return ControlFlow::Continue(());
        }
        if !self.visible {
            return ControlFlow::Continue(())
        }

        if let InputEvent::Key { key } = evt {
            match key {
                Key::A => self.command = self.command.clone() + "a",
                Key::B => self.command = self.command.clone() + "b",
                Key::C => self.command = self.command.clone() + "c",
                Key::D => self.command = self.command.clone() + "d",
                Key::E => self.command = self.command.clone() + "e",
                Key::F => self.command = self.command.clone() + "f",
                Key::G => self.command = self.command.clone() + "g",
                Key::H => self.command = self.command.clone() + "h",
                Key::I => self.command = self.command.clone() + "i",
                Key::J => self.command = self.command.clone() + "j",
                Key::K => self.command = self.command.clone() + "k",
                Key::L => self.command = self.command.clone() + "l",
                Key::M => self.command = self.command.clone() + "m",
                Key::N => self.command = self.command.clone() + "n",
                Key::O => self.command = self.command.clone() + "o",
                Key::P => self.command = self.command.clone() + "p",
                Key::Q => self.command = self.command.clone() + "q",
                Key::R => self.command = self.command.clone() + "r",
                Key::S => self.command = self.command.clone() + "s",
                Key::T => self.command = self.command.clone() + "t",
                Key::U => self.command = self.command.clone() + "u",
                Key::V => self.command = self.command.clone() + "v",
                Key::W => self.command = self.command.clone() + "w",
                Key::X => self.command = self.command.clone() + "x",
                Key::Y => self.command = self.command.clone() + "y",
                Key::Z => self.command = self.command.clone() + "z",
                Key::NumPad0 | Key::D0 => self.command = self.command.clone() + "0",
                Key::NumPad1 | Key::D1 => self.command = self.command.clone() + "1",
                Key::NumPad2 | Key::D2 => self.command = self.command.clone() + "2",
                Key::NumPad3 | Key::D3 => self.command = self.command.clone() + "3",
                Key::NumPad4 | Key::D4 => self.command = self.command.clone() + "4",
                Key::NumPad5 | Key::D5 => self.command = self.command.clone() + "5",
                Key::NumPad6 | Key::D6 => self.command = self.command.clone() + "6",
                Key::NumPad7 | Key::D7 => self.command = self.command.clone() + "7",
                Key::NumPad8 | Key::D8 => self.command = self.command.clone() + "8",
                Key::NumPad9 | Key::D9 => self.command = self.command.clone() + "9",
                Key::Space => self.command = self.command.clone() + " ",
                Key::Comma | Key::NumPadComma => self.command = self.command.clone() + ",",
                Key::Semicolon => self.command = self.command.clone() + ":",
                // TODO:
                Key::Minus => self.command = self.command.clone() + "_",
                Key::Underscore => self.command = self.command.clone() + "_",
                Key::At => self.command = self.command.clone() + "@",
                Key::Slash => self.command = self.command.clone() + "/",
                Key::Backspace => self.command = self.command[0..self.command.len()-1].to_string(),
                Key::Return => {
                    match self.run_command(chunk, ctx) {
                        Ok(str) => self.output = str,
                        Err(str) => self.output = str,
                    }
                    self.command = String::new();
                }
                _ => ()
            }
            return ControlFlow::Break(())
        }
        
        return ControlFlow::Continue(())
    }

    fn run_command(&mut self, chunk: &mut Chunk, ctx: &mut GameContext) -> Result<String, String> {
        let mut parts = self.command.split(' ');
        let command = parts.next();
        match command {
            None => return Result::Err(format!("Type a command")),
            Some("/generate") => {
                let structure = parts.next().ok_or("Param 1 should be the structure name")?;

                let pos = chunk.player().xy.clone();
                let mut generator = ChunkGenerator::new(chunk, Rng::rand());

                let mut solver = generator.get_jigsaw_solver();
                let structure = solver.solve_structure(structure, pos, &mut Rng::rand());
                if let Some(structure) = structure {
                    for (pos, piece) in structure.vec.iter() {
                        generator.place_template(*pos, &piece, &ctx.resources);
                    }
                    return Result::Ok(format!("Generated"));
                } else {
                    return Result::Err(format!("Error while generating"));
                }

                
            },
            Some("/spawn") => {
                let species = parts.next().ok_or("Param 1 should be the species")?;
                let species_id = parse_species(species, &ctx.resources.species)?;
                let species = ctx.resources.species.get(&species_id);

                //let position = parts.next().ok_or("Param 2 should be the position")?;

                let xy = chunk.player().xy.clone() + Coord2::xy(8, 0);

                let actor = Actor::from_species(xy, &species_id, species, chunk.ai_groups.next_group());

                chunk.spawn(actor);

                return Result::Ok(format!("Spawned"));
            },
            Some("/tp") => {
                let coords = parts.next().ok_or("Param 1 should be the coords")?;
                let coords = parse_coords(coords)?;

                chunk.player_mut().xy = coords;

                return Result::Ok(format!("Spawned"));
            },
            Some(cmd) => return Result::Err(format!("Command {} not found", cmd))
        }
    }

}

fn parse_species(string: &str, species: &SpeciesMap) -> Result<SpeciesId, String> {
    let mut string = String::from(string);
    if !string.starts_with("species:") {
        string = String::from("species:") + &string;
    }
    Ok(species.id_of(&string))
}

fn parse_coords(string: &str) -> Result<Coord2, String> {
    let mut parts = string.split(",");
    let x = parts.next().ok_or("missing x")?;
    let x = x.parse::<i32>().ok().ok_or("x must be a number")?;
    let y = parts.next().ok_or("missing y")?;
    let y = y.parse::<i32>().ok().ok_or("y must be a number")?;
    return Ok(Coord2::xy(x, y))
}