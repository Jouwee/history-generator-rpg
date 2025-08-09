use std::{cell::RefCell, time::Instant};

use image::ImageReader;

use crate::{commons::{damage_model::{DamageModel, DamageRoll}, resource_map::ResourceMap}, engine::{assets::ImageSheetAsset, audio::SoundEffect, geometry::Size2D, pallete_sprite::PalleteSprite, tilemap::{Tile16Subset, TileRandom, TileSingle}, Color}, game::{actor::health_component::BodyPart, inventory::inventory::EquipmentType}, resources::{action::{ActionArea, ActionEffect, ActionProjectile, ActionTarget, ImpactPosition, SpellProjectileType, FILTER_CAN_DIG, FILTER_CAN_OCCUPY, FILTER_CAN_SLEEP, FILTER_CAN_VIEW, FILTER_ITEM, FILTER_NOT_HOSTILE}, item_blueprint::ArmorBlueprintComponent, material::{MAT_TAG_BONE, MAT_TAG_METAL, MAT_TAG_WOOD}, species::SpeciesAppearance}, world::{attributes::Attributes, item::{ActionProviderComponent, EquippableComponent}}, MarkovChainSingleWordModel};
use super::{action::{Action, Actions, Affliction}, biome::{Biome, Biomes}, culture::{Culture, Cultures}, item_blueprint::{ArtworkSceneBlueprintComponent, ItemBlueprint, ItemBlueprints, MaterialBlueprintComponent, MelleeDamageBlueprintComponent, NameBlueprintComponent, QualityBlueprintComponent}, material::{Material, Materials}, object_tile::{ObjectTile, ObjectTileId}, species::{Species, SpeciesIntelligence, SpeciesMap}, tile::{Tile, TileId}};

#[derive(Clone)]
pub(crate) struct Resources {
    pub(crate) actions: Actions,
    pub(crate) materials: Materials,
    pub(crate) species: SpeciesMap,
    pub(crate) tiles: ResourceMap<TileId, Tile>,
    pub(crate) object_tiles: ResourceMap<ObjectTileId, ObjectTile>,
    pub(crate) cultures: Cultures,
    pub(crate) biomes: Biomes,
    pub(crate) item_blueprints: ItemBlueprints,
}

impl Resources {

    pub(crate) fn new() -> Resources {
        Resources {
            actions: Actions::new(),
            materials: Materials::new(),
            species: SpeciesMap::new(),
            tiles: ResourceMap::new(),
            object_tiles: ResourceMap::new(),
            cultures: Cultures::new(),
            biomes: Biomes::new(),
            item_blueprints: ItemBlueprints::new(),
        }
    }

    pub(crate) fn load(&mut self) {
        let now = Instant::now();
        self.load_materials();
        self.load_tiles();
        self.load_object_tiles();
        self.load_actions();
        self.load_species();
        self.load_cultures();
        self.load_biomes();
        self.load_item_blueprints();
        println!("Loading resources took {:.2?}", now.elapsed())
    }

    fn load_materials(&mut self) {
        let mut steel = Material::new_metal("steel");
        steel.color_pallete = [Color::from_hex("405273"), Color::from_hex("6c81a1"), Color::from_hex("96a9c1"), Color::from_hex("bbc3d0")];
        steel.sharpness = 1.75;
        self.materials.add("mat:steel", steel);
        
        let mut iron = Material::new_metal("iron");
        iron.color_pallete = [Color::from_hex("4d5666"), Color::from_hex("798494"), Color::from_hex("a1aab6"), Color::from_hex("c0c4cb")];
        iron.sharpness = 1.5;
        self.materials.add("mat:iron", iron);
        
        let mut bronze = Material::new_metal("bronze");
        bronze.color_pallete = [Color::from_hex("a57855"), Color::from_hex("de9f47"), Color::from_hex("fdd179"), Color::from_hex("fee1b8")];
        bronze.sharpness = 1.2;
        self.materials.add("mat:bronze", bronze);

        let mut copper = Material::new_metal("copper");
        copper.color_pallete = [Color::from_hex("593e47"), Color::from_hex("b55945"), Color::from_hex("de9f47"), Color::from_hex("f2b888")];
        copper.sharpness = 1.;
        self.materials.add("mat:copper", copper);

        self.materials.add("mat:birch", Material::new_wood("birch"));

        self.materials.add("mat:oak", Material::new_wood("oak"));

        let mut bone = Material::new_bone("varningr's bone");
        bone.extra_damage = DamageRoll::arcane(10.);
        self.materials.add("mat:varningr_bone", bone);
    }

    fn load_biomes(&mut self) {
        self.biomes.add("biome:ocean", Biome {
            elevation: (-2000, 0),
            temperature: (0, 5),
            vegetation: (0.0, 0.0),
            soil_fertility_range: (0.8, 1.2),
        });
        self.biomes.add("biome:coast", Biome {
            elevation: (0, 16),
            temperature: (0, 5),
            vegetation: (0.0, 0.1),
            soil_fertility_range: (0.8, 1.2),
        });
        self.biomes.add("biome:grasslands", Biome {
            elevation: (16, 255),
            temperature: (0, 2),
            vegetation: (0.5, 1.),
            soil_fertility_range: (1.0, 1.4),
        });
        self.biomes.add("biome:forest", Biome {
            elevation: (16, 255),
            temperature: (0, 2),
            vegetation: (0.5, 1.),
            soil_fertility_range: (1.0, 1.4),
        });
        self.biomes.add("biome:desert", Biome {
            elevation: (16, 255),
            temperature: (3, 6),
            vegetation: (0.0, 0.1),
            soil_fertility_range: (0.5, 0.9),
        });
    }

    fn load_actions(&mut self) {
        self.actions.add("act:strike", Action {
            name: String::from("Strike"),
            description: String::from("Strikes with your weapon"),
            icon: String::from("gui/icons/actions/slashing_cut.png"),
            log_use: true,
            cast_sfx: Some(SoundEffect::new(vec!("sfx/sword_1.mp3", "sfx/sword_2.mp3", "sfx/sword_3.mp3"))),
            ap_cost: 40,
            stamina_cost: 3.,
            cooldown: 0,
            target: ActionTarget::Actor { range: 1.5, filter_mask: 0 },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::Damage { add_weapon: true, damage: DamageRoll::empty() },
            ),
            cast_sprite: None,
            projectile: None,
            impact_sprite: None,
            impact_sfx: None,
            damage_sfx: None
        });

        self.actions.add("act:sword:bleeding_cut", Action {
            name: String::from("Bleeding Cut"),
            description: String::from("A deep cut that causes bleeding"),
            log_use: true,
            icon: String::from("gui/icons/actions/bleeding_cut.png"),
            cast_sfx: Some(SoundEffect::new(vec!("sfx/sword_1.mp3", "sfx/sword_2.mp3", "sfx/sword_3.mp3"))),
            ap_cost: 60,
            stamina_cost: 20.,
            cooldown: 2,
            target: ActionTarget::Actor { range: 1.5, filter_mask: 0 },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::Damage { add_weapon: true, damage: DamageRoll::empty() },
                ActionEffect::Inflicts { affliction: Affliction::Bleeding { duration: 3 } }
            ),
            cast_sprite: None,
            projectile: None,
            impact_sprite: None,
            impact_sfx: None,
            damage_sfx: None
        });

        self.actions.add("act:mace:concussive_strike", Action {
            name: String::from("Concussive Strike"),
            description: String::from("An aimed hit at the head"),
            log_use: true,
            icon: String::from("gui/icons/actions/concussive_strike.png"),
            cast_sfx: Some(SoundEffect::new(vec!("sfx/punch_1.mp3", "sfx/punch_2.mp3"))),
            ap_cost: 60,
            stamina_cost: 20.,
            cooldown: 2,
            target: ActionTarget::Actor { range: 1.5, filter_mask: 0 },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::Damage { add_weapon: true, damage: DamageRoll::empty() },
                ActionEffect::Inflicts { affliction: Affliction::Stunned { duration: 1 } }
            ),
            cast_sprite: None,
            projectile: None,
            impact_sprite: None,
            impact_sfx: None,
            damage_sfx: None
        });
        self.actions.add("act:punch", Action {
            name: String::from("Punch"),
            description: String::from("A good ol' punch"),
            log_use: true,
            icon: String::from("gui/icons/actions/unarmed_attack.png"),
            cast_sfx: Some(SoundEffect::new(vec!("sfx/punch_1.mp3", "sfx/punch_2.mp3"))),
            ap_cost: 40,
            stamina_cost: 5.,
            cooldown: 0,
            target: ActionTarget::Actor { range: 1.5, filter_mask: 0 },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::Damage { add_weapon: false, damage: DamageRoll::bludgeoning(5.) },
            ),
            cast_sprite: None,
            projectile: None,
            impact_sprite: None,
            impact_sfx: None,
            damage_sfx: None
        });
        self.actions.add("act:spider_bite", Action {
            name: String::from("Bite"),
            description: String::from("A spider bite"),
            log_use: true,
            icon: String::from("missing.png"),
            cast_sfx: Some(SoundEffect::new(vec!("sfx/generic_swoosh_1.mp3", "sfx/generic_swoosh_2.mp3", "sfx/generic_swoosh_3.mp3", "sfx/generic_swoosh_4.mp3"))),
            ap_cost: 40,
            stamina_cost: 3.,
            cooldown: 0,
            target: ActionTarget::Actor { range: 1.5, filter_mask: 0 },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::Damage { add_weapon: false, damage: DamageRoll::piercing(10.) },
                ActionEffect::Inflicts { affliction: Affliction::Poisoned { duration: 10 } }
            ),
            cast_sprite: None,
            projectile: None,
            impact_sprite: None,
            impact_sfx: None,
            damage_sfx: None
        });
        self.actions.add("act:bite", Action {
            name: String::from("Bite"),
            description: String::from("A bite"),
            icon: String::from("missing.png"),
            log_use: true,
            cast_sfx: Some(SoundEffect::new(vec!("sfx/generic_swoosh_1.mp3", "sfx/generic_swoosh_2.mp3", "sfx/generic_swoosh_3.mp3", "sfx/generic_swoosh_4.mp3"))),
            ap_cost: 40,
            stamina_cost: 3.,
            cooldown: 0,
            target: ActionTarget::Actor { range: 1.5, filter_mask: 0 },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::Damage { add_weapon: false, damage: DamageRoll::piercing(10.) },
            ),
            cast_sprite: None,
            projectile: None,
            impact_sprite: Some((ImageSheetAsset::new("visual_effects/bite.png", Size2D(24, 24)), 0.5, ImpactPosition::EachTarget, false)),
            impact_sfx: None,
            damage_sfx: Some(SoundEffect::new(vec!("sfx/damage_flesh_1.mp3", "sfx/damage_flesh_2.mp3", "sfx/damage_flesh_3.mp3"))),
        });
        self.actions.add("act:bite_varningr", Action {
            name: String::from("Bite"),
            description: String::from("A bite"),
            icon: String::from("missing.png"),
            log_use: true,
            cast_sfx: Some(SoundEffect::new(vec!("sfx/generic_swoosh_1.mp3", "sfx/generic_swoosh_2.mp3", "sfx/generic_swoosh_3.mp3", "sfx/generic_swoosh_4.mp3"))),
            ap_cost: 40,
            stamina_cost: 3.,
            cooldown: 0,
            target: ActionTarget::Actor { range: 1.5, filter_mask: 0 },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::Damage { add_weapon: false, damage: DamageRoll::piercing(20.) },
            ),
            cast_sprite: None,
            projectile: None,
            impact_sprite: Some((ImageSheetAsset::new("visual_effects/bite.png", Size2D(24, 24)), 0.5, ImpactPosition::EachTarget, false)),
            impact_sfx: None,
            damage_sfx: Some(SoundEffect::new(vec!("sfx/damage_flesh_1.mp3", "sfx/damage_flesh_2.mp3", "sfx/damage_flesh_3.mp3"))),
        });

        self.actions.add("act:deafening_howl", Action {
            name: String::from("Deafening howl"),
            description: String::from("A deafening howl"),
            icon: String::from("missing.png"),
            log_use: true,
            cast_sfx: Some(SoundEffect::new(vec!("sfx/varningr_screech.mp3"))),
            ap_cost: 40,
            stamina_cost: 5.,
            cooldown: 30,
            target: ActionTarget::Caster,
            area: ActionArea::Circle { radius: 8.2 },
            effects: vec!(
                ActionEffect::Inflicts { affliction: Affliction::Stunned { duration: 2 } }
            ),
            cast_sprite: None,
            projectile: None,
            impact_sprite: Some((ImageSheetAsset::new("visual_effects/shockwave.png", Size2D(72, 72)), 0.5, ImpactPosition::Cursor, false)),
            impact_sfx: None,
            damage_sfx: None
        });

        self.actions.add("act:firebolt", Action {
            name: String::from("Firebolt"),
            description: String::from("Throws a fiery bolt"),
            icon: String::from("gui/icons/actions/firebolt.png"),
            log_use: true,
            cast_sfx: Some(SoundEffect::new(vec!("sfx/firebolt_cast.wav"))),
            ap_cost: 50,
            stamina_cost: 5.,
            cooldown: 2,
            target: ActionTarget::Actor { range: 10., filter_mask: FILTER_CAN_VIEW },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::Damage { add_weapon: false, damage: DamageRoll::fire(20.) },
                ActionEffect::Inflicts { affliction: Affliction::OnFire { duration: 6 } }
            ),
            cast_sprite: Some((ImageSheetAsset::new("projectiles/cast_fire.png", Size2D(16, 16)), 0.1)),
            projectile: Some(ActionProjectile { wait: true, position: ImpactPosition::EachTarget, projectile_type: SpellProjectileType::Projectile { sprite: ImageSheetAsset::new("projectiles/firebolt.png", Size2D(16, 8)), speed: 20. } }),
            impact_sprite: Some((ImageSheetAsset::new("projectiles/explosion.png", Size2D(64, 64)), 0.5, ImpactPosition::EachTarget, false)),
            impact_sfx: Some(SoundEffect::new(vec!("sfx/fire_explosion.wav"))),
            damage_sfx: None
        });

        self.actions.add("act:fireball", Action {
            name: String::from("Fireball"),
            description: String::from("Casts an explosive ball of fire"),
            log_use: true,
            icon: String::from("gui/icons/actions/fireball.png"),
            cast_sfx: Some(SoundEffect::new(vec!("sfx/firebolt_cast.wav"))),
            ap_cost: 60,
            stamina_cost: 5.,
            cooldown: 5,
            target: ActionTarget::Tile { range: 10., filter_mask: FILTER_CAN_OCCUPY },
            area: ActionArea::Circle { radius: 2.5 },
            effects: vec!(
                ActionEffect::Damage { add_weapon: false, damage: DamageRoll::fire(20.) },
            ),
            cast_sprite: Some((ImageSheetAsset::new("projectiles/cast_fire.png", Size2D(16, 16)), 0.1)),
            projectile: Some(ActionProjectile { wait: true, position: ImpactPosition::Cursor, projectile_type: SpellProjectileType::Projectile { sprite: ImageSheetAsset::new("projectiles/firebolt.png", Size2D(16, 8)), speed: 20. } }),
            impact_sprite: Some((ImageSheetAsset::new("visual_effects/explosion_big.png", Size2D(128, 128)), 0.5, ImpactPosition::Cursor, false)),
            impact_sfx: Some(SoundEffect::new(vec!("sfx/fire_explosion.wav"))),
            damage_sfx: None
        });

        self.actions.add("act:rockpillar", Action {
            name: String::from("Rock Pillar"),
            description: String::from("Summons a pillar of rock"),
            icon: String::from("gui/icons/actions/rock_pillar.png"),
            log_use: true,
            cast_sfx: Some(SoundEffect::new(vec!("sfx/rockwall.wav"))),
            ap_cost: 20,
            stamina_cost: 5.,
            cooldown: 5,
            target: ActionTarget::Tile { range: 10., filter_mask: FILTER_CAN_OCCUPY },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::ReplaceObject { tile: self.object_tiles.id_of("obj:rock_pillar") }
            ),
            cast_sprite: None,
            projectile: None,
            impact_sprite: Some((ImageSheetAsset::new("visual_effects/rock_pillar_spawn.png", Size2D(24, 32)), 0.5, ImpactPosition::EachTile, true)),
            impact_sfx: None,
            damage_sfx: None
        });

        self.actions.add("act:teleport", Action {
            name: String::from("Teleport"),
            description: String::from("Instantly teleports away"),
            icon: String::from("gui/icons/actions/teleport.png"),
            log_use: true,
            cast_sfx: Some(SoundEffect::new(vec!("sfx/teleport_cast.wav"))),
            ap_cost: 20,
            stamina_cost: 5.,
            cooldown: 10,
            target: ActionTarget::Tile { range: 13., filter_mask: FILTER_CAN_OCCUPY },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::TeleportActor
            ),
            cast_sprite: Some((ImageSheetAsset::new("visual_effects/teleport_source.png", Size2D(24, 48)), 0.1)),
            projectile: None,
            impact_sprite: Some((ImageSheetAsset::new("visual_effects/teleport_dest.png", Size2D(24, 48)), 0.5, ImpactPosition::Cursor, false)),
            impact_sfx: None,
            damage_sfx: None
        });

        self.actions.add("act:talk", Action {
            name: String::from("Talk"),
            description: String::from("Talk with"),
            icon: String::from("gui/icons/actions/talk.png"),
            log_use: false,
            cast_sfx: None,
            ap_cost: 0,
            stamina_cost: 0.,
            cooldown: 0,
            target: ActionTarget::Actor { range: 3., filter_mask: FILTER_NOT_HOSTILE },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::Talk
            ),
            cast_sprite: None,
            projectile: None,
            impact_sprite: None,
            impact_sfx: None,
            damage_sfx: None
        });

        self.actions.add("act:inspect", Action {
            name: String::from("Inspect"),
            description: String::from("Inspect something"),
            icon: String::from("gui/icons/actions/inspect.png"),
            log_use: false,
            cast_sfx: None,
            ap_cost: 0,
            stamina_cost: 0.,
            cooldown: 0,
            target: ActionTarget::Tile { range: 5., filter_mask: 0 },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::Inspect
            ),
            cast_sprite: None,
            projectile: None,
            impact_sprite: None,
            impact_sfx: None,
            damage_sfx: None
        });
        self.actions.add("act:dig", Action {
            name: String::from("Dig"),
            description: String::from("Dig the ground"),
            icon: String::from("gui/icons/actions/dig.png"),
            log_use: false,
            cast_sfx: None,
            ap_cost: 0,
            stamina_cost: 0.,
            cooldown: 0,
            target: ActionTarget::Tile { range: 1.5, filter_mask: FILTER_CAN_DIG },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::Dig
            ),
            cast_sprite: None,
            projectile: None,
            impact_sprite: None,
            impact_sfx: None,
            damage_sfx: None
        });
        self.actions.add("act:pickup", Action {
            name: String::from("Pick Up"),
            description: String::from("Pick up something from the ground"),
            icon: String::from("gui/icons/actions/pickup.png"),
            log_use: false,
            cast_sfx: None,
            ap_cost: 20,
            stamina_cost: 1.,
            cooldown: 0,
            target: ActionTarget::Tile { range: 1.5, filter_mask: FILTER_ITEM },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::PickUp
            ),
            cast_sprite: None,
            projectile: None,
            impact_sprite: None,
            impact_sfx: None,
            damage_sfx: None
        });
        self.actions.add("act:sleep", Action {
            name: String::from("Sleep"),
            description: String::from("Sleep in a bed"),
            icon: String::from("gui/icons/actions/sleep.png"),
            log_use: false,
            cast_sfx: None,
            ap_cost: 0,
            stamina_cost: 0.,
            cooldown: 0,
            target: ActionTarget::Tile { range: 1.5, filter_mask: FILTER_CAN_SLEEP },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::Sleep
            ),
            cast_sprite: None,
            projectile: None,
            impact_sprite: None,
            impact_sfx: None,
            damage_sfx: None
        });

        self.actions.add("act:move", Action {
            name: String::from("Move"),
            description: String::from("Move"),
            icon: String::from("gui/icons/actions/sleep.png"),
            log_use: false,
            cast_sfx: None,
            ap_cost: 20,
            stamina_cost: 0.2,
            cooldown: 0,
            target: ActionTarget::Tile { range: 1., filter_mask: FILTER_CAN_OCCUPY },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::Walk
            ),
            cast_sprite: None,
            projectile: None,
            impact_sprite: None,
            impact_sfx: None,
            damage_sfx: None

        });
    }

    fn load_species(&mut self) {
        self.species.add("species:human", Species::new("human", SpeciesAppearance::Composite {
            base: vec!("species/human/base.png".to_string()),
            top: vec!(
                "species/human/hair_a.png".to_string(),
                "species/human/hair_b.png".to_string(),
                "species/human/hair_c.png".to_string(),
            )
        })
        .innate_actions(vec!(self.actions.id_of("act:punch"))));

        self.species.add("species:spider", Species::new("spider", SpeciesAppearance::Single("species/spider.png".to_string()))
            .intelligence(SpeciesIntelligence::Instinctive)
            .attributes(Attributes { strength: 5, agility: 12, constitution: 10, unallocated: 0 })
            .innate_actions(vec!(self.actions.id_of("act:spider_bite")))
        );

        self.species.add("species:wolf", Species::new("wolf", SpeciesAppearance::Single("species/wolf/wolf.png".to_string()))
            .intelligence(SpeciesIntelligence::Instinctive)
            .attributes(Attributes { strength: 5, agility: 12, constitution: 10, unallocated: 0 })
            .drops(vec!())
            .innate_actions(vec!(self.actions.id_of("act:bite")))
            .hurt_sound(SoundEffect::new(vec!("sfx/wolf_hurt-01.mp3", "sfx/wolf_hurt-02.mp3")))
        );

        self.species.add("species:varningr", Species::new("varningr", SpeciesAppearance::Single("species/varningr/varningr.png".to_string()))
            .intelligence(SpeciesIntelligence::Instinctive)
            .attributes(Attributes { strength: 5, agility: 12, constitution: 10, unallocated: 0 })
            .drops(vec!(self.materials.id_of("mat:varningr_bone")))
            .max_hp(200.)
            .innate_actions(vec!(self.actions.id_of("act:bite_varningr"), self.actions.id_of("act:deafening_howl")))
        );
    }

    fn load_cultures(&mut self) {

        self.cultures.add("culture:default", Culture {
            first_name_male_model: MarkovChainSingleWordModel::train(vec!(
                "Aawlynson", "Abbathor", "Marcus", "Acuakvacaesin", "Bert", "Adarbron", "Adarl", "Adon", "Adreim", "Adrian", "Adrik", "Ganrahast", "Aelioth", "Aereld", "Aesgareth", "Agnar", "Agni", "Savyels", "Akhlaur", "Akila", "Akimatsu", "Rusty", "Alash", "Alavaernith", "Ali", "Emri", "Saerghon", "Alledec", "Corkitron", "Alnyskawer", "Alobal", "Alsevir", "Dauphran", "Alvarro", "Erilon", "Amalzen", "Amandarn", "Amanthan", "Amaunator", "Petric", "Ambrival", "Ambrose", "Teclel", "Amhar", "Noro", "Naralis", "Anauviir", "Astarion", "Naeryk", "Andras", "Andromalius", "Orluth", "Gyldro", "Anndoquat", "Ansel", "Ansur", "Antaerl", "Antarn", "Anthraxus", "Anubis", "Apep", "Reldyk", "Aptoryx", "Aquilan", "Araithe", "Araleth", "Arash", "Architrave", "Ardalis", "Argathakul", "Arghel", "Argon", "Arkwright", "Danzo", "Ayo", "Tym", "Ildur", "Vicben", "Arthius", "Arthlach", "Arvas", "Arvoreen", "Asglyn", "Ashardalon", "Ashen", "Ashni", "Askell", "Askepel", "Aszhgruzz", "Athlar", "Atson", "Aumvor", "Aunsiber", "Irbryth", "Avarilous", "Banister", "Odezzt", "Orgallryd", "Phargred", "Guldryx", "Axerock", "Azazabus", "Azena'ar", "Azmyrandyr", "Azuth", "Babadul", "Hugo", "Bad", "Badmuddin", "Julani", "Baeraden", "Baerdagh", "Baerengard", "Orbrin", "Baerimgrim", "Rungo", "Baerold", "Bagdog", "Bahamut", "Bahgtru", "Bailey", "Balagos", "Hasheth", "Augustin", "Grykk", "Bantu", "Baphomet", "Barachiel", "Nildon", "Barcus", "Montror", "Barent", "Aldon", "Feston", "Barnabas", "Baron", "Xorthaul", "Bartley", "Barundryn", "Barze", "Bruenor", "Gandalug", "Batu", "Phaernos", "Arls", "Yintras", "Bedi", "Begoas", "Beherit", "Zhuang", "Belarian", "Belathin", "Beldar", "Beldrune", "Kregos", "Bellard", "Anwhar", "Belub-Zi", "Belundrar", "Belvyn", "Belym", "Ben-hadar", "Benevolent", "Beornegar", "Tim", "Berkthgar", "Bertio", "Torin", "Bexendral", "Bezantur", "Bheloris", "Hastar", "Bile-Tooth", "Silas", "Bim", "Birtron", "Drongo", "Florean", "Black", "Dedryk", "Branson", "Blackclaw", "Blackcut", "Klaern", "Korlar", "Othkyn", "Yarjack", "Nets", "Blau", "Bliggerillo", "Sivinil", "Tharl", "Parespur", "Skyrim", "Bug"
            ), 3),
            first_name_female_model: MarkovChainSingleWordModel::train(vec!(
                "Faline", "Abigail", "Aconflagblazen", "Ada", "Addee", "Adrianna", "Adrielle", "Aeark", "Kirina", "Soraevora", "Aeriell", "Agarta", "Agaza", "Aghilde", "Ai", "Aizagora", "Akadi", "Jaluth", "Alaithe", "Alathrien", "Haramara", "Shanyrria", "Endrenn", "Alodia", "Alshinree", "Altaira", "Alustriel", "Alyth", "Clarorna", "Amaka", "Amalrae", "Amaranth", "Amaterasu", "Tresse", "Amelyssan", "Ammuthe", "Andalara", "Angharradh", "Annya", "Antinia", "Aquesita", "Arabella", "Arariel", "Archmeagan", "Arden", "Arlavaunta", "Talli", "Arrinye", "Arsekaslyx", "Eolynn", "Arveiaturace", "Ashanda", "Asmartha", "Asta", "Lyrna", "Astrala", "Ate'Niah", "Meralda", "Aurelia", "Aurora", "Greyarra", "Seyll", "Balbaste", "Avroana", "Ayana", "Aylin", "Baalfe", "Chessae", "Sos'Umptu", "Betilda", "Ilmadia", "Batu", "Bay", "Bearclaw", "Rosie", "Hannaah", "Oryne", "Marrauda", "Bella", "Belmora", "Rina", "Ailis", "Benta", "Lora", "Vanra", "Berna", "Bethany", "Jula", "Birdsong", "Birgit", "Ciara", "Blibdoolpoolp", "Blizzard", "Iza", "Allice", "Ilza", "Isabella", "Voebe", "Melinda", "Ana", "Branwyn", "Randalla", "Breatis", "Brelma", "Tasha", "Emalline", "Lokelis", "Brigid", "Blossom", "Bunny", "Leepak", "Tanya", "Brunwell", "Mama", "Budaera", "Bwimb", "Caerdwyn", "Calantha", "Calathlarra", "Callia", "Calliope", "Aida", "Candril", "Carmen", "Eleanor", "Yolanda", "Sylull", "Cathla", "Cecie", "Hanali", "Ch'kk'ch", "Chan", "Caryn", "Chandra", "Chaotic", "Chaslarla", "Chee'ah", "Chessa", "Sofia", "Chinedu", "Chioptl", "Jemima", "Chlasa", "Harissa", "Cirian", "Clara", "Colson", "Connomae", "Prophia", "Sallyanne", "Corlos", "Sharanralee", "Delly", "Miranda", "Tamara", "Cyriana", "Cyrilla", "Cyrrollalee", "Daera", "Daerdatha", "Shiv", "Daija", "Dalyria", "Danfora", "Mantorra", "Darantha", "Darbonna", "Erliza", "Darien", "Helacorte", "Dark", "Scyllua", "Darlethra", "Neetha", "Darthleene", "Dasumia", "Rosella", "Valanice", "Eleuthra", "Nerys", "Darra", "Debac", "Rags", "Deelin", "Tuala", "Deidre", "Marantine", "Delores", "Delyarna", "Denderida", "Lobo", "Dia", "Kree", "Deucala", "Kimmy", "Monifa", "Diancastra"
            ), 3),
            last_name_model: MarkovChainSingleWordModel::train(vec!(
                "Abbot", "Ulphor", "Aelorothi", "Aeravand", "Ch'ing", "Alaerth", "Druanna", "Alenuath", "Alenuath", "Allerendris", "Drodeen", "Amaerityl", "Ambryn", "Solar'el", "Armbrust", "Arva", "Aster", "Ganderlay", "Auvryath", "Auzkovyn", "Avithoul", "Baenrahel", "Baenre", "Bankstone", "Bariel", "Hsuang", "Beestinger", "Befling", "Belamadin", "Belaskurth", "Belostos", "Belt", "Bergauz", "Bergauz", "Bingle", "Blank", "Blouin", "Bormul", "Bormul", "Bormul", "Bormul", "Bothgan", "Brabener", "Moonsinger", "Brasshorn", "Brightbottle", "Brightbough", "Brightburn", "Brislen", "Brislen", "Brislen", "Brock", "Godber", "Bubbins", "Camber", "Carrathal", "Carrathal", "Carrathal", "Cassalanter", "Celanil", "Chandler", "Stol", "Eva", "Chiaroscuro", "Chisolm", "Chopeta", "Copperfire", "Corks", "Crownstar", "Curtie", "Cymrych", "Cymrych", "Dahlia", "Daramos", "Daressin", "Darimmon", "Stag", "Darkhope", "Greatgaunt", "Darra", "Daventhorn", "Daventhorn", "Davos", "Dawnhelm", "Aerindale", "Deelarma", "Uwoke", "Delcastle", "Denysse", "Depthcaria", "Derryck", "Diallo", "Aboy", "Adams", "Summerstar", "Freeman", "Ironcross", "Aeiulvana", "Aka'Pillihp", "Mori", "Al", "ibn-Dakimh", "Aliniki", "Alir", "Allinamuck", "Lorfiril", "Alskyte", "Amadar", "Amber", "Jondrathal", "Amoto", "Analor", "Ancunín", "Andolphyn", "Angalaer", "Angleiron", "Applecrown", "Greatspan", "Letheranil", "Sanjar", "Bael", "Arkwright", "Armbrust", "Armbrust", "Arntar", "Arroway", "Rock", "Stockhold", "Authamaun", "Avithoul", "Avithoul", "Avithoul", "Avithoul", "Avrenyl", "Babris", "Fruul", "Baenre", "Baerent", "Baerlan", "Balik", "Bambulozzi", "Bannersworn", "Baraejhe", "Wroot", "Barelder", "Bargewright", "Bargewright", "of", "Barriath", "Harbright", "Battlehammer", "Battlehammer", "Battlehammer", "Min", "Bauldyn", "Beatorh", "Bedelmrin", "Bei", "Belizzian", "Bellgate", "Twofingers", "Betrich", "Biggs", "Billmore", "Bixworth", "Bixworth", "Dan", "Jack", "Leopard", "Black", "Blackboot", "Blaenbar", "Blaenbar", "Blaenbar", "Blamreld", "Blandorf", "Blomyr", "Bloodbar", "Bloodbright", "Bloodfang", "Bludgeon"
            ), 3),
            city_name_model: MarkovChainSingleWordModel::train(vec!(
                "Luskan", "Mirabar", "Neverwinter", "Waterdeep", "Harrowdale", "Mythdrannor", "Ordulin", "Scardale", "Procampur", "Thultanthar", "Tilverton", "Zhentil", "Bildoobaris", "Immilmar", "Lyrabar", "Telflamm", "Athkatla", "Beregost", "Calimport", "Darromar", "Iriaebor", "Alaghon", "Arrabar", "Ordulin", "Ormath", "Suzail", "Westgate", "Wheloon", "Airspur", "Cimbar", "Djerad", "Thymar", "Eltabbar", "Gheldaneth", "Messemprar", "Ghaast", "Skyclave", "Skuld", "Veltalar", "Mezro", "Narubel", "Tashluta", "Alamontyr", "Cathyr", "Derlusk", "Halarahh", "Rethmar", "Beluir", "Chavyondat", "Vaelan", "Palevash", "Tomyris", "Ausa", "Banang", "Durkon", "Hachoni", "Karatin", "Kirin", "Linshung", "Shangtou", "To'ming", "Tsingtao", "Wai", "Yenching", "Aru", "Chozawa", "Dojyu", "Fochu", "Jasuga", "Masakado", "Nakamaru", "Tupe", "Uwaji", "Hafayah", "Hawa", "Liham", "Muluk", "Qadib", "Qudra", "Umara", "Utaqa", "Halwa", "Hiyal", "Huzuz", "Wasat", "Dihliz", "Kadarasto", "Rog'osto", "Ajayib", "Gana", "Jumlat", "Sikak", "Tajar", "Fahhas", "Hilm", "Hudid", "I'tiraf", "Mahabba", "Talab"
            ), 3)
        });
    }

    fn load_tiles(&mut self) {
        let mut tile = Tile::new(0, "chunk_tiles/stone.png");
        tile.step_sound_effect = Some(SoundEffect::new(vec!("sfx/step_stone_1.mp3", "sfx/step_stone_2.mp3", "sfx/step_stone_3.mp3")));
        self.tiles.add("tile:stone", tile);
        let mut tile = Tile::new(4, "chunk_tiles/grass.png");
        tile.step_sound_effect = Some(SoundEffect::new(vec!("sfx/step_grass_1.mp3", "sfx/step_grass_2.mp3", "sfx/step_grass_3.mp3")));
        self.tiles.add("tile:grass", tile);
        let tile = Tile::new(1, "chunk_tiles/sand.png");
        self.tiles.add("tile:sand", tile);
        let tile = Tile::new(2, "chunk_tiles/water.png");
        self.tiles.add("tile:water", tile);
        let mut tile = Tile::new(3, "chunk_tiles/floor.png");
        tile.step_sound_effect = Some(SoundEffect::new(vec!("sfx/step_wood_1.mp3", "sfx/step_wood_2.mp3", "sfx/step_wood_3.mp3")));
        self.tiles.add("tile:floor", tile);
        let mut tile = Tile::new(2, "chunk_tiles/cobblestone.png");
        tile.step_sound_effect = Some(SoundEffect::new(vec!("sfx/step_stone_1.mp3", "sfx/step_stone_2.mp3", "sfx/step_stone_3.mp3")));
        self.tiles.add("tile:cobblestone", tile);

        let mut tile = Tile::new(4, "chunk_tiles/grass_dark.png");
        tile.step_sound_effect = Some(SoundEffect::new(vec!("sfx/step_grass_1.mp3", "sfx/step_grass_2.mp3", "sfx/step_grass_3.mp3")));
        self.tiles.add("tile:grass_dark", tile);

        let mut tile = Tile::new(4, "chunk_tiles/grass_patchy.png");
        tile.step_sound_effect = Some(SoundEffect::new(vec!("sfx/step_grass_1.mp3", "sfx/step_grass_2.mp3", "sfx/step_grass_3.mp3")));
        self.tiles.add("tile:grass_patchy", tile);

        let mut tile = Tile::new(4, "chunk_tiles/cave_floor.png");
        tile.step_sound_effect = Some(SoundEffect::new(vec!("sfx/step_stone_1.mp3", "sfx/step_stone_2.mp3", "sfx/step_stone_3.mp3")));
        self.tiles.add("tile:cave_floor", tile);
    }

    pub(crate) fn load_object_tiles(&mut self) {
        
        let image = ImageSheetAsset::new("chunk_tiles/stone_walls.png", Size2D(24, 48));
        self.object_tiles.add("obj:wall", ObjectTile::new(crate::engine::tilemap::Tile::T16Subset(Tile16Subset::new(image)), true).with_shadow());

        let image = ImageSheetAsset::new("chunk_tiles/tree.png", Size2D(64, 64));
        self.object_tiles.add("obj:tree", ObjectTile::new(crate::engine::tilemap::Tile::TileRandom(TileRandom::new(image)), true).with_shadow());

        let image = String::from("bed.png");
        self.object_tiles.add("obj:bed", ObjectTile::new(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)), true));

        let image = ImageSheetAsset::new("chunk_tiles/wood_small_table.png", Size2D(24, 24));
        self.object_tiles.add("obj:table", ObjectTile::new(crate::engine::tilemap::Tile::TileRandom(TileRandom::new(image)), true));

        let image = ImageSheetAsset::new("chunk_tiles/wood_stool.png", Size2D(24, 24));
        self.object_tiles.add("obj:stool", ObjectTile::new(crate::engine::tilemap::Tile::TileRandom(TileRandom::new(image)), true));
        
        let image = ImageSheetAsset::new("chunk_tiles/tombstone.png", Size2D(24, 24));
        self.object_tiles.add("obj:tombstone", ObjectTile::new(crate::engine::tilemap::Tile::TileRandom(TileRandom::new(image)), true).with_shadow());
        
        let image = String::from("chunk_tiles/anvil.png");
        self.object_tiles.add("obj:anvil", ObjectTile::new(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)), true));
        
        let image = String::from("chunk_tiles/barrel.png");
        self.object_tiles.add("obj:barrel", ObjectTile::new(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)), true).with_shadow());
        
        let image = ImageSheetAsset::new("chunk_tiles/grass_decal.png", Size2D(24, 24));
        self.object_tiles.add("obj:grass_decal", ObjectTile::new(crate::engine::tilemap::Tile::TileRandom(TileRandom::new(image)), false));

        let image = String::from("chunk_tiles/tent.png");
        self.object_tiles.add("obj:tent", ObjectTile::new(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)), true).with_shadow());

        let image = ImageSheetAsset::new("chunk_tiles/pebbles.png", Size2D(24, 24));
        self.object_tiles.add("obj:pebbles", ObjectTile::new(crate::engine::tilemap::Tile::TileRandom(TileRandom::new(image)), false));

        let image = ImageSheetAsset::new("chunk_tiles/flowers.png", Size2D(24, 24));
        self.object_tiles.add("obj:flowers", ObjectTile::new(crate::engine::tilemap::Tile::TileRandom(TileRandom::new(image)), false));

        let image = ImageSheetAsset::new("chunk_tiles/small_game_carcass.png", Size2D(24, 24));
        self.object_tiles.add("obj:small_game_carcass", ObjectTile::new(crate::engine::tilemap::Tile::TileRandom(TileRandom::new(image)), false));

        let image = String::from("chunk_tiles/rock_pillar.png");
        self.object_tiles.add("obj:rock_pillar", ObjectTile::new(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)), true).with_shadow());

        let image = ImageSheetAsset::new("chunk_tiles/cave_walls.png", Size2D(24, 48));
        self.object_tiles.add("obj:cave_wall", ObjectTile::new(crate::engine::tilemap::Tile::T16Subset(Tile16Subset::new(image)), true).with_shadow());

        let image = String::from("chunk_tiles/ladder_down.png");
        self.object_tiles.add("obj:ladder_down", ObjectTile::new(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)), true));

        let image = String::from("chunk_tiles/ladder_up.png");
        self.object_tiles.add("obj:ladder_up", ObjectTile::new(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)), true));
    }

    fn load_item_blueprints(&mut self) {
        let actions = &self.actions;

        let image = ImageReader::open("./assets/sprites/chunk_tiles/stone_statue.png").unwrap().decode().unwrap();
        let placed_sprite = PalleteSprite::new(image);
        let statue = ItemBlueprint {
            name: String::from("statue"),
            placed_sprite, 
            action_provider: None,
            equippable: None,
            material: Some(MaterialBlueprintComponent {
                primary_tag_bitmask: MAT_TAG_WOOD | MAT_TAG_METAL,
                secondary_tag_bitmask: None,
                details_tag_bitmask: None,
            }),
            quality: None,
            mellee_damage: None,
            armor: None,
            artwork_scene: Some(ArtworkSceneBlueprintComponent { }),
            name_blueprint: None,
        };
        self.item_blueprints.add("itb:statue", statue);

        let image = ImageReader::open("./assets/sprites/species/human/sword_equipped.png").unwrap().decode().unwrap();
        let pallete_sprite = PalleteSprite::new(image);
        let image = ImageReader::open("./assets/sprites/sword.png").unwrap().decode().unwrap();
        let placed_sprite = PalleteSprite::new(image);
        let sword_blueprint = ItemBlueprint {
            name: String::from("sword"),
            placed_sprite, 
            action_provider: Some(ActionProviderComponent { actions: vec!(actions.id_of("act:strike"), actions.id_of("act:sword:bleeding_cut")) }),
            equippable: Some(EquippableComponent { sprite: pallete_sprite, slot: EquipmentType::Hand, cached_texture: RefCell::new(None) }),
            material: Some(MaterialBlueprintComponent {
                primary_tag_bitmask: MAT_TAG_METAL,
                secondary_tag_bitmask: Some(MAT_TAG_WOOD | MAT_TAG_BONE),
                details_tag_bitmask: Some(MAT_TAG_WOOD | MAT_TAG_BONE | MAT_TAG_METAL),
            }),
            quality: Some(QualityBlueprintComponent { }),
            mellee_damage: Some(MelleeDamageBlueprintComponent { base_damage: DamageRoll::slashing(20.) }),
            armor: None,
            artwork_scene: None,
            name_blueprint: Some(NameBlueprintComponent { suffixes: vec!(
                String::from("sword"),
                String::from("blade"),
                String::from("slash"),
                String::from("fang"),
                String::from("tongue"),
                String::from("kiss"),
                String::from("wing"),
                String::from("edge"),
                String::from("talon")
            ) })
        };
        self.item_blueprints.add("itb:sword", sword_blueprint);

        let image = ImageReader::open("./assets/sprites/species/human/axe_equipped.png").unwrap().decode().unwrap();
        let pallete_sprite = PalleteSprite::new(image);
        let image = ImageReader::open("./assets/sprites/species/human/axe.png").unwrap().decode().unwrap();
        let placed_sprite = PalleteSprite::new(image);
        let sword_blueprint = ItemBlueprint {
            name: String::from("axe"),
            placed_sprite, 
            action_provider: Some(ActionProviderComponent { actions: vec!(actions.id_of("act:strike")) }),
            equippable: Some(EquippableComponent { sprite: pallete_sprite, slot: EquipmentType::Hand, cached_texture: RefCell::new(None) }),
            material: Some(MaterialBlueprintComponent {
                primary_tag_bitmask: MAT_TAG_METAL,
                secondary_tag_bitmask: Some(MAT_TAG_WOOD | MAT_TAG_BONE),
                details_tag_bitmask: Some(MAT_TAG_WOOD | MAT_TAG_BONE | MAT_TAG_METAL),
            }),
            quality: Some(QualityBlueprintComponent { }),
            mellee_damage: Some(MelleeDamageBlueprintComponent { base_damage: DamageRoll::slashing(30.) }),
            armor: None,
            artwork_scene: None,
            name_blueprint: Some(NameBlueprintComponent { suffixes: vec!(
                String::from("faller"),
                String::from("blade"),
                String::from("slash"),
                String::from("fang"),
                String::from("tongue"),
                String::from("kiss"),
                String::from("wing"),
                String::from("edge"),
                String::from("talon")
            ) })
        };
        self.item_blueprints.add("itb:axe", sword_blueprint);

        let image = ImageReader::open("./assets/sprites/species/human/mace_equipped.png").unwrap().decode().unwrap();
        let pallete_sprite = PalleteSprite::new(image);
        let image = ImageReader::open("./assets/sprites/mace.png").unwrap().decode().unwrap();
        let placed_sprite = PalleteSprite::new(image);
        let mace_blueprint = ItemBlueprint {
            name: String::from("mace"),
            placed_sprite, 
            action_provider: Some(ActionProviderComponent { actions: vec!(actions.id_of("act:strike"), actions.id_of("act:mace:concussive_strike")) }),
            equippable: Some(EquippableComponent { sprite: pallete_sprite, slot: EquipmentType::Hand, cached_texture: RefCell::new(None) }),
            material: Some(MaterialBlueprintComponent {
                primary_tag_bitmask: MAT_TAG_METAL,
                secondary_tag_bitmask: Some(MAT_TAG_WOOD | MAT_TAG_BONE),
                details_tag_bitmask: Some(MAT_TAG_WOOD | MAT_TAG_BONE | MAT_TAG_METAL),
            }),
            quality: Some(QualityBlueprintComponent { }),
            mellee_damage: Some(MelleeDamageBlueprintComponent { base_damage: DamageRoll::bludgeoning(20.) }),
            armor: None,
            artwork_scene: None,
            name_blueprint: Some(NameBlueprintComponent { suffixes: vec!(String::from("breaker"), String::from("kiss"), String::from("fist"), String::from("touch")) })
        };
        self.item_blueprints.add("itb:mace", mace_blueprint);


        let image = ImageReader::open("./assets/sprites/species/human/shirt_equipped.png").unwrap().decode().unwrap();
        let pallete_sprite = PalleteSprite::new(image);
        let image = ImageReader::open("./assets/sprites/species/human/peasant_shirt.png").unwrap().decode().unwrap();
        let placed_sprite = PalleteSprite::new(image);
        let shirt_blueprint = ItemBlueprint {
            name: String::from("peasant shirt"),
            placed_sprite, 
            action_provider: None,
            equippable: Some(EquippableComponent { sprite: pallete_sprite, slot: EquipmentType::TorsoGarment, cached_texture: RefCell::new(None) }),
            material: None,
            quality: None,
            mellee_damage: None,
            armor: Some(ArmorBlueprintComponent { protection: DamageModel::new_spb(1, 1, 0), coverage: vec!(BodyPart::Torso) }),
            artwork_scene: None,
            name_blueprint: None
        };
        self.item_blueprints.add("itb:shirt", shirt_blueprint);

        let image = ImageReader::open("./assets/sprites/species/human/pants_equipped.png").unwrap().decode().unwrap();
        let pallete_sprite = PalleteSprite::new(image);
        let image = ImageReader::open("./assets/sprites/species/human/pants_simple.png").unwrap().decode().unwrap();
        let placed_sprite = PalleteSprite::new(image);
        let shirt_blueprint = ItemBlueprint {
            name: String::from("pants"),
            placed_sprite, 
            action_provider: None,
            equippable: Some(EquippableComponent { sprite: pallete_sprite, slot: EquipmentType::Legs, cached_texture: RefCell::new(None) }),
            material: None,
            quality: None,
            mellee_damage: None,
            armor: Some(ArmorBlueprintComponent { protection: DamageModel::new_spb(1, 0, 0), coverage: vec!(BodyPart::LeftLeg, BodyPart::RightLeg) }),
            artwork_scene: None,
            name_blueprint: None
        };
        self.item_blueprints.add("itb:pants", shirt_blueprint);

        let image = ImageReader::open("./assets/sprites/species/human/boots_equipped.png").unwrap().decode().unwrap();
        let pallete_sprite = PalleteSprite::new(image);
        let image = ImageReader::open("./assets/sprites/species/human/boots.png").unwrap().decode().unwrap();
        let placed_sprite = PalleteSprite::new(image);
        let shirt_blueprint = ItemBlueprint {
            name: String::from("boots"),
            placed_sprite, 
            action_provider: None,
            equippable: Some(EquippableComponent { sprite: pallete_sprite, slot: EquipmentType::Feet, cached_texture: RefCell::new(None) }),
            material: None,
            quality: None,
            mellee_damage: None,
            armor: Some(ArmorBlueprintComponent { protection: DamageModel::new_spb(1, 1, 1), coverage: vec!(BodyPart::LeftLeg, BodyPart::RightLeg) }),
            artwork_scene: None,
            name_blueprint: None
        };
        self.item_blueprints.add("itb:boots", shirt_blueprint);

        let image = ImageReader::open("./assets/sprites/species/human/brigandine_equipped.png").unwrap().decode().unwrap();
        let pallete_sprite = PalleteSprite::new(image);
        let image = ImageReader::open("./assets/sprites/species/human/armor.png").unwrap().decode().unwrap();
        let placed_sprite = PalleteSprite::new(image);
        let shirt_blueprint = ItemBlueprint {
            name: String::from("brigandine"),
            placed_sprite, 
            action_provider: None,
            equippable: Some(EquippableComponent { sprite: pallete_sprite, slot: EquipmentType::TorsoInner, cached_texture: RefCell::new(None) }),
            material: None,
            quality: None,
            mellee_damage: None,
            armor: Some(ArmorBlueprintComponent { protection: DamageModel::new_spb(3, 3, 1), coverage: vec!(BodyPart::Torso) }),
            artwork_scene: None,
            name_blueprint: None
        };
        self.item_blueprints.add("itb:brigandine", shirt_blueprint);

        let image = ImageReader::open("./assets/sprites/species/human/cuirass_equipped.png").unwrap().decode().unwrap();
        let pallete_sprite = PalleteSprite::new(image);
        let image = ImageReader::open("./assets/sprites/species/human/cuirass.png").unwrap().decode().unwrap();
        let placed_sprite = PalleteSprite::new(image);
        let shirt_blueprint = ItemBlueprint {
            name: String::from("cuirass"),
            placed_sprite, 
            action_provider: None,
            equippable: Some(EquippableComponent { sprite: pallete_sprite, slot: EquipmentType::TorsoInner, cached_texture: RefCell::new(None) }),
            material: Some(MaterialBlueprintComponent {
                primary_tag_bitmask: MAT_TAG_METAL,
                secondary_tag_bitmask: None,
                details_tag_bitmask: None,
            }),
            quality: None,
            mellee_damage: None,
            armor: Some(ArmorBlueprintComponent { protection: DamageModel::new_spb(10, 10, 10), coverage: vec!(BodyPart::Torso) }),
            artwork_scene: None,
            name_blueprint: None
        };
        self.item_blueprints.add("itb:cuirass", shirt_blueprint);

        let image = ImageReader::open("./assets/sprites/species/human/crown_equipped.png").unwrap().decode().unwrap();
        let pallete_sprite = PalleteSprite::new(image);
        let image = ImageReader::open("./assets/sprites/species/human/crown.png").unwrap().decode().unwrap();
        let placed_sprite = PalleteSprite::new(image);
        let shirt_blueprint = ItemBlueprint {
            name: String::from("crown"),
            placed_sprite, 
            action_provider: None,
            equippable: Some(EquippableComponent { sprite: pallete_sprite, slot: EquipmentType::Head, cached_texture: RefCell::new(None) }),
            material: None,
            quality: None,
            mellee_damage: None,
            armor: Some(ArmorBlueprintComponent { protection: DamageModel::new_spb(1, 1, 0), coverage: vec!(BodyPart::Head) }),
            artwork_scene: None,
            name_blueprint: None
        };
        self.item_blueprints.add("itb:crown", shirt_blueprint);

        let image = ImageReader::open("./assets/sprites/species/human/tome_equipped.png").unwrap().decode().unwrap();
        let pallete_sprite = PalleteSprite::new(image);
        let image = ImageReader::open("./assets/sprites/species/human/spell_tome_firebolt.png").unwrap().decode().unwrap();
        let placed_sprite = PalleteSprite::new(image);
        let mace_blueprint = ItemBlueprint {
            name: String::from("spell tome (Fire Bolt)"),
            placed_sprite, 
            action_provider: Some(ActionProviderComponent { actions: vec!(actions.id_of("act:firebolt")) }),
            equippable: Some(EquippableComponent { sprite: pallete_sprite, slot: EquipmentType::Trinket, cached_texture: RefCell::new(None) }),
            material: None,
            quality: None,
            mellee_damage: None,
            armor: None,
            artwork_scene: None,
            name_blueprint: None,
        };
        self.item_blueprints.add("itb:tome_firebolt", mace_blueprint);

        let image = ImageReader::open("./assets/sprites/species/human/tome_equipped.png").unwrap().decode().unwrap();
        let pallete_sprite = PalleteSprite::new(image);
        let image = ImageReader::open("./assets/sprites/species/human/spell_tome_fireball.png").unwrap().decode().unwrap();
        let placed_sprite = PalleteSprite::new(image);
        let mace_blueprint = ItemBlueprint {
            name: String::from("spell tome (Fireball)"),
            placed_sprite, 
            action_provider: Some(ActionProviderComponent { actions: vec!(actions.id_of("act:fireball")) }),
            equippable: Some(EquippableComponent { sprite: pallete_sprite, slot: EquipmentType::Trinket, cached_texture: RefCell::new(None) }),
            material: None,
            quality: None,
            mellee_damage: None,
            armor: None,
            artwork_scene: None,
            name_blueprint: None,
        };
        self.item_blueprints.add("itb:tome_fireball", mace_blueprint);

        let image = ImageReader::open("./assets/sprites/species/human/tome_equipped.png").unwrap().decode().unwrap();
        let pallete_sprite = PalleteSprite::new(image);
        let image = ImageReader::open("./assets/sprites/species/human/spell_tome_teleport.png").unwrap().decode().unwrap();
        let placed_sprite = PalleteSprite::new(image);
        let mace_blueprint = ItemBlueprint {
            name: String::from("spell tome (Teleport)"),
            placed_sprite, 
            action_provider: Some(ActionProviderComponent { actions: vec!(actions.id_of("act:teleport")) }),
            equippable: Some(EquippableComponent { sprite: pallete_sprite, slot: EquipmentType::Trinket, cached_texture: RefCell::new(None) }),
            material: None,
            quality: None,
            mellee_damage: None,
            armor: None,
            artwork_scene: None,
            name_blueprint: None,
        };
        self.item_blueprints.add("itb:tome_teleport", mace_blueprint);

        let image = ImageReader::open("./assets/sprites/species/human/tome_equipped.png").unwrap().decode().unwrap();
        let pallete_sprite = PalleteSprite::new(image);
        let image = ImageReader::open("./assets/sprites/species/human/spell_tome_rockpillar.png").unwrap().decode().unwrap();
        let placed_sprite = PalleteSprite::new(image);
        let mace_blueprint = ItemBlueprint {
            name: String::from("spell tome (Rock Pillar)"),
            placed_sprite, 
            action_provider: Some(ActionProviderComponent { actions: vec!(actions.id_of("act:rockpillar")) }),
            equippable: Some(EquippableComponent { sprite: pallete_sprite, slot: EquipmentType::Trinket, cached_texture: RefCell::new(None) }),
            material: None,
            quality: None,
            mellee_damage: None,
            armor: None,
            artwork_scene: None,
            name_blueprint: None,
        };
        self.item_blueprints.add("itb:tome_rockpillar", mace_blueprint);

    }

}