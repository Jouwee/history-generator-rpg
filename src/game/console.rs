use std::ops::ControlFlow;

use piston::Key;

use crate::{chunk_gen::{jigsaw_parser::JigsawParser, jigsaw_structure_generator::{JigsawPiece, JigsawPieceTile, JigsawSolver}, structure_filter::{NoopFilter, StructureFilter}}, commons::rng::Rng, engine::{geometry::Coord2, input::InputEvent, render::RenderContext, Color}, game::{actor::actor::Actor, chunk::Chunk}, resources::species::{Species, SpeciesId, SpeciesMap}, GameContext};

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

        ctx.rectangle_fill([ctx.layout_rect[0], ctx.layout_rect[1], ctx.layout_rect[2], 32.], Color::from_hex("00000080"));
        ctx.text(&format!("> {}", self.command), game_ctx.assets.font_standard(), [8, 8], &Color::from_hex("ffffffff"));
        ctx.text(&format!("  {}", self.output), game_ctx.assets.font_standard(), [8, 24], &Color::from_hex("ffffffff"));
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
                Key::Space => self.command = self.command.clone() + " ",
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

                // TODO(8QXbvyNV): Dupped code
                let mut solver = JigsawSolver::new(chunk.size.clone(), Rng::rand());
                let parser = JigsawParser::new();
                if let Ok(pools) = parser.parse_file("assets/structures/village.toml") {
                    for (name, pool) in pools {
                        solver.add_pool(&name, pool);
                    }
                }
                if let Ok(pools) = parser.parse_file("assets/structures/bandit_camp.toml") {
                    for (name, pool) in pools {
                        solver.add_pool(&name, pool);
                    }
                }
                if let Ok(pools) = parser.parse_file("assets/structures/wilderness.toml") {
                    for (name, pool) in pools {
                        solver.add_pool(&name, pool);
                    }
                }

                let structure = solver.solve_structure(structure, chunk.player().xy.clone(), &mut Rng::rand());
                if let Some(structure) = structure {
                    for (pos, piece) in structure.vec.iter() {
                        self.place_template(chunk, *pos, &piece);
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

                let actor = Actor::from_species(xy, &species_id, species);

                chunk.spawn(actor);

                return Result::Ok(format!("Spawned"));
            },
            Some(cmd) => return Result::Err(format!("Command {} not found", cmd))
        }
    }

    // TODO(8QXbvyNV): Dupped cope
    fn place_template(&mut self, chunk: &mut Chunk, origin: Coord2, template: &JigsawPiece) {
        self.place_template_filtered(chunk, origin, template, NoopFilter {});
    }

    // TODO(8QXbvyNV): Dupped cope
    fn place_template_filtered<F>(&mut self, chunk: &mut Chunk, origin: Coord2, template: &JigsawPiece, mut filter: F) where F: StructureFilter {
        for i in 0..template.size.area() {
            let x = origin.x as usize + i % template.size.x();
            let y = origin.y as usize + i / template.size.x();
            let mut tile = template.tiles.get(i).unwrap().clone();

            let filtered = filter.filter(Coord2::xy(x as i32, y as i32), &tile);
            if let Some(filtered) = filtered {
                tile = filtered;
            }

            match tile {
                JigsawPieceTile::Air => (),
                JigsawPieceTile::Empty => (),
                JigsawPieceTile::PathEndpoint => (),
                JigsawPieceTile::Connection(_) => chunk.map.ground_layer.set_tile(x, y, 4),
                JigsawPieceTile::Fixed { ground, object, statue_spot } => {
                    chunk.map.ground_layer.set_tile(x, y, ground);
                    if let Some(object) = object {
                        chunk.map.object_layer.set_tile(x, y, object)
                    }
                    if statue_spot {
                        // statue_spots.push(Coord2::xy(x as i32, y as i32))
                    }
                },
            }
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