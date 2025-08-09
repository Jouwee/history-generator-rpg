use crate::{commons::{rng::Rng, strings::Strings}, engine::geometry::Coord2, resources::resources::Resources, world::{history_sim::factories::CreatureFactory, unit::*, world::World}};

pub(crate) fn spawn_random_village(world: &mut World, rng: &mut Rng, resources: &Resources, population: u32) -> Result<UnitId, ()> {
    let pos = search_new_unit_pos(world, rng)?;

    let name = resources.cultures.get(&resources.cultures.random()).city_name_model.generate(rng, 3, 10);
    let name = Strings::capitalize(&name);
    let mut unit = Unit {
        xy: pos,
        creatures: Vec::new(),
        cemetery: Vec::new(),
        resources: UnitResources {
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
        unit_type: UnitType::Village
    };

    while unit.creatures.len() < population as usize {
        
        let mut factory = CreatureFactory::new(rng.clone());
        let date = world.date.clone();
        let family = factory.make_family_or_single(&date, resources.species.id_of("species:human"), world, &resources);
        for creature_id in family {
            unit.creatures.push(creature_id);
        }
        rng.next();

    }

    return Ok(world.units.add::<UnitId>(unit));
}

fn search_new_unit_pos(world: &World, rng: &mut Rng) -> Result<Coord2, ()> {
    for _ in 0..100 {
        let x = rng.randu_range(0, world.map.size.x());
        let y = rng.randu_range(0, world.map.size.y());
        let tile = world.map.tile(x, y);
        match tile.region_id {
            // Ocean
            0 => continue,
            // Desert
            4 => continue,
            _ => ()
        }
        let candidate = Coord2::xy(x as i32, y as i32);
        let too_close = world.units.iter().any(|unit| {
            let unit = unit.borrow();
            if unit.creatures.len() == 0 {
                if unit.xy == candidate {
                    return true;
                }
                return false;
            }
            return unit.xy.dist_squared(&candidate) < 3. * 3.
        });
        if too_close {
            continue;
        }
        return Ok(candidate)
    }
    return Err(());
}
