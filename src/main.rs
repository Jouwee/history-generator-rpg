use std::{cmp, collections::HashMap, io};
use commons::rng::Rng;


pub mod commons;

struct CulturePrefab {
    name: String,
    language: LanguagePrefab,
    names: Vec<String>
}

struct LanguagePrefab {
    dictionary: HashMap<String, String>
}


struct RegionPrefab {
    name: String,
    fauna: Vec<String>,
    flora: Vec<String>,
}

#[derive(Clone)]
struct VillageEncyclopedia<'a> {
    name: String,
    founding_year: u32,
    culture: &'a CulturePrefab,
    region: &'a RegionPrefab,
    allies: Vec<&'a VillageEncyclopedia<'a>>,
    enemies: Vec<&'a VillageEncyclopedia<'a>>,
}

struct WorldGraph<'a> {
    nodes: Vec<WorldGraphNode<'a>>,
    people: Vec<Person<'a>>,
    events: Vec<Event<'a>>
}

enum WorldGraphNode<'a> {
    Village(VillageEncyclopedia<'a>)
}

#[derive(Clone)]
struct Person<'a> {
    name: String,
    birth: u32,
    death: u32,
    culture: &'a CulturePrefab,
    spouse: Option<Box<Person<'a>>>,
    leader: bool
}

enum Event<'a> {
    PersonBorn(u32, Person<'a>),
    PersonDeath(u32, Person<'a>),
    Marriage(u32, Person<'a>, Person<'a>),
    VillageFounded(u32, VillageEncyclopedia<'a>, Person<'a>)
}

fn main() {
    let seed: u32 = 123456;
    let nords = CulturePrefab {
        name: String::from("Nords"),
        language: LanguagePrefab {
            dictionary: HashMap::from([
                (String::from("birch"), String::from("borch")),
                (String::from("pine"), String::from("pin")),
                (String::from("elk"), String::from("skog")),
                (String::from("boar"), String::from("vevel")),
                (String::from("fortress"), String::from("stad")),
                (String::from("sea"), String::from("so")),
                (String::from("port"), String::from("por")),
                (String::from("fish"), String::from("fisk")),
                (String::from("whale"), String::from("vale")),
                (String::from("kelp"), String::from("kjel")),
                (String::from("coral"), String::from("krall")),
            ])
        },
        names: vec!(
            String::from("Brefdemar Bog-Dawn"),
            String::from("Kjarkmar Maiden-Pommel"),
            String::from("Norratr Bog-Crusher"),
            String::from("Berami Hammer-Shield"),
            String::from("Holmis Blackthorn"),
            String::from("Batorolf Whitemane"),
            String::from("Yngokmar the Feeble"),
            String::from("Kverlam Hahranssen"),
            String::from("Belehl Hararikssen"),
            String::from("Gisljof Fanrarikesen")
        )
    };

    let khajit = CulturePrefab {
        name: String::from("Khajiit"),
        language: LanguagePrefab {
            dictionary: HashMap::from([
                (String::from("birch"), String::from("has")),
                (String::from("pine"), String::from("apa'")),
                (String::from("elk"), String::from("liz")),
                (String::from("boar"), String::from("skish")),
                (String::from("sea"), String::from("shas")),
                (String::from("fish"), String::from("rah")),
                (String::from("whale"), String::from("shin")),
                (String::from("kelp"), String::from("klash")),
                (String::from("coral"), String::from("fal")),
                (String::from("fortress"), String::from("'kanash")),
                (String::from("port"), String::from("'kapor")),
            ])
        },
        names: vec!(
            String::from("Arababa"),
            String::from("Qa'asi"),
            String::from("J'rji"),
            String::from("Nisaravi"),
            String::from("Shavrasha"),
            String::from("Kisi Rojstahe"),
            String::from("Zahleena Xatannil"),
            String::from("Ahjiuki Ahjbes"),
            String::from("Yusadhi Rahkhan"),
            String::from("Shivrri Karabi")
        )
    };

    let coastal = RegionPrefab {
        name: String::from("Coastal"),
        fauna: Vec::from([
            String::from("whale"),
            String::from("fish")
        ]),
        flora: Vec::from([
            String::from("kelp"),
            String::from("coral")
        ])
    };

    let forest = RegionPrefab {
        name: String::from("Forest"),
        fauna: Vec::from([
            String::from("elk"),
            String::from("boar")
        ]),
        flora: Vec::from([
            String::from("pine"),
            String::from("birch")
        ])
    };


    use std::time::Instant;
    let now = Instant::now();

    let world = generate_world(seed, 200, vec!(&nords, &khajit), vec!(&coastal, &forest));

    // let village = generate_village_encyclopedia(seed, &culture, &region);

    let elapsed = now.elapsed();

    //display_village(village);
    println!("");
    println!("World generated in {:.2?}", elapsed);

    loop {

                

        println!("");
        println!("Type something to filter");

        let mut filter = String::new();
        let _ = io::stdin().read_line(&mut filter);

        filter = filter.trim().to_string();

        // for node in world.nodes {
        //     match node {
        //         WorldGraphNode::Village(village) => display_village(&village)
        //     }
        // }
        println!();

        let mut anals: Vec<String> = Vec::new();

        for event in world.events.iter() {
            match event {
                Event::PersonBorn(year, person) => anals.push(format!("In {}, {} was born", year, person.name)),
                Event::PersonDeath(year, person) => anals.push(format!("In {}, {} died", year, person.name)),
                Event::VillageFounded(year, village, person) => anals.push(format!("In {}, {} found the city of {}", year, person.name, village.name)),
                Event::Marriage(year, person_a, person_b) => anals.push(format!("In {}, {} and {} married", year, person_a.name, person_b.name)),
            }
            
        }

        for gospel in anals {
            if filter.len() == 0 || gospel.contains(&filter) {
                println!("{}", gospel);
            }
        }

    }
}

#[derive(Debug)]
struct VillageSimulationData {
    farmers: f32,
    extractors: f32,
    industry: f32,
    army: f32,
    food: f32,
    money: f32,
}

fn generate_world<'a>(seed: u32, world_age: u32, cultures: Vec<&'a CulturePrefab>, regions: Vec<&'a RegionPrefab>) -> WorldGraph<'a> {
    let mut year: u32 = 1;
    let mut nodes: Vec<WorldGraphNode> = Vec::new();
    let mut people: Vec<Person> = Vec::new();

    let mut events: Vec<Event> = Vec::new();
    let mut node_simulation: Vec<VillageSimulationData> = Vec::new();

    let mut rng = Rng::new(seed);

    // Generate starter people
    for i in 0..10 {
        let culture = cultures[rng.randu_range(0, cultures.len())];
        let person = generate_person(seed + i, year, culture);
        people.push(person.clone());
        events.push(Event::PersonBorn(year, person.clone()));
    }


    loop {
        year = year + 1;
        if year > world_age {
            break;
        }

        let mut new_people: Vec<Person> = Vec::new();

        for person in people.iter_mut() {
            // TODO: More performant approach
            if person.death > 0 {
                continue
            }

            let age = (year - person.birth) as f32;
            if rng.rand_chance(f32::min(1.0, (age/120.0).powf(5.0))) {
                person.death = year;
                events.push(Event::PersonDeath(year, person.clone()));
                continue
            }

            if age > 18.0 && person.spouse.is_none() && rng.rand_chance(0.1) {
                let spouse_age = rng.randu_range(18, age as usize + 10) as u32;
                let spouse_birth_year = year - u32::min(spouse_age, year);
                let mut spouse = generate_person(seed, spouse_birth_year, person.culture);
                spouse.spouse = Some(Box::new(person.clone()));
                person.spouse = Some(Box::new(spouse.clone()));
                new_people.push(spouse.clone());
                events.push(Event::Marriage(year, person.clone(), spouse.clone()));
            }

            if age > 18.0 && !person.leader && rng.rand_chance(1.0/50.0) {

                let region = regions[rng.randu_range(0, regions.len())];

                let village = generate_village_encyclopedia(seed, year, person.culture, region);
                
                // TODO: Ideally this would be a reference, but I can't figure out how to do it
                events.push(Event::VillageFounded(year, village.clone(), person.clone()));
                nodes.push(WorldGraphNode::Village(village));

                person.leader = true;
                
                node_simulation.push(VillageSimulationData {
                    farmers: 30.0,
                    extractors: 10.0,
                    industry: 5.0,
                    army: 5.0,
                    food: 50.0,
                    money: 0.0
                });

            }

        }

        for new_person in new_people {
            people.push(new_person)
        }

        

        for i in 0..nodes.len() {
            let node = &nodes[i];
            match node {
                // TODO: This will break if node is not village - Vectors mismatch
                WorldGraphNode::Village(village) => simulate_village_year(seed, &village, &mut node_simulation[i])
            }
            
        }

    }

    // println!("{:?}", node_simulation);

    return WorldGraph {
        nodes: Vec::from(nodes),
        people: Vec::from(people),
        events: Vec::from(events)
    }
}

fn simulate_village_year(seed: u32, village: &VillageEncyclopedia, simulation: &mut VillageSimulationData) {

    let total_population = simulation.farmers + simulation.extractors + simulation.industry + simulation.army;

    // TODO: random from region
    simulation.food += simulation.farmers * 2.0;
    simulation.food -= (simulation.farmers * 1.0) + (simulation.extractors * 1.0) + (simulation.industry * 1.1) + (simulation.army * 1.2);

    simulation.money += simulation.industry;
    simulation.money -= simulation.army * 0.2;

    // Buy food if necessary
    if simulation.food < 0.0 {
        let can_buy = simulation.money * 10.0;
        let need_to_buy = -simulation.food;
        if can_buy < need_to_buy {
            simulation.food += can_buy;
            simulation.money = 0.0;
        } else {
            simulation.food += need_to_buy;
            simulation.money -= need_to_buy / 10.0;
        }
    }

    if simulation.food > 0.0 {
        // TODO: IDK, some logic on culture precepts? Or whats missing?
        let growth = 1.05;
        simulation.farmers = simulation.farmers * growth;
        simulation.extractors = simulation.extractors * growth;
        simulation.industry = simulation.industry * growth;
        simulation.army = simulation.army * growth;

        // simulation.population = simulation.population * 1.1;
    } else {

        let starvation = simulation.food / total_population;

        simulation.farmers = simulation.farmers * starvation;
        simulation.extractors = simulation.extractors * starvation;
        simulation.industry = simulation.industry * starvation;
        simulation.army = simulation.army * starvation;

        simulation.food = 0.0;
    }

    let death_rate = 0.98;
    simulation.farmers = simulation.farmers * death_rate;
    simulation.extractors = simulation.extractors * death_rate;
    simulation.industry = simulation.industry * death_rate;
    simulation.army = simulation.army * death_rate;


}

fn generate_person<'a>(seed: u32, birth_year: u32, culture: &'a CulturePrefab) -> Person<'a> {
    let mut rng = Rng::new(seed);
    return Person {
        // TODO: Markov chains: https://www.samcodes.co.uk/project/markov-namegen/
        name: culture.names[rng.randu_range(0, culture.name.len())].clone(),
        birth: birth_year,
        culture,
        death: 0,
        spouse: None,
        leader: false
    }
}

fn generate_village_encyclopedia<'a>(seed: u32, founding_year: u32, culture: &'a CulturePrefab, region: &'a RegionPrefab) -> VillageEncyclopedia<'a> {
    let seed = seed + founding_year as u32;
    return VillageEncyclopedia {
        name: String::from(generate_location_name(seed, culture, region)),
        founding_year: founding_year,
        culture: culture,
        region: region,
        allies: vec!(),
        enemies: vec!()
    };
}

fn generate_location_name(seed: u32, culture: &CulturePrefab, region: &RegionPrefab) -> String {
    let mut rng = Rng::new(seed);

    let mut landmarks = Vec::new();
    landmarks.extend(&region.fauna);
    landmarks.extend(&region.flora);
    if let Some(landmark) = landmarks.get(rng.randu_range(0, landmarks.len())) {

        // TODO: Based on location
        let place_types = [String::from("fortress"), String::from("port")];
        if let Some(place_type) = place_types.get(rng.randu_range(0, place_types.len())) {

            // TODO: Fallback to something
            let landmark_tr = culture.language.dictionary.get(*landmark).unwrap_or(landmark);
            let placetype_tr = culture.language.dictionary.get(&*place_type).unwrap_or(place_type);
            return landmark_tr.to_owned() + placetype_tr;

        }
    }
    // TODO: Fallback to something
    return String::from("Village")
}

fn display_village(village: &VillageEncyclopedia) {
    println!("");
    println!("Village of {}", village.name);
    println!("--------------");

    println!("The {} village of {} was funded in the year {} by the {}.", village.region.name, village.name, village.founding_year, village.culture.name);
}