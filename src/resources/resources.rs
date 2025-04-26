use crate::{commons::{damage_model::DamageComponent, resource_map::ResourceMap}, engine::{audio::SoundEffect, geometry::Coord2, Color}, game::action::{Action, ActionId, ActionType, Affliction, AfflictionChance, DamageType, Infliction}, world::{attributes::Attributes, material::{Material, MaterialId}, species::{Species, SpeciesApearance, SpeciesId, SpeciesIntelligence}}};

use super::tile::{Tile, TileId};

pub(crate) type Actions = ResourceMap<ActionId, Action>;
pub(crate) type Materials = ResourceMap<MaterialId, Material>;
pub(crate) type SpeciesMap = ResourceMap<SpeciesId, Species>;

#[derive(Clone)]
pub(crate) struct Resources {
    pub(crate) actions: Actions,
    pub(crate) materials: Materials,
    pub(crate) species: SpeciesMap,
    pub(crate) tiles: ResourceMap<TileId, Tile>
}

impl Resources {

    pub(crate) fn new() -> Resources {
        Resources {
            actions: ResourceMap::new(),
            materials: ResourceMap::new(),
            species: ResourceMap::new(),
            tiles: ResourceMap::new(),
        }
    }

    pub(crate) fn load(&mut self) {
        self.load_materials();
        self.load_actions();
        self.load_species();
        self.load_tiles();
    }

    pub(crate) fn load_materials(&mut self) {
        self.materials.add("mat:steel", Material::new_metal("steel"));
        let mut bronze = Material::new_metal("bronze");
        bronze.color_pallete = [Color::from_hex("a57855"), Color::from_hex("de9f47"), Color::from_hex("fdd179"), Color::from_hex("fee1b8")];
        self.materials.add("mat:bronze", bronze);
        self.materials.add("mat:birch", Material::new_wood("birch"));
        self.materials.add("mat:oak", Material::new_wood("oak"));
        self.materials.add("mat:bone_leshen", Material::new_bone("leshen bone"));
        self.materials.add("mat:bone_fiend", Material::new_bone("fiend bone"));
        let mut copper = Material::new_metal("copper");
        copper.color_pallete = [Color::from_hex("593e47"), Color::from_hex("b55945"), Color::from_hex("de9f47"), Color::from_hex("f2b888")];
        self.materials.add("mat:copper", copper);
    }

    pub(crate) fn load_actions(&mut self) {
        self.actions.add("act:sword:slash", Action {
            name: String::from("Slash"),
            description: String::from("A slashing strike"),
            icon: String::from("gui/icons/actions/slashing_cut.png"),
            sound_effect: Some(SoundEffect::new(vec!("sfx/sword_1.mp3", "sfx/sword_2.mp3", "sfx/sword_3.mp3"))),
            ap_cost: 40,
            action_type: ActionType::Targeted {
                damage: Some(DamageType::FromWeapon(DamageComponent::new(1., 0., 0.))),
                inflicts: None
            }
        });
        self.actions.add("act:sword:bleeding_cut", Action {
            name: String::from("Bleeding Cut"),
            description: String::from("A deep cut that causes bleeding"),
            icon: String::from("gui/icons/actions/bleeding_cut.png"),
            sound_effect: Some(SoundEffect::new(vec!("sfx/sword_1.mp3", "sfx/sword_2.mp3", "sfx/sword_3.mp3"))),
            ap_cost: 60,
            action_type: ActionType::Targeted {
                damage: Some(DamageType::FromWeapon(DamageComponent::new(0.8, 0.0, 0.))),
                inflicts: Some(Infliction {
                    chance: AfflictionChance::OnHit,
                    affliction: Affliction::Bleeding { duration: 5 }
                })
            }
        });
        self.actions.add("act:mace:smash", Action {
            name: String::from("Smash"),
            description: String::from("A heavy smash"),
            icon: String::from("gui/icons/actions/mace_smash.png"),
            sound_effect: Some(SoundEffect::new(vec!("sfx/punch_1.mp3", "sfx/punch_2.mp3"))),
            ap_cost: 40,
            action_type: ActionType::Targeted {
                damage: Some(DamageType::FromWeapon(DamageComponent::new(0., 0., 1.))),
                inflicts: None
            }
        });
        self.actions.add("act:mace:concussive_strike", Action {
            name: String::from("Concussive Strike"),
            description: String::from("An aimed hit at the head"),
            icon: String::from("gui/icons/actions/concussive_strike.png"),
            sound_effect: Some(SoundEffect::new(vec!("sfx/punch_1.mp3", "sfx/punch_2.mp3"))),
            ap_cost: 60,
            action_type: ActionType::Targeted {
                damage: Some(DamageType::FromWeapon(DamageComponent::new(1.0, 0.0, 0.))),
                inflicts: Some(Infliction {
                    chance: AfflictionChance::OnHit,
                    affliction: Affliction::Stunned { duration: 1 }
                })
            }
        });
        self.actions.add("act:punch", Action {
            name: String::from("Punch"),
            description: String::from("A good ol' punch"),
            icon: String::from("gui/icons/actions/unarmed_attack.png"),
            sound_effect: Some(SoundEffect::new(vec!("sfx/punch_1.mp3", "sfx/punch_2.mp3"))),
            ap_cost: 40,
            action_type: ActionType::Targeted {
                damage: Some(DamageType::Fixed(DamageComponent::new(0., 0., 1.))),
                inflicts: None
            }
        });
        self.actions.add("act:spider_bite", Action {
            name: String::from("Bite"),
            description: String::from("A spider bite"),
            icon: String::from("missing.png"),
            sound_effect: Some(SoundEffect::new(vec!("sfx/monster_bite.mp3"))),
            ap_cost: 40,
            action_type: ActionType::Targeted {
                damage: Some(DamageType::Fixed(DamageComponent::new(0., 1., 0.))),
                inflicts: Some(Infliction {
                    chance: AfflictionChance::OnHit,
                    affliction: Affliction::Poisoned { duration: 10 }
                })
            }
        });
        self.actions.add("act:talk", Action {
            name: String::from("Talk"),
            description: String::from("Talk with a friendly NPC"),
            icon: String::from("gui/icons/actions/talk.png"),
            sound_effect: None,
            ap_cost: 0,
            action_type: ActionType::Talk
        });
        self.actions.add("act:inspect", Action {
            name: String::from("Inspect"),
            description: String::from("Inspect something"),
            icon: String::from("gui/icons/actions/inspect.png"),
            sound_effect: None,
            ap_cost: 0,
            action_type: ActionType::Inspect
        });
        self.actions.add("act:dig", Action {
            name: String::from("Dig"),
            description: String::from("Dig the ground"),
            icon: String::from("gui/icons/actions/dig.png"),
            sound_effect: None,
            ap_cost: 0,
            action_type: ActionType::Dig
        });
        self.actions.add("act:pickup", Action {
            name: String::from("Pick Up"),
            description: String::from("Pick up something from the ground"),
            icon: String::from("gui/icons/actions/pickup.png"),
            sound_effect: None,
            ap_cost: 20,
            action_type: ActionType::PickUp
        });
        self.actions.add("act:sleep", Action {
            name: String::from("Sleep"),
            description: String::from("Sleep in a bed"),
            icon: String::from("gui/icons/actions/sleep.png"),
            sound_effect: None,
            ap_cost: 0,
            action_type: ActionType::Sleep
        });
        self.actions.add("act:move_left", Action {
            name: String::from("Move Left"),
            description: String::from("Move"),
            icon: String::from("gui/icons/actions/sleep.png"),
            sound_effect: None,
            ap_cost: 20,
            action_type: ActionType::Move { offset: Coord2::xy(-1, 0) }
        });
        self.actions.add("act:move_right", Action {
            name: String::from("Move Right"),
            description: String::from("Move"),
            icon: String::from("gui/icons/actions/sleep.png"),
            sound_effect: None,
            ap_cost: 20,
            action_type: ActionType::Move { offset: Coord2::xy(1, 0) }
        });
        self.actions.add("act:move_up", Action {
            name: String::from("Move Up"),
            description: String::from("Move"),
            icon: String::from("gui/icons/actions/sleep.png"),
            sound_effect: None,
            ap_cost: 20,
            action_type: ActionType::Move { offset: Coord2::xy(0, -1) }
        });
        self.actions.add("act:move_down", Action {
            name: String::from("Move Down"),
            description: String::from("Move"),
            icon: String::from("gui/icons/actions/sleep.png"),
            sound_effect: None,
            ap_cost: 20,
            action_type: ActionType::Move { offset: Coord2::xy(0, 1) }
        });
    }

    pub(crate) fn load_species(&mut self) {
        self.species.add("species:human", Species::new("human", SpeciesApearance::composite(
            vec!(
                ("base", vec!(
                    ("male_light", "species/human/base_male_light.png"),
                    ("female_light", "species/human/base_female_light.png")
                )),
                ("hair", vec!(
                    ("bun", "species/human/hair_bun.png"),
                    ("short", "species/human/hair_short.png"),
                    ("shaved", "species/human/hair_shaved.png"),
                    ("bald", "system/transparent.png"),
                )),
                ("clothes", vec!(
                    ("peasant", "species/human/clothes_peasant.png"),
                    ("armor", "species/human/armor_placeholder.png")
                )),
            )
        )).innate_actions(vec!(self.actions.id_of("act:punch"))));
        self.species.add("species:leshen", Species::new("leshen", SpeciesApearance::single_sprite("leshen.png"))
            .intelligence(SpeciesIntelligence::Instinctive)
            .attributes(Attributes { strength: 45, agility: 15, constitution: 45, unallocated: 0 })
            .innate_actions(vec!(self.actions.id_of("act:spider_bite")))
            .lifetime(300)
            .fertility(0.)
            .drops(vec!((self.materials.id_of("mat:bone_leshen"), 1)))
        );
        self.species.add("species:fiend", Species::new("fiend", SpeciesApearance::single_sprite("fiend.png"))
            .intelligence(SpeciesIntelligence::Instinctive)
            .attributes(Attributes { strength: 35, agility: 25, constitution: 35, unallocated: 0 })
            .innate_actions(vec!(self.actions.id_of("act:spider_bite")))
            .lifetime(200)
            .fertility(0.)
            .drops(vec!((self.materials.id_of("mat:bone_fiend"), 1)))
        );
        self.species.add("species:spider", Species::new("spider", SpeciesApearance::single_sprite("species/spider.png"))
            .intelligence(SpeciesIntelligence::Instinctive)
            .attributes(Attributes { strength: 5, agility: 12, constitution: 10, unallocated: 0 })
            .innate_actions(vec!(self.actions.id_of("act:spider_bite")))
        );
    }

    pub(crate) fn load_tiles(&mut self) {
        let mut tile = Tile::new(0, "assets/sprites/chunk_tiles/stone.png");
        tile.step_sound_effect = Some(SoundEffect::new(vec!("sfx/step_stone_1.mp3", "sfx/step_stone_2.mp3", "sfx/step_stone_3.mp3")));
        self.tiles.add("tile:stone", tile);
        let mut tile = Tile::new(4, "assets/sprites/chunk_tiles/grass.png");
        tile.step_sound_effect = Some(SoundEffect::new(vec!("sfx/step_grass_1.mp3", "sfx/step_grass_2.mp3", "sfx/step_grass_3.mp3")));
        self.tiles.add("tile:grass", tile);
        let tile = Tile::new(1, "assets/sprites/chunk_tiles/sand.png");
        self.tiles.add("tile:sand", tile);
        let tile = Tile::new(2, "assets/sprites/chunk_tiles/water.png");
        self.tiles.add("tile:water", tile);
        let mut tile = Tile::new(3, "assets/sprites/chunk_tiles/floor.png");
        tile.step_sound_effect = Some(SoundEffect::new(vec!("sfx/step_wood_1.mp3", "sfx/step_wood_2.mp3", "sfx/step_wood_3.mp3")));
        self.tiles.add("tile:floor", tile);
        let mut tile = Tile::new(2, "assets/sprites/chunk_tiles/cobblestone.png");
        tile.step_sound_effect = Some(SoundEffect::new(vec!("sfx/step_stone_1.mp3", "sfx/step_stone_2.mp3", "sfx/step_stone_3.mp3")));
        self.tiles.add("tile:cobblestone", tile);
    }

    pub(crate) fn tile(&self, id: &TileId) -> &Tile {
        return self.tiles.get(id);
    }

    pub(crate) fn find_tile(&self, key: &str) -> &Tile {
        return self.tiles.find(key);
    }

}