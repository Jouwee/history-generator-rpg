use std::time::Instant;

use image::ImageReader;

use crate::{commons::{damage_model::DamageComponent, resource_map::ResourceMap}, engine::{asset::{image::ImageAsset, image_sheet::ImageSheetAsset}, audio::SoundEffect, geometry::{Coord2, Size2D}, pallete_sprite::PalleteSprite, tilemap::{Tile16Subset, TileRandom, TileSingle}, Color}, game::{actor::health_component::BodyPart, inventory::inventory::EquipmentType}, resources::material::{MAT_TAG_BONE, MAT_TAG_METAL, MAT_TAG_WOOD}, world::{attributes::Attributes, item::{ActionProviderComponent, ArmorComponent, EquippableComponent}}, MarkovChainSingleWordModel};

use super::{action::{Action, ActionType, Actions, Affliction, AfflictionChance, DamageType, Infliction}, biome::{Biome, Biomes}, culture::{Culture, Cultures}, item_blueprint::{ArtworkSceneBlueprintComponent, ItemBlueprint, ItemBlueprints, MaterialBlueprintComponent, MelleeDamageBlueprintComponent, NameBlueprintComponent, QualityBlueprintComponent}, material::{Material, Materials}, object_tile::{ObjectTile, ObjectTileId}, species::{Species, SpeciesApearance, SpeciesIntelligence, SpeciesMap}, tile::{Tile, TileId}};

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
        self.load_actions();
        self.load_species();
        self.load_cultures();
        self.load_biomes();
        self.load_tiles();
        self.load_object_tiles();
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
        bone.extra_damage = DamageComponent::arcane(5.);
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
            icon: ImageAsset::new("gui/icons/actions/slashing_cut.png"),
            sound_effect: Some(SoundEffect::new(vec!("sfx/sword_1.mp3", "sfx/sword_2.mp3", "sfx/sword_3.mp3"))),
            ap_cost: 40,
            stamina_cost: 5.,
            action_type: ActionType::Targeted {
                damage: Some(DamageType::FromWeapon(DamageComponent::new(1., 0., 0.))),
                inflicts: None
            }
        });
        self.actions.add("act:sword:bleeding_cut", Action {
            name: String::from("Bleeding Cut"),
            description: String::from("A deep cut that causes bleeding"),
            icon: ImageAsset::new("gui/icons/actions/bleeding_cut.png"),
            sound_effect: Some(SoundEffect::new(vec!("sfx/sword_1.mp3", "sfx/sword_2.mp3", "sfx/sword_3.mp3"))),
            ap_cost: 60,
            stamina_cost: 20.,
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
            icon: ImageAsset::new("gui/icons/actions/mace_smash.png"),
            sound_effect: Some(SoundEffect::new(vec!("sfx/punch_1.mp3", "sfx/punch_2.mp3"))),
            ap_cost: 40,
            stamina_cost: 5.,
            action_type: ActionType::Targeted {
                damage: Some(DamageType::FromWeapon(DamageComponent::new(0., 0., 1.))),
                inflicts: None
            }
        });
        self.actions.add("act:mace:concussive_strike", Action {
            name: String::from("Concussive Strike"),
            description: String::from("An aimed hit at the head"),
            icon: ImageAsset::new("gui/icons/actions/concussive_strike.png"),
            sound_effect: Some(SoundEffect::new(vec!("sfx/punch_1.mp3", "sfx/punch_2.mp3"))),
            ap_cost: 60,
            stamina_cost: 20.,
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
            icon: ImageAsset::new("gui/icons/actions/unarmed_attack.png"),
            sound_effect: Some(SoundEffect::new(vec!("sfx/punch_1.mp3", "sfx/punch_2.mp3"))),
            ap_cost: 40,
            stamina_cost: 5.,
            action_type: ActionType::Targeted {
                damage: Some(DamageType::Fixed(DamageComponent::new(0., 0., 1.))),
                inflicts: None
            }
        });
        self.actions.add("act:spider_bite", Action {
            name: String::from("Bite"),
            description: String::from("A spider bite"),
            icon: ImageAsset::new("missing.png"),
            sound_effect: Some(SoundEffect::new(vec!("sfx/monster_bite.mp3"))),
            ap_cost: 40,
            stamina_cost: 5.,
            action_type: ActionType::Targeted {
                damage: Some(DamageType::Fixed(DamageComponent::new(0., 10., 0.))),
                inflicts: Some(Infliction {
                    chance: AfflictionChance::OnHit,
                    affliction: Affliction::Poisoned { duration: 10 }
                })
            }
        });
        self.actions.add("act:bite", Action {
            name: String::from("Bite"),
            description: String::from("A bite"),
            icon: ImageAsset::new("missing.png"),
            sound_effect: Some(SoundEffect::new(vec!("sfx/monster_bite.mp3"))),
            ap_cost: 40,
            stamina_cost: 5.,
            action_type: ActionType::Targeted {
                damage: Some(DamageType::Fixed(DamageComponent::new(0., 10., 0.))),
                inflicts: None
            }
        });
        // self.actions.add("act:talk", Action {
        //     name: String::from("Talk"),
        //     description: String::from("Talk with a friendly NPC"),
        //     icon: ImageAsset::new("gui/icons/actions/talk.png"),
        //     sound_effect: None,
        //     ap_cost: 0,
        //     stamina_cost: 0.,
        //     action_type: ActionType::Talk
        // });
        self.actions.add("act:inspect", Action {
            name: String::from("Inspect"),
            description: String::from("Inspect something"),
            icon: ImageAsset::new("gui/icons/actions/inspect.png"),
            sound_effect: None,
            ap_cost: 0,
            stamina_cost: 0.,
            action_type: ActionType::Inspect
        });
        self.actions.add("act:dig", Action {
            name: String::from("Dig"),
            description: String::from("Dig the ground"),
            icon: ImageAsset::new("gui/icons/actions/dig.png"),
            sound_effect: None,
            ap_cost: 0,
            stamina_cost: 0.,
            action_type: ActionType::Dig
        });
        self.actions.add("act:pickup", Action {
            name: String::from("Pick Up"),
            description: String::from("Pick up something from the ground"),
            icon: ImageAsset::new("gui/icons/actions/pickup.png"),
            sound_effect: None,
            ap_cost: 20,
            stamina_cost: 1.,
            action_type: ActionType::PickUp
        });
        self.actions.add("act:sleep", Action {
            name: String::from("Sleep"),
            description: String::from("Sleep in a bed"),
            icon: ImageAsset::new("gui/icons/actions/sleep.png"),
            sound_effect: None,
            ap_cost: 0,
            stamina_cost: 0.,
            action_type: ActionType::Sleep
        });
        self.actions.add("act:move_left", Action {
            name: String::from("Move Left"),
            description: String::from("Move"),
            icon: ImageAsset::new("gui/icons/actions/sleep.png"),
            sound_effect: None,
            ap_cost: 20,
            stamina_cost: 0.2,
            action_type: ActionType::Move { offset: Coord2::xy(-1, 0) }
        });
        self.actions.add("act:move_right", Action {
            name: String::from("Move Right"),
            description: String::from("Move"),
            icon: ImageAsset::new("gui/icons/actions/sleep.png"),
            sound_effect: None,
            ap_cost: 20,
            stamina_cost: 0.2,
            action_type: ActionType::Move { offset: Coord2::xy(1, 0) }
        });
        self.actions.add("act:move_up", Action {
            name: String::from("Move Up"),
            description: String::from("Move"),
            icon: ImageAsset::new("gui/icons/actions/sleep.png"),
            sound_effect: None,
            ap_cost: 20,
            stamina_cost: 0.2,
            action_type: ActionType::Move { offset: Coord2::xy(0, -1) }
        });
        self.actions.add("act:move_down", Action {
            name: String::from("Move Down"),
            description: String::from("Move"),
            icon: ImageAsset::new("gui/icons/actions/sleep.png"),
            sound_effect: None,
            ap_cost: 20,
            stamina_cost: 0.2,
            action_type: ActionType::Move { offset: Coord2::xy(0, 1) }
        });
    }

    fn load_species(&mut self) {
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
                ))
            )
        )).innate_actions(vec!(self.actions.id_of("act:punch"))));

        self.species.add("species:spider", Species::new("spider", SpeciesApearance::single_sprite("species/spider.png"))
            .intelligence(SpeciesIntelligence::Instinctive)
            .attributes(Attributes { strength: 5, agility: 12, constitution: 10, unallocated: 0 })
            .innate_actions(vec!(self.actions.id_of("act:spider_bite")))
        );

        self.species.add("species:varningr", Species::new("varningr", SpeciesApearance::single_sprite("species/varningr/varningr.png"))
            .intelligence(SpeciesIntelligence::Instinctive)
            .attributes(Attributes { strength: 5, agility: 12, constitution: 10, unallocated: 0 })
            .drops(vec!(self.materials.id_of("mat:varningr_bone")))
            // TODO:
            .innate_actions(vec!(self.actions.id_of("act:bite")))
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
    }

    pub(crate) fn load_object_tiles(&mut self) {
        
        let image = ImageSheetAsset::new("chunk_tiles/stone_walls.png", Size2D(24, 48));
        self.object_tiles.add("obj:wall", ObjectTile::new(crate::engine::tilemap::Tile::T16Subset(Tile16Subset::new(image)), true));

        let image = ImageSheetAsset::new("chunk_tiles/tree.png", Size2D(64, 64));
        self.object_tiles.add("obj:tree", ObjectTile::new(crate::engine::tilemap::Tile::TileRandom(TileRandom::new(image)), true));

        let image = ImageAsset::new("bed.png");
        self.object_tiles.add("obj:bed", ObjectTile::new(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)), true));

        let image = ImageSheetAsset::new("chunk_tiles/wood_small_table.png", Size2D(24, 24));
        self.object_tiles.add("obj:table", ObjectTile::new(crate::engine::tilemap::Tile::TileRandom(TileRandom::new(image)), true));

        let image = ImageSheetAsset::new("chunk_tiles/wood_stool.png", Size2D(24, 24));
        self.object_tiles.add("obj:stool", ObjectTile::new(crate::engine::tilemap::Tile::TileRandom(TileRandom::new(image)), true));
        
        let image = ImageSheetAsset::new("chunk_tiles/tombstone.png", Size2D(24, 24));
        self.object_tiles.add("obj:tombstone", ObjectTile::new(crate::engine::tilemap::Tile::TileRandom(TileRandom::new(image)), true));
        
        let image = ImageAsset::new("chunk_tiles/anvil.png");
        self.object_tiles.add("obj:anvil", ObjectTile::new(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)), true));
        
        let image = ImageAsset::new("chunk_tiles/barrel.png");
        self.object_tiles.add("obj:barrel", ObjectTile::new(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)), true));
        
        let image = ImageSheetAsset::new("chunk_tiles/grass_decal.png", Size2D(24, 24));
        self.object_tiles.add("obj:grass_decal", ObjectTile::new(crate::engine::tilemap::Tile::TileRandom(TileRandom::new(image)), false));

        let image = ImageAsset::new("chunk_tiles/tent.png");
        self.object_tiles.add("obj:tent", ObjectTile::new(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)), true));

        let image = ImageSheetAsset::new("chunk_tiles/pebbles.png", Size2D(24, 24));
        self.object_tiles.add("obj:pebbles", ObjectTile::new(crate::engine::tilemap::Tile::TileRandom(TileRandom::new(image)), false));

        let image = ImageSheetAsset::new("chunk_tiles/flowers.png", Size2D(24, 24));
        self.object_tiles.add("obj:flowers", ObjectTile::new(crate::engine::tilemap::Tile::TileRandom(TileRandom::new(image)), false));

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
            mellee_damage: Some(MelleeDamageBlueprintComponent { base_damage: DamageComponent::new(10., 0., 0.) }),
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
            mellee_damage: Some(MelleeDamageBlueprintComponent { base_damage: DamageComponent::new(0., 0., 10.) }),
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
            armor: Some(ArmorComponent { protection: DamageComponent::new(1., 8., 0.), coverage: vec!(BodyPart::Torso) }),
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
            armor: Some(ArmorComponent { protection: DamageComponent::new(1., 0., 0.), coverage: vec!(BodyPart::LeftLeg, BodyPart::RightLeg) }),
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
            armor: Some(ArmorComponent { protection: DamageComponent::new(1., 1., 0.), coverage: vec!(BodyPart::LeftLeg, BodyPart::RightLeg) }),
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
            armor: Some(ArmorComponent { protection: DamageComponent::new(3., 3., 1.), coverage: vec!(BodyPart::Torso) }),
            artwork_scene: None,
            name_blueprint: None
        };
        self.item_blueprints.add("itb:armor", shirt_blueprint);

    }

}