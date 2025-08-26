use text::capitalize;

use crate::{commons::{rng::Rng}, engine::geometry::Coord2, resources::resources::Resources, world::{history_sim::factories::CreatureFactory, site::*, world::World}};

pub(crate) fn spawn_random_village(world: &mut World, rng: &mut Rng, resources: &Resources, population: u32) -> Result<SiteId, ()> {
    let pos = search_new_site_pos(world, rng)?;

    let name = resources.cultures.get(&resources.cultures.random()).city_name_model.generate(rng, 3, 10);
    let name = capitalize(&name);
    let mut site = Site {
        xy: pos,
        creatures: Vec::new(),
        cemetery: Vec::new(),
        resources: SiteResources {
            // Enough food for a year
            food: population as f32
        },
        name: Some(name),
        settlement: Some(SettlementComponent {
            leader: None,
            material_stock: Vec::new()
        }),
        artifacts: Vec::new(),
        population_peak: (0, 0),
        site_type: SiteType::Village,
        structures: Vec::new()
    };

    site.structures.push(Structure::new(StructureType::TownHall));

    while site.creatures.len() < population as usize {
        
        let mut factory = CreatureFactory::new(rng.clone());
        let date = world.date.clone();
        let family = factory.make_family_or_single(&date, resources.species.id_of("species:human"), world, &resources);
        let mut structure = Structure::new(StructureType::House);
        for creature_id in family {
            site.creatures.push(creature_id);
            structure.add_ocuppant(creature_id);
        }

        site.structures.push(structure);

        rng.next();

    }

    return Ok(world.sites.add::<SiteId>(site));
}

fn search_new_site_pos(world: &World, rng: &mut Rng) -> Result<Coord2, ()> {
    for _ in 0..100 {
        let x = rng.randu_range(3, world.map.size.x() - 3);
        let y = rng.randu_range(3, world.map.size.y() - 3);
        let candidate = Coord2::xy(x as i32, y as i32);
        let too_close = world.sites.iter().any(|site| {
            let site = site.borrow();
            if site.creatures.len() == 0 {
                if site.xy == candidate {
                    return true;
                }
                return false;
            }
            return site.xy.dist_squared(&candidate) < 3. * 3.
        });
        if too_close {
            continue;
        }
        return Ok(candidate)
    }
    return Err(());
}
