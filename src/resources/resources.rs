use crate::{commons::{damage_model::DamageComponent, resource_map::ResourceMap}, engine::{audio::SoundEffect, geometry::Coord2, Color}, game::action::{Action, ActionId, ActionType, DamageType}, world::material::{Material, MaterialId}};

use super::tile::{Tile, TileId};

pub type Actions = ResourceMap<ActionId, Action>;
pub type Materials = ResourceMap<MaterialId, Material>;

#[derive(Clone)]
pub struct Resources {
    pub actions: Actions,
    pub materials: Materials,
    pub tiles: ResourceMap<TileId, Tile>
}

impl Resources {

    pub fn new() -> Resources {
        Resources {
            actions: ResourceMap::new(),
            materials: ResourceMap::new(),
            tiles: ResourceMap::new(),
        }
    }

    pub fn load(&mut self) {
        self.load_actions();
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

    pub fn load_actions(&mut self) {
        self.actions.add("act:sword:attack", Action {
            name: String::from("Attack"),
            icon: String::from("gui/icons/actions/armed_attack.png"),
            sound_effect: Some(SoundEffect::new(vec!("sfx/sword_1.mp3", "sfx/sword_2.mp3", "sfx/sword_3.mp3"))),
            ap_cost: 40,
            action_type: ActionType::Targeted { damage: Some(DamageType::FromWeapon) }
        });
        self.actions.add("act:punch", Action {
            name: String::from("Punch"),
            icon: String::from("gui/icons/actions/unarmed_attack.png"),
            sound_effect: Some(SoundEffect::new(vec!("sfx/punch_1.mp3", "sfx/punch_2.mp3"))),
            ap_cost: 40,
            action_type: ActionType::Targeted { damage: Some(DamageType::Fixed(DamageComponent::new(0., 0., 1.))) }
        });
        self.actions.add("act:talk", Action {
            name: String::from("Talk"),
            icon: String::from("gui/icons/actions/talk.png"),
            sound_effect: None,
            ap_cost: 0,
            action_type: ActionType::Talk
        });
        self.actions.add("act:pickup", Action {
            name: String::from("Pick Up"),
            icon: String::from("gui/icons/actions/pickup.png"),
            sound_effect: None,
            ap_cost: 20,
            action_type: ActionType::PickUp
        });
        self.actions.add("act:sleep", Action {
            name: String::from("Sleep"),
            icon: String::from("gui/icons/actions/sleep.png"),
            sound_effect: None,
            ap_cost: 0,
            action_type: ActionType::Sleep
        });
        self.actions.add("act:move_left", Action {
            name: String::from("Move Left"),
            icon: String::from("gui/icons/actions/sleep.png"),
            sound_effect: None,
            ap_cost: 20,
            action_type: ActionType::Move { offset: Coord2::xy(-1, 0) }
        });
        self.actions.add("act:move_right", Action {
            name: String::from("Move Right"),
            icon: String::from("gui/icons/actions/sleep.png"),
            sound_effect: None,
            ap_cost: 20,
            action_type: ActionType::Move { offset: Coord2::xy(1, 0) }
        });
        self.actions.add("act:move_up", Action {
            name: String::from("Move Up"),
            icon: String::from("gui/icons/actions/sleep.png"),
            sound_effect: None,
            ap_cost: 20,
            action_type: ActionType::Move { offset: Coord2::xy(0, -1) }
        });
        self.actions.add("act:move_down", Action {
            name: String::from("Move Down"),
            icon: String::from("gui/icons/actions/sleep.png"),
            sound_effect: None,
            ap_cost: 20,
            action_type: ActionType::Move { offset: Coord2::xy(0, 1) }
        });
    }

    pub fn tile(&self, id: &TileId) -> &Tile {
        return self.tiles.get(id);
    }

    pub fn find_tile(&self, key: &str) -> &Tile {
        return self.tiles.find(key);
    }

}