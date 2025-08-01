use std::time::Instant;

use image::ImageReader;

use crate::{commons::{damage_model::{DamageModel, DamageRoll}, resource_map::ResourceMap}, engine::{assets::ImageSheetAsset, audio::SoundEffect, geometry::Size2D, pallete_sprite::PalleteSprite, tilemap::{Tile16Subset, TileRandom, TileSingle}, Color}, game::{actor::health_component::BodyPart, inventory::inventory::EquipmentType}, resources::{action::{ActionArea, ActionEffect, ActionProjectile, ActionTarget, ImpactPosition, SpellProjectileType, FILTER_CAN_DIG, FILTER_CAN_OCCUPY, FILTER_CAN_SLEEP, FILTER_CAN_VIEW, FILTER_ITEM}, material::{MAT_TAG_BONE, MAT_TAG_METAL, MAT_TAG_WOOD}, species::SpeciesAppearance}, world::{attributes::Attributes, item::{ActionProviderComponent, ArmorComponent, EquippableComponent}}, MarkovChainSingleWordModel};
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
        self.materials.add("mat:steel", Material::new_metal("steel"));

        let mut bronze = Material::new_metal("bronze");
        bronze.color_pallete = [Color::from_hex("a57855"), Color::from_hex("de9f47"), Color::from_hex("fdd179"), Color::from_hex("fee1b8")];
        self.materials.add("mat:bronze", bronze);

        self.materials.add("mat:birch", Material::new_wood("birch"));

        self.materials.add("mat:oak", Material::new_wood("oak"));

        let mut copper = Material::new_metal("copper");
        copper.color_pallete = [Color::from_hex("593e47"), Color::from_hex("b55945"), Color::from_hex("de9f47"), Color::from_hex("f2b888")];
        self.materials.add("mat:copper", copper);

        let mut bone = Material::new_bone("varningr's bone");
        bone.extra_damage = DamageRoll::arcane("1d6");
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
        self.actions.add("act:sword:slash", Action {
            name: String::from("Slash"),
            description: String::from("A slashing strike"),
            icon: String::from("gui/icons/actions/slashing_cut.png"),
            log_use: true,
            cast_sfx: Some(SoundEffect::new(vec!("sfx/sword_1.mp3", "sfx/sword_2.mp3", "sfx/sword_3.mp3"))),
            ap_cost: 40,
            stamina_cost: 3.,
            cooldown: 0,
            target: ActionTarget::Actor { range: 1., filter_mask: 0 },
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
            target: ActionTarget::Actor { range: 1., filter_mask: 0 },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::Damage { add_weapon: true, damage: DamageRoll::empty() },
                ActionEffect::Inflicts { affliction: Affliction::Bleeding { duration: 5 } }
            ),
            cast_sprite: None,
            projectile: None,
            impact_sprite: None,
            impact_sfx: None,
            damage_sfx: None
        });
        self.actions.add("act:mace:smash", Action {
            name: String::from("Smash"),
            description: String::from("A heavy smash"),
            log_use: true,
            icon: String::from("gui/icons/actions/mace_smash.png"),
            cast_sfx: Some(SoundEffect::new(vec!("sfx/punch_1.mp3", "sfx/punch_2.mp3"))),
            ap_cost: 40,
            stamina_cost: 3.,
            cooldown: 0,
            target: ActionTarget::Actor { range: 1., filter_mask: 0 },
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
        self.actions.add("act:mace:concussive_strike", Action {
            name: String::from("Concussive Strike"),
            description: String::from("An aimed hit at the head"),
            log_use: true,
            icon: String::from("gui/icons/actions/concussive_strike.png"),
            cast_sfx: Some(SoundEffect::new(vec!("sfx/punch_1.mp3", "sfx/punch_2.mp3"))),
            ap_cost: 60,
            stamina_cost: 20.,
            cooldown: 2,
            target: ActionTarget::Actor { range: 1., filter_mask: 0 },
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
            target: ActionTarget::Actor { range: 1., filter_mask: 0 },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::Damage { add_weapon: false, damage: DamageRoll::bludgeoning("d4") },
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
            target: ActionTarget::Actor { range: 1., filter_mask: 0 },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::Damage { add_weapon: false, damage: DamageRoll::piercing("2d6+6") },
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
            target: ActionTarget::Actor { range: 1., filter_mask: 0 },
            area: ActionArea::Target,
            effects: vec!(
                ActionEffect::Damage { add_weapon: false, damage: DamageRoll::piercing("2d4") },
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
                ActionEffect::Damage { add_weapon: false, damage: DamageRoll::fire("2d6+4") },
                ActionEffect::Inflicts { affliction: Affliction::OnFire { duration: 5 } }
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
                ActionEffect::Damage { add_weapon: false, damage: DamageRoll::fire("3d6+12") },
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

        // self.actions.add("act:talk", Action {
        //     name: String::from("Talk"),
        //     description: String::from("Talk with a friendly NPC"),
        //     icon: String::from("gui/icons/actions/talk.png"),
        //     sound_effect: None,
        //     ap_cost: 0,
        //     stamina_cost: 0.,
        //     action_type: ActionType::Talk
        // });
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
            target: ActionTarget::Tile { range: 1., filter_mask: FILTER_CAN_DIG },
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
            target: ActionTarget::Tile { range: 1., filter_mask: FILTER_ITEM },
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
            target: ActionTarget::Tile { range: 1., filter_mask: FILTER_CAN_SLEEP },
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
            base: vec!("species/human/base_male_light.png".to_string(), "species/human/base_female_light.png".to_string()),
            top: vec!(
                "species/human/hair_bun.png".to_string(),
                "species/human/hair_short.png".to_string(),
                "species/human/hair_shaved.png".to_string(),
            )
        }).innate_actions(vec!(self.actions.id_of("act:punch"))));

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
            .innate_actions(vec!(self.actions.id_of("act:bite"), self.actions.id_of("act:deafening_howl")))
        );
    }

    fn load_cultures(&mut self) {
        self.cultures.add("culture:nord", Culture {
            first_name_male_model: MarkovChainSingleWordModel::train(vec!(
                "Alald", "Alan", "Alar", "Alarik", "Alarke", "Alarne", "Aleld", "Alen", "Alens",
                "Aler", "Alik", "Alis", "Alorn", "Asgald", "Asgan", "Asgar", "Asgarik", "Asgarke",
                "Asgarne", "Asgeld", "Asgen", "Asgens", "Asger", "Asgik", "Asgis", "Asgorn", "Bjald",
                "Bjan", "Bjar", "Bjarik", "Bjarke", "Bjarne", "Bjeld", "Bjen", "Bjens", "Bjer",
                "Bjik", "Bjis", "Bjorn", "Erald", "Eran", "Erar", "Erarik", "Erarke", "Erarne",
                "Ereld", "Eren", "Erens", "Erer", "Erik", "Eris", "Erorn", "Fenrald", "Fenran",
                "Fenrar", "Fenrarik", "Fenrarke", "Fenrarne", "Fenreld", "Fenren", "Fenrens",
                "Fenrer", "Fenrik", "Fenris", "Fenrorn", "Harald", "Haran", "Harar", "Hararik", 
                "Hararke", "Hararne", "Hareld", "Haren", "Harens", "Harer", "Harik", "Haris", 
                "Harorn", "Ingmald", "Ingman", "Ingmar", "Ingmarik", "Ingmarke", "Ingmarne", 
                "Ingmeld", "Ingmen", "Ingmens", "Ingmer", "Ingmik", "Ingmis", "Ingmorn", "Jurgald", 
                "Jurgan", "Jurgar", "Jurgarik", "Jurgarke", "Jurgarne", "Jurgeld", "Jurgen", 
                "Jurgens", "Jurger", "Jurgik", "Jurgis", "Jurgorn", "Kjald", "Kjan", "Kjar", "Kjarik", 
                "Kjarke", "Kjarne", "Kjeld", "Kjen", "Kjens", "Kjer", "Kjik", "Kjis", "Kjorn", "Mojald", 
                "Mojan", "Mojar", "Mojarik", "Mojarke", "Mojarne", "Mojeld", "Mojen", "Mojens", "Mojer", 
                "Mojik", "Mojis", "Mojorn", "Sorald", "Soran", "Sorar", "Sorarik", "Sorarke", "Sorarne", 
                "Soreld", "Soren", "Sorens", "Sorer", "Sorik", "Soris", "Sororn", "Torbald", "Torban", 
                "Torbar", "Torbarik", "Torbarke", "Torbarne", "Torbeld", "Torben", "Torbens", "Torber", 
                "Torbik", "Torbis", "Torborn", "Ulrald", "Ulran", "Ulrar", "Ulrarik", "Ulrarke", 
                "Ulrarne", "Ulreld", "Ulren", "Ulrens", "Ulrer", "Ulrik", "Ulris", "Ulrorn"
            ), 3),
            first_name_female_model: MarkovChainSingleWordModel::train(vec!(
                "Ana", "Ane", "Anen", "Ania", "Anina", "Anne", "Ante", "Beta", "Bete", "Beten",
                "Betia", "Betina", "Betne", "Bette", "Dora", "Dore", "Doren", "Doria", "Dorina",
                "Dorne", "Dorte", "Ella", "Elle", "Ellen", "Ellia", "Ellina", "Ellne", "Ellte",
                "Hana", "Hane", "Hanen", "Hania", "Hanina", "Hanne", "Hante", "Hella", "Helle",
                "Hellen", "Hellia", "Hellina", "Hellne", "Hellte", "Inga", "Inge", "Ingen", "Ingia",
                "Ingina", "Ingne", "Ingte", "Jyta", "Jyte", "Jyten", "Jytia", "Jytina", "Jytne",
                "Jytte", "Kirsta", "Kirste", "Kirsten", "Kirstia", "Kirstina", "Kirstne", "Kirstte",
                "Meta", "Mete", "Meten", "Metia", "Metina", "Metne", "Mette", "Morga", "Morge",
                "Morgen", "Morgia", "Morgina", "Morgne", "Morgte", "Silla", "Sille", "Sillen",
                "Sillia", "Sillina", "Sillne", "Sillte", "Ulla", "Ulle", "Ullen", "Ullia", "Ullina",
                "Ullne", "Ullte"
            ), 3),
            last_name_model: MarkovChainSingleWordModel::train(vec!(
                "Alaldsen", "Alansen", "Alarsen", "Alariksen", "Alarkesen", "Alarnesen", "Aleldsen",
                "Alensen", "Alenssen", "Alersen", "Aliksen", "Alissen", "Alornsen", "Asgaldsen",
                "Asgansen", "Asgarsen", "Asgariksen", "Asgarkesen", "Asgarnesen", "Asgeldsen",
                "Asgensen", "Asgenssen", "Asgersen", "Asgiksen", "Asgissen", "Asgornsen",
                "Bjaldsen", "Bjansen", "Bjarsen", "Bjariksen", "Bjarkesen", "Bjarnesen", "Bjeldsen",
                "Bjensen", "Bjenssen", "Bjersen", "Bjiksen", "Bjissen", "Bjornsen", "Eraldsen",
                "Eransen", "Erarsen", "Erariksen", "Erarkesen", "Erarnesen", "Ereldsen", "Erensen",
                "Erenssen", "Erersen", "Eriksen", "Erissen", "Erornsen", "Fenraldsen", "Fenransen",
                "Fenrarsen", "Fenrariksen", "Fenrarkesen", "Fenrarnesen", "Fenreldsen", "Fenrensen",
                "Fenrenssen", "Fenrersen", "Fenriksen", "Fenrissen", "Fenrornsen", "Haraldsen",
                "Haransen", "Hararsen", "Harariksen", "Hararkesen", "Hararnesen", "Hareldsen",
                "Harensen", "Harenssen", "Harersen", "Hariksen", "Harissen", "Harornsen",
                "Ingmaldsen", "Ingmansen", "Ingmarsen", "Ingmariksen", "Ingmarkesen", "Ingmarnesen",
                "Ingmeldsen", "Ingmensen", "Ingmenssen", "Ingmersen", "Ingmiksen", "Ingmissen",
                "Ingmornsen", "Jurgaldsen", "Jurgansen", "Jurgarsen", "Jurgariksen", "Jurgarkesen",
                "Jurgarnesen", "Jurgeldsen", "Jurgensen", "Jurgenssen", "Jurgersen", "Jurgiksen",
                "Jurgissen", "Jurgornsen", "Kjaldsen", "Kjansen", "Kjarsen", "Kjariksen", "Kjarkesen",
                "Kjarnesen", "Kjeldsen", "Kjensen", "Kjenssen", "Kjersen", "Kjiksen", "Kjissen",
                "Kjornsen", "Mojaldsen", "Mojansen", "Mojarsen", "Mojariksen", "Mojarkesen",
                "Mojarnesen", "Mojeldsen", "Mojensen", "Mojenssen", "Mojersen", "Mojiksen",
                "Mojissen", "Mojornsen", "Soraldsen", "Soransen", "Sorarsen", "Sorariksen",
                "Sorarkesen", "Sorarnesen", "Soreldsen", "Sorensen", "Sorenssen", "Sorersen",
                "Soriksen", "Sorissen", "Sorornsen", "Torbaldsen", "Torbansen", "Torbarsen",
                "Torbariksen", "Torbarkesen", "Torbarnesen", "Torbeldsen", "Torbensen", "Torbenssen",
                "Torbersen", "Torbiksen", "Torbissen", "Torbornsen", "Ulraldsen", "Ulransen",
                "Ulrarsen", "Ulrariksen", "Ulrarkesen", "Ulrarnesen", "Ulreldsen", "Ulrensen",
                "Ulrenssen", "Ulrersen", "Ulriksen", "Ulrissen", "Ulrornsen"
            ), 3)
        });

        self.cultures.add("culture:khajit", Culture {
            first_name_male_model: MarkovChainSingleWordModel::train(vec!(
                "Ab'ar", "Ab'bar", "Ab'bil", "Ab'der", "Ab'dul", "Ab'gh", "Ab'ir", "Ab'kir", "Ab'med", "Ab'nir", "Ab'noud", "Ab'sien", "Ab'soud", "Ab'taba", "Ab'tabe", "Ab'urabi", "Ak'ar", "Ak'bar", "Ak'bil", "Ak'der", "Ak'dul", "Ak'gh", "Ak'ir", "Ak'kir", "Ak'med", "Ak'nir", "Ak'noud", "Ak'sien", "Ak'soud", "Ak'taba", "Ak'tabe", "Ak'urabi", "Akh'ar", "Akh'bar", "Akh'bil", "Akh'der", "Akh'dul", "Akh'gh", "Akh'ir", "Akh'kir", "Akh'med", "Akh'nir", "Akh'noud", "Akh'sien", "Akh'soud", "Akh'taba", "Akh'tabe", "Akh'urabi", "Amar", "Ambar", "Ambil", "Amder", "Amdul", "Amgh", "Amir", "Amkir", "Ammed", "Amnir", "Amnoud", "Amsien", "Amsoud", "Amtaba", "Amtabe", "Amurabi", "Fa'ar", "Fa'bar", "Fa'bil", "Fa'der", "Fa'dul", "Fa'gh", "Fa'ir", "Fa'kir", "Fa'med", "Fa'nir", "Fa'noud", "Fa'sien", "Fa'soud", "Fa'taba", "Fa'tabe", "Fa'urabi", "Husar", "Husbar", "Husbil", "Husder", "Husdul", "Husgh", "Husir", "Huskir", "Husmed", "Husnir", "Husnoud", "Hussien", "Hussoud", "Hustaba", "Hustabe", "Husurabi", "Moar", "Mobar", "Mobil", "Moder", "Modul", "Mogh", "Moir", "Mokir", "Momed", "Monir", "Monoud", "Mosien", "Mosoud", "Motaba", "Motabe", "Mourabi", "Mohamar", "Mohambar", "Mohambil", "Mohamder", "Mohamdul", "Mohamgh", "Mohamir", "Mohamkir", "Mohammed", "Mohamnir", "Mohamnoud", "Mohamsien", "Mohamsoud", "Mohamtaba", "Mohamtabe", "Mohamurabi", "Mojar", "Mojbar", "Mojbil", "Mojder", "Mojdul", "Mojgh", "Mojir", "Mojkir", "Mojmed", "Mojnir", "Mojnoud", "Mojsien", "Mojsoud", "Mojtaba", "Mojtabe", "Mojurabi", "Naar", "Nabar", "Nabil", "Nader", "Nadul", "Nagh", "Nair", "Nakir", "Named", "Nanir", "Nanoud", "Nasien", "Nasoud", "Nataba", "Natabe", "Naurabi", "Omar", "Ombar", "Ombil", "Omder", "Omdul", "Omgh", "Omir", "Omkir", "Ommed", "Omnir", "Omnoud", "Omsien", "Omsoud", "Omtaba", "Omtabe", "Omurabi", "Shaar", "Shabar", "Shabil", "Shader", "Shadul", "Shagh", "Shair", "Shakir", "Shamed", "Shanir", "Shanoud", "Shasien", "Shasoud", "Shataba", "Shatabe", "Shaurabi", "Sinar", "Sinbar", "Sinbil", "Sinder", "Sindul", "Singh", "Sinir", "Sinkir", "Sinmed", "Sinnir", "Sinnoud", "Sinsien", "Sinsoud", "Sintaba", "Sintabe", "Sinurabi", "Za'ar", "Za'bar", "Za'bil", "Za'der", "Za'dul", "Za'gh", "Za'ir", "Za'kir", "Za'med", "Za'nir", "Za'noud", "Za'sien", "Za'soud", "Za'taba", "Za'tabe", "Za'urabi", "Zan'ar", "Zan'bar", "Zan'bil", "Zan'der", "Zan'dul", "Zan'gh", "Zan'ir", "Zan'kir", "Zan'med", "Zan'nir", "Zan'noud", "Zan'sien", "Zan'soud", "Zan'taba", "Zan'tabe", "Zan'urabi",
            ), 3),
            first_name_female_model: MarkovChainSingleWordModel::train(vec!(
                "Aahin", "Aahni", "Afeliz", "Ahana", "Aheh", "Ahrazad", "Ajjan", "Akhtar", "Anita", "Araya", "Ariba", "Ashima", "Asrin", "Atima", "Azita", "Aziahin", "Aziahni", "Azifeliz", "Azihana", "Aziheh", "Azihrazad", "Azijjan", "Azikhtar", "Azinita", "Aziraya", "Aziriba", "Azishima", "Azisrin", "Azitima", "Azizita", "Elaahin", "Elaahni", "Elafeliz", "Elahana", "Elaheh", "Elahrazad", "Elajjan", "Elakhtar", "Elanita", "Elaraya", "Elariba", "Elashima", "Elasrin", "Elatima", "Elazita", "Faahin", "Faahni", "Fafeliz", "Fahana", "Faheh", "Fahrazad", "Fajjan", "Fakhtar", "Fanita", "Faraya", "Fariba", "Fashima", "Fasrin", "Fatima", "Fazita", "Khaahin", "Khaahni", "Khafeliz", "Khahana", "Khaheh", "Khahrazad", "Khajjan", "Khakhtar", "Khanita", "Kharaya", "Khariba", "Khashima", "Khasrin", "Khatima", "Khazita", "Kiahin", "Kiahni", "Kifeliz", "Kihana", "Kiheh", "Kihrazad", "Kijjan", "Kikhtar", "Kinita", "Kiraya", "Kiriba", "Kishima", "Kisrin", "Kitima", "Kizita", "Moahin", "Moahni", "Mofeliz", "Mohana", "Moheh", "Mohrazad", "Mojjan", "Mokhtar", "Monita", "Moraya", "Moriba", "Moshima", "Mosrin", "Motima", "Mozita", "Naahin", "Naahni", "Nafeliz", "Nahana", "Naheh", "Nahrazad", "Najjan", "Nakhtar", "Nanita", "Naraya", "Nariba", "Nashima", "Nasrin", "Natima", "Nazita", "Raahin", "Raahni", "Rafeliz", "Rahana", "Raheh", "Rahrazad", "Rajjan", "Rakhtar", "Ranita", "Raraya", "Rariba", "Rashima", "Rasrin", "Ratima", "Razita", "Riahin", "Riahni", "Rifeliz", "Rihana", "Riheh", "Rihrazad", "Rijjan", "Rikhtar", "Rinita", "Riraya", "Ririba", "Rishima", "Risrin", "Ritima", "Rizita", "Saahin", "Saahni", "Safeliz", "Sahana", "Saheh", "Sahrazad", "Sajjan", "Sakhtar", "Sanita", "Saraya", "Sariba", "Sashima", "Sasrin", "Satima", "Sazita", "Shaahin", "Shaahni", "Shafeliz", "Shahana", "Shaheh", "Shahrazad", "Shajjan", "Shakhtar", "Shanita", "Sharaya", "Shariba", "Shashima", "Shasrin", "Shatima", "Shazita", "Soahin", "Soahni", "Sofeliz", "Sohana", "Soheh", "Sohrazad", "Sojjan", "Sokhtar", "Sonita", "Soraya", "Soriba", "Soshima", "Sosrin", "Sotima", "Sozita", "Taahin", "Taahni", "Tafeliz", "Tahana", "Taheh", "Tahrazad", "Tajjan", "Takhtar", "Tanita", "Taraya", "Tariba", "Tashima", "Tasrin", "Tatima", "Tazita", "Zaahin", "Zaahni", "Zafeliz", "Zahana", "Zaheh", "Zahrazad", "Zajjan", "Zakhtar", "Zanita", "Zaraya", "Zariba", "Zashima", "Zasrin", "Zatima", "Zazita", 
            ), 3),
            last_name_model: MarkovChainSingleWordModel::train(vec!(
                "Abiri", "Abus", "Adavi", "Ahan", "Ahir", "Akar", "Amanni", "Amnin", "Anai", "Aoni", "Arabi", "Aspoor", "Astae", "Atani", "Avandi", "Barabiri", "Barabus", "Baradavi", "Barahan", "Barahir", "Barakar", "Baramanni", "Baramnin", "Baranai", "Baraoni", "Bararabi", "Baraspoor", "Barastae", "Baratani", "Baravandi", "Hammubiri", "Hammubus", "Hammudavi", "Hammuhan", "Hammuhir", "Hammukar", "Hammumanni", "Hammumnin", "Hammunai", "Hammuoni", "Hammurabi", "Hammuspoor", "Hammustae", "Hammutani", "Hammuvandi", "Jabiri", "Jabus", "Jadavi", "Jahan", "Jahir", "Jakar", "Jamanni", "Jamnin", "Janai", "Jaoni", "Jarabi", "Jaspoor", "Jastae", "Jatani", "Javandi", "Khabiri", "Khabus", "Khadavi", "Khahan", "Khahir", "Khakar", "Khamanni", "Khamnin", "Khanai", "Khaoni", "Kharabi", "Khaspoor", "Khastae", "Khatani", "Khavandi", "Kibiri", "Kibus", "Kidavi", "Kihan", "Kihir", "Kikar", "Kimanni", "Kimnin", "Kinai", "Kioni", "Kirabi", "Kispoor", "Kistae", "Kitani", "Kivandi", "Mahbiri", "Mahbus", "Mahdavi", "Mahhan", "Mahhir", "Mahkar", "Mahmanni", "Mahmnin", "Mahnai", "Mahoni", "Mahrabi", "Mahspoor", "Mahstae", "Mahtani", "Mahvandi", "Raibiri", "Raibus", "Raidavi", "Raihan", "Raihir", "Raikar", "Raimanni", "Raimnin", "Rainai", "Raioni", "Rairabi", "Raispoor", "Raistae", "Raitani", "Raivandi", "Robiri", "Robus", "Rodavi", "Rohan", "Rohir", "Rokar", "Romanni", "Romnin", "Ronai", "Rooni", "Rorabi", "Rospoor", "Rostae", "Rotani", "Rovandi", "Sabiri", "Sabus", "Sadavi", "Sahan", "Sahir", "Sakar", "Samanni", "Samnin", "Sanai", "Saoni", "Sarabi", "Saspoor", "Sastae", "Satani", "Savandi", "Sibiri", "Sibus", "Sidavi", "Sihan", "Sihir", "Sikar", "Simanni", "Simnin", "Sinai", "Sioni", "Sirabi", "Sispoor", "Sistae", "Sitani", "Sivandi", "Solbiri", "Solbus", "Soldavi", "Solhan", "Solhir", "Solkar", "Solmanni", "Solmnin", "Solnai", "Soloni", "Solrabi", "Solspoor", "Solstae", "Soltani", "Solvandi", "Tavakbiri", "Tavakbus", "Tavakdavi", "Tavakhan", "Tavakhir", "Tavakkar", "Tavakmanni", "Tavakmnin", "Tavaknai", "Tavakoni", "Tavakrabi", "Tavakspoor", "Tavakstae", "Tavaktani", "Tavakvandi", "Zabiri", "Zabus", "Zadavi", "Zahan", "Zahir", "Zakar", "Zamanni", "Zamnin", "Zanai", "Zaoni", "Zarabi", "Zaspoor", "Zastae", "Zatani", "Zavandi", 
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
            action_provider: Some(ActionProviderComponent { actions: vec!(actions.id_of("act:sword:slash"), actions.id_of("act:sword:bleeding_cut")) }),
            equippable: Some(EquippableComponent { sprite: pallete_sprite, slot: EquipmentType::Hand }),
            material: Some(MaterialBlueprintComponent {
                primary_tag_bitmask: MAT_TAG_METAL,
                secondary_tag_bitmask: Some(MAT_TAG_WOOD | MAT_TAG_BONE),
                details_tag_bitmask: Some(MAT_TAG_WOOD | MAT_TAG_BONE | MAT_TAG_METAL),
            }),
            quality: Some(QualityBlueprintComponent { }),
            mellee_damage: Some(MelleeDamageBlueprintComponent { base_damage: DamageRoll::slashing("1d6") }),
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

        let image = ImageReader::open("./assets/sprites/species/human/mace_equipped.png").unwrap().decode().unwrap();
        let pallete_sprite = PalleteSprite::new(image);
        let image = ImageReader::open("./assets/sprites/mace.png").unwrap().decode().unwrap();
        let placed_sprite = PalleteSprite::new(image);
        let mace_blueprint = ItemBlueprint {
            name: String::from("mace"),
            placed_sprite, 
            action_provider: Some(ActionProviderComponent { actions: vec!(actions.id_of("act:mace:smash"), actions.id_of("act:mace:concussive_strike")) }),
            equippable: Some(EquippableComponent { sprite: pallete_sprite, slot: EquipmentType::Hand }),
            material: Some(MaterialBlueprintComponent {
                primary_tag_bitmask: MAT_TAG_METAL,
                secondary_tag_bitmask: Some(MAT_TAG_WOOD | MAT_TAG_BONE),
                details_tag_bitmask: Some(MAT_TAG_WOOD | MAT_TAG_BONE | MAT_TAG_METAL),
            }),
            quality: Some(QualityBlueprintComponent { }),
            mellee_damage: Some(MelleeDamageBlueprintComponent { base_damage: DamageRoll::bludgeoning("1d6") }),
            armor: None,
            artwork_scene: None,
            name_blueprint: Some(NameBlueprintComponent { suffixes: vec!(String::from("breaker"), String::from("kiss"), String::from("fist"), String::from("touch")) })
        };
        self.item_blueprints.add("itb:mace", mace_blueprint);


        let image = ImageReader::open("./assets/sprites/species/human/peasant_shirt_equipped.png").unwrap().decode().unwrap();
        let pallete_sprite = PalleteSprite::new(image);
        let image = ImageReader::open("./assets/sprites/species/human/peasant_shirt.png").unwrap().decode().unwrap();
        let placed_sprite = PalleteSprite::new(image);
        let shirt_blueprint = ItemBlueprint {
            name: String::from("peasant shirt"),
            placed_sprite, 
            action_provider: None,
            equippable: Some(EquippableComponent { sprite: pallete_sprite, slot: EquipmentType::TorsoGarment }),
            material: None,
            quality: None,
            mellee_damage: None,
            armor: Some(ArmorComponent { protection: DamageModel::new_spb(1, 8, 0), coverage: vec!(BodyPart::Torso) }),
            artwork_scene: None,
            name_blueprint: None
        };
        self.item_blueprints.add("itb:shirt", shirt_blueprint);

        let image = ImageReader::open("./assets/sprites/species/human/pants_simple_equipped.png").unwrap().decode().unwrap();
        let pallete_sprite = PalleteSprite::new(image);
        let image = ImageReader::open("./assets/sprites/species/human/pants_simple.png").unwrap().decode().unwrap();
        let placed_sprite = PalleteSprite::new(image);
        let shirt_blueprint = ItemBlueprint {
            name: String::from("pants"),
            placed_sprite, 
            action_provider: None,
            equippable: Some(EquippableComponent { sprite: pallete_sprite, slot: EquipmentType::Legs }),
            material: None,
            quality: None,
            mellee_damage: None,
            armor: Some(ArmorComponent { protection: DamageModel::new_spb(1, 0, 0), coverage: vec!(BodyPart::LeftLeg, BodyPart::RightLeg) }),
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
            equippable: Some(EquippableComponent { sprite: pallete_sprite, slot: EquipmentType::Feet }),
            material: None,
            quality: None,
            mellee_damage: None,
            armor: Some(ArmorComponent { protection: DamageModel::new_spb(1, 1, 0), coverage: vec!(BodyPart::LeftLeg, BodyPart::RightLeg) }),
            artwork_scene: None,
            name_blueprint: None
        };
        self.item_blueprints.add("itb:boots", shirt_blueprint);

        let image = ImageReader::open("./assets/sprites/species/human/armor_equipped.png").unwrap().decode().unwrap();
        let pallete_sprite = PalleteSprite::new(image);
        let image = ImageReader::open("./assets/sprites/species/human/armor.png").unwrap().decode().unwrap();
        let placed_sprite = PalleteSprite::new(image);
        let shirt_blueprint = ItemBlueprint {
            name: String::from("armor"),
            placed_sprite, 
            action_provider: None,
            equippable: Some(EquippableComponent { sprite: pallete_sprite, slot: EquipmentType::TorsoInner }),
            material: None,
            quality: None,
            mellee_damage: None,
            armor: Some(ArmorComponent { protection: DamageModel::new_spb(3, 3, 1), coverage: vec!(BodyPart::Torso) }),
            artwork_scene: None,
            name_blueprint: None
        };
        self.item_blueprints.add("itb:armor", shirt_blueprint);

    }

}