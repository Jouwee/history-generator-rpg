use std::{cell::{Ref, RefCell, RefMut}, cmp::Ordering, collections::{BTreeMap, HashMap, HashSet}, io, vec};
use colored::Colorize;
use commons::{markovchains::MarkovChainSingleWordModel, rng::Rng, strings::Strings};
use engine::{Id, Point2D};
use noise::{NoiseFn, Perlin};
use world::settlement::{Settlement, SettlementBuilder};

pub mod engine;
pub mod commons;
pub mod world;

#[derive(Clone)]
struct CulturePrefab {
    id: Id,
    name: String,
    language: LanguagePrefab,
    first_name_male_model: MarkovChainSingleWordModel,
    first_name_female_model: MarkovChainSingleWordModel,
    last_name_model: MarkovChainSingleWordModel,
}

#[derive(Clone)]
struct LanguagePrefab {
    dictionary: HashMap<String, String>
}


#[derive(Debug)]
struct RegionPrefab {
    name: String,
    id: usize,
    elevation: (u8, u8),
    temperature: (u8, u8),
    fauna: Vec<String>,
    flora: Vec<String>,
}

struct WorldGraph {
    cultures: HashMap<Id, CulturePrefab>,
    settlements: HashMap<Id, Settlement>,
    people: People,
    events: Vec<Event>
}

struct People {
    historic: HashMap<Id, RefCell<Person>>,
    alive: BTreeMap<Id, RefCell<Person>>
}

impl People {
    
    fn new() -> People {
        People {
            historic: HashMap::new(),
            alive: BTreeMap::new()
        }
    }

    fn get(&self, id: &Id) -> Option<Ref<Person>> {
        let option = self.alive.get(id).or(self.historic.get(id));
        match option {
            None => None,
            Some(ref_cell) => Some(ref_cell.borrow())
        }
    }

    fn get_mut(&self, id: &Id) -> Option<RefMut<Person>> {
        let option = self.alive.get(id).or(self.historic.get(id));
        match option {
            None => None,
            Some(ref_cell) => Some(ref_cell.borrow_mut())
        }
    }

    fn len(&self) -> usize {
        return self.alive.len() + self.historic.len();
    }

    fn insert(&mut self, person: Person) {
        if person.simulatable() {
            self.alive.insert(person.id, RefCell::new(person));
        } else {
            self.historic.insert(person.id, RefCell::new(person));
        }
    }

    fn iter(&self) -> impl Iterator<Item = (&Id, &RefCell<Person>)> {
        return self.alive.iter()
    }

    fn reindex(&mut self) {
        let mut historic = HashMap::new();
        let mut alive = BTreeMap::new();
        for person in self.alive.values() {
            if person.borrow().simulatable() {
                alive.insert(person.borrow().id, person.clone());
            } else {
                historic.insert(person.borrow().id, person.clone());
            }
        }
        for person in self.historic.values() {
            if person.borrow().simulatable() {
                alive.insert(person.borrow().id, person.clone());
            } else {
                historic.insert(person.borrow().id, person.clone());
            }
        }
        self.alive = alive;
        self.historic = historic;
    }

}

const WORLD_MAP_HEIGHT: usize = 64;
const WORLD_MAP_WIDTH: usize = 64;

struct WorldMap {
    elevation: [u8; WORLD_MAP_HEIGHT * WORLD_MAP_WIDTH],
    temperature: [u8; WORLD_MAP_HEIGHT * WORLD_MAP_WIDTH],
    region_id: [u8; WORLD_MAP_HEIGHT * WORLD_MAP_WIDTH]
}

impl WorldMap {

    pub fn get_world_tile(&self, x: usize, y: usize) -> WorldTileData {
        let i = (y * WORLD_MAP_WIDTH) + x;
        return WorldTileData {
            xy: Point2D(x, y),
            elevation: self.elevation[i],
            temperature: self.temperature[i],
            region_id: self.region_id[i],
        }
    }

}

struct WorldTileData {
    xy: Point2D,
    elevation: u8,
    temperature: u8,
    region_id: u8
}

#[derive(Clone, PartialEq, Debug)]
enum PersonSex {
    Male,
    Female
}

impl PersonSex {

    fn opposite(&self) -> PersonSex {
        match self {
            PersonSex::Male => return PersonSex::Female,
            PersonSex::Female => return PersonSex::Male,
        }
    }

}

#[derive(Clone, Debug)]
struct Person {
    id: Id,
    first_name: String,
    last_name: String,
    birth_last_name: String,
    importance: Importance,
    birth: u32,
    sex: PersonSex,
    death: u32,
    culture_id: Id,
    next_of_kin: Vec<NextOfKin>,
    leader: bool
}

#[derive(Clone, PartialEq, Debug)]
enum Importance {
    Important,
    Unimportant,
    Unknown
}

impl Importance {
    fn lower(&self) -> Importance {
        match self {
            Importance::Important => return Importance::Unimportant,
            Importance::Unimportant => return Importance::Unknown,
            Importance::Unknown => return Importance::Unknown,
        }
    }
}

impl Person {

    fn birth_name(&self) -> String {
        return format!("{} {}", self.first_name, self.birth_last_name)
    }

    fn name(&self) -> String {
        let mut title = "Commoner";
        if self.leader {
            title = "Leader";
        }
        return format!("{} {} ({:?}, {})", self.first_name, self.last_name, self.importance, title)
    }

    fn simulatable(&self) -> bool {
        self.alive() && self.importance != Importance::Unknown
    }

    fn alive(&self) -> bool {
        return self.death == 0
    }

    fn spouse(&self) -> Option<&Id> {
        let spouse = self.next_of_kin.iter().find(|r| r.relative == Relative::Spouse);
        if let Some(spouse) = spouse {
            return Some(&spouse.person_id)
        };
        return None
    }

    fn fertility(&self, year: u32) -> f32 {
        let age = (year - self.birth) as f32;
        // https://thefertilityshop.co.uk/wp-content/uploads/2021/12/bfs-monthly-fertility-by-age-1024x569.png
        if self.sex == PersonSex::Male {
            return f32::max(0.0, -(age / 60.0).powf(2.0) + 1.0)
        } else {
            return f32::max(0.0, -(age / 40.0).powf(6.0) + 1.0)
        }
    }

    fn find_next_of_kin(&self, relative: Relative) -> Option<&Id> {
        let spouse = self.next_of_kin.iter().find(|r| r.relative == relative);
        if let Some(spouse) = spouse {
            return Some(&spouse.person_id)
        };
        return None
    }

    fn remove_next_of_kin(&mut self, person_id: Id) {
        let i = self.next_of_kin.iter().position(|r: &NextOfKin| r.person_id == person_id);
        if let Some(i) = i {
            self.next_of_kin.remove(i);
        }
    }

    fn sorted_heirs(&self) -> Vec<NextOfKin> {
        let priorities = [
            Relative::Child,
            Relative::Spouse,
            Relative::Sibling,
        ];
        
        let mut sorted = self.next_of_kin.clone();
        sorted.sort_by(|kin1, kin2| {
            let priority_1 = priorities.iter().position(|r| *r == kin1.relative);
            let priority_2 = priorities.iter().position(|r| *r == kin2.relative);
            if priority_1 != priority_2 {
                return priority_1.cmp(&priority_2);
            }
            return Ordering::Equal;
        });
        return sorted
    }

}

#[derive(Clone, Debug)]
struct NextOfKin {
    person_id: Id,
    relative: Relative
}

#[derive(Clone, PartialEq, Debug)]
enum Relative {
    Spouse,
    Sibling,
    Parent,
    Child,
}

enum Event {
    PersonBorn(u32, Id),
    PersonDeath(u32, Id),
    Marriage(u32, Id, Id),
    Inheritance(u32, Id, Id),
    RoseToPower(u32, Id),
    SettlementFounded(u32, Id, Id)
}

fn main() {

    use std::time::Instant;
    let now = Instant::now();

    let nords = CulturePrefab {
        id: Id(0),
        name: String::from("Nords"),
        language: LanguagePrefab {
            dictionary: HashMap::from([
                (String::from("birch"), String::from("borch")),
                (String::from("pine"), String::from("pin")),
                (String::from("elk"), String::from("skog")),
                (String::from("boar"), String::from("vevel")),
                (String::from("fortress"), String::from("stad")),
                (String::from("sea"), String::from("so")),
                (String::from("port"), String::from("pør")),
                (String::from("fish"), String::from("fisk")),
                (String::from("whale"), String::from("vale")),
                (String::from("kelp"), String::from("kjel")),
                (String::from("coral"), String::from("krall")),
                (String::from("scorpion"), String::from("skør")),
                (String::from("vulture"), String::from("vøl")),
                (String::from("cactus"), String::from("kak")),
                (String::from("palm"), String::from("pølm")),
            ])
        },
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
    };

    let khajit = CulturePrefab {
        id: Id(1),
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
                (String::from("scorpion"), String::from("sacrah")),
                (String::from("vulture"), String::from("va'al")),
                (String::from("cactus"), String::from("kazh")),
                (String::from("palm"), String::from("pahz")),
            ])
        },
        first_name_male_model: MarkovChainSingleWordModel::train(vec!(
            "Ab'ar", "Ab'bar", "Ab'bil", "Ab'der", "Ab'dul", "Ab'gh", "Ab'ir", "Ab'kir", "Ab'med", "Ab'nir", "Ab'noud", "Ab'sien", "Ab'soud", "Ab'taba", "Ab'tabe", "Ab'urabi", "Ak'ar", "Ak'bar", "Ak'bil", "Ak'der", "Ak'dul", "Ak'gh", "Ak'ir", "Ak'kir", "Ak'med", "Ak'nir", "Ak'noud", "Ak'sien", "Ak'soud", "Ak'taba", "Ak'tabe", "Ak'urabi", "Akh'ar", "Akh'bar", "Akh'bil", "Akh'der", "Akh'dul", "Akh'gh", "Akh'ir", "Akh'kir", "Akh'med", "Akh'nir", "Akh'noud", "Akh'sien", "Akh'soud", "Akh'taba", "Akh'tabe", "Akh'urabi", "Amar", "Ambar", "Ambil", "Amder", "Amdul", "Amgh", "Amir", "Amkir", "Ammed", "Amnir", "Amnoud", "Amsien", "Amsoud", "Amtaba", "Amtabe", "Amurabi", "Fa'ar", "Fa'bar", "Fa'bil", "Fa'der", "Fa'dul", "Fa'gh", "Fa'ir", "Fa'kir", "Fa'med", "Fa'nir", "Fa'noud", "Fa'sien", "Fa'soud", "Fa'taba", "Fa'tabe", "Fa'urabi", "Husar", "Husbar", "Husbil", "Husder", "Husdul", "Husgh", "Husir", "Huskir", "Husmed", "Husnir", "Husnoud", "Hussien", "Hussoud", "Hustaba", "Hustabe", "Husurabi", "Moar", "Mobar", "Mobil", "Moder", "Modul", "Mogh", "Moir", "Mokir", "Momed", "Monir", "Monoud", "Mosien", "Mosoud", "Motaba", "Motabe", "Mourabi", "Mohamar", "Mohambar", "Mohambil", "Mohamder", "Mohamdul", "Mohamgh", "Mohamir", "Mohamkir", "Mohammed", "Mohamnir", "Mohamnoud", "Mohamsien", "Mohamsoud", "Mohamtaba", "Mohamtabe", "Mohamurabi", "Mojar", "Mojbar", "Mojbil", "Mojder", "Mojdul", "Mojgh", "Mojir", "Mojkir", "Mojmed", "Mojnir", "Mojnoud", "Mojsien", "Mojsoud", "Mojtaba", "Mojtabe", "Mojurabi", "Naar", "Nabar", "Nabil", "Nader", "Nadul", "Nagh", "Nair", "Nakir", "Named", "Nanir", "Nanoud", "Nasien", "Nasoud", "Nataba", "Natabe", "Naurabi", "Omar", "Ombar", "Ombil", "Omder", "Omdul", "Omgh", "Omir", "Omkir", "Ommed", "Omnir", "Omnoud", "Omsien", "Omsoud", "Omtaba", "Omtabe", "Omurabi", "Shaar", "Shabar", "Shabil", "Shader", "Shadul", "Shagh", "Shair", "Shakir", "Shamed", "Shanir", "Shanoud", "Shasien", "Shasoud", "Shataba", "Shatabe", "Shaurabi", "Sinar", "Sinbar", "Sinbil", "Sinder", "Sindul", "Singh", "Sinir", "Sinkir", "Sinmed", "Sinnir", "Sinnoud", "Sinsien", "Sinsoud", "Sintaba", "Sintabe", "Sinurabi", "Za'ar", "Za'bar", "Za'bil", "Za'der", "Za'dul", "Za'gh", "Za'ir", "Za'kir", "Za'med", "Za'nir", "Za'noud", "Za'sien", "Za'soud", "Za'taba", "Za'tabe", "Za'urabi", "Zan'ar", "Zan'bar", "Zan'bil", "Zan'der", "Zan'dul", "Zan'gh", "Zan'ir", "Zan'kir", "Zan'med", "Zan'nir", "Zan'noud", "Zan'sien", "Zan'soud", "Zan'taba", "Zan'tabe", "Zan'urabi",
        ), 3),
        first_name_female_model: MarkovChainSingleWordModel::train(vec!(
            "Aahin", "Aahni", "Afeliz", "Ahana", "Aheh", "Ahrazad", "Ajjan", "Akhtar", "Anita", "Araya", "Ariba", "Ashima", "Asrin", "Atima", "Azita", "Aziahin", "Aziahni", "Azifeliz", "Azihana", "Aziheh", "Azihrazad", "Azijjan", "Azikhtar", "Azinita", "Aziraya", "Aziriba", "Azishima", "Azisrin", "Azitima", "Azizita", "Elaahin", "Elaahni", "Elafeliz", "Elahana", "Elaheh", "Elahrazad", "Elajjan", "Elakhtar", "Elanita", "Elaraya", "Elariba", "Elashima", "Elasrin", "Elatima", "Elazita", "Faahin", "Faahni", "Fafeliz", "Fahana", "Faheh", "Fahrazad", "Fajjan", "Fakhtar", "Fanita", "Faraya", "Fariba", "Fashima", "Fasrin", "Fatima", "Fazita", "Khaahin", "Khaahni", "Khafeliz", "Khahana", "Khaheh", "Khahrazad", "Khajjan", "Khakhtar", "Khanita", "Kharaya", "Khariba", "Khashima", "Khasrin", "Khatima", "Khazita", "Kiahin", "Kiahni", "Kifeliz", "Kihana", "Kiheh", "Kihrazad", "Kijjan", "Kikhtar", "Kinita", "Kiraya", "Kiriba", "Kishima", "Kisrin", "Kitima", "Kizita", "Moahin", "Moahni", "Mofeliz", "Mohana", "Moheh", "Mohrazad", "Mojjan", "Mokhtar", "Monita", "Moraya", "Moriba", "Moshima", "Mosrin", "Motima", "Mozita", "Naahin", "Naahni", "Nafeliz", "Nahana", "Naheh", "Nahrazad", "Najjan", "Nakhtar", "Nanita", "Naraya", "Nariba", "Nashima", "Nasrin", "Natima", "Nazita", "Raahin", "Raahni", "Rafeliz", "Rahana", "Raheh", "Rahrazad", "Rajjan", "Rakhtar", "Ranita", "Raraya", "Rariba", "Rashima", "Rasrin", "Ratima", "Razita", "Riahin", "Riahni", "Rifeliz", "Rihana", "Riheh", "Rihrazad", "Rijjan", "Rikhtar", "Rinita", "Riraya", "Ririba", "Rishima", "Risrin", "Ritima", "Rizita", "Saahin", "Saahni", "Safeliz", "Sahana", "Saheh", "Sahrazad", "Sajjan", "Sakhtar", "Sanita", "Saraya", "Sariba", "Sashima", "Sasrin", "Satima", "Sazita", "Shaahin", "Shaahni", "Shafeliz", "Shahana", "Shaheh", "Shahrazad", "Shajjan", "Shakhtar", "Shanita", "Sharaya", "Shariba", "Shashima", "Shasrin", "Shatima", "Shazita", "Soahin", "Soahni", "Sofeliz", "Sohana", "Soheh", "Sohrazad", "Sojjan", "Sokhtar", "Sonita", "Soraya", "Soriba", "Soshima", "Sosrin", "Sotima", "Sozita", "Taahin", "Taahni", "Tafeliz", "Tahana", "Taheh", "Tahrazad", "Tajjan", "Takhtar", "Tanita", "Taraya", "Tariba", "Tashima", "Tasrin", "Tatima", "Tazita", "Zaahin", "Zaahni", "Zafeliz", "Zahana", "Zaheh", "Zahrazad", "Zajjan", "Zakhtar", "Zanita", "Zaraya", "Zariba", "Zashima", "Zasrin", "Zatima", "Zazita", 
        ), 3),
        last_name_model: MarkovChainSingleWordModel::train(vec!(
            "Abiri", "Abus", "Adavi", "Ahan", "Ahir", "Akar", "Amanni", "Amnin", "Anai", "Aoni", "Arabi", "Aspoor", "Astae", "Atani", "Avandi", "Barabiri", "Barabus", "Baradavi", "Barahan", "Barahir", "Barakar", "Baramanni", "Baramnin", "Baranai", "Baraoni", "Bararabi", "Baraspoor", "Barastae", "Baratani", "Baravandi", "Hammubiri", "Hammubus", "Hammudavi", "Hammuhan", "Hammuhir", "Hammukar", "Hammumanni", "Hammumnin", "Hammunai", "Hammuoni", "Hammurabi", "Hammuspoor", "Hammustae", "Hammutani", "Hammuvandi", "Jabiri", "Jabus", "Jadavi", "Jahan", "Jahir", "Jakar", "Jamanni", "Jamnin", "Janai", "Jaoni", "Jarabi", "Jaspoor", "Jastae", "Jatani", "Javandi", "Khabiri", "Khabus", "Khadavi", "Khahan", "Khahir", "Khakar", "Khamanni", "Khamnin", "Khanai", "Khaoni", "Kharabi", "Khaspoor", "Khastae", "Khatani", "Khavandi", "Kibiri", "Kibus", "Kidavi", "Kihan", "Kihir", "Kikar", "Kimanni", "Kimnin", "Kinai", "Kioni", "Kirabi", "Kispoor", "Kistae", "Kitani", "Kivandi", "Mahbiri", "Mahbus", "Mahdavi", "Mahhan", "Mahhir", "Mahkar", "Mahmanni", "Mahmnin", "Mahnai", "Mahoni", "Mahrabi", "Mahspoor", "Mahstae", "Mahtani", "Mahvandi", "Raibiri", "Raibus", "Raidavi", "Raihan", "Raihir", "Raikar", "Raimanni", "Raimnin", "Rainai", "Raioni", "Rairabi", "Raispoor", "Raistae", "Raitani", "Raivandi", "Robiri", "Robus", "Rodavi", "Rohan", "Rohir", "Rokar", "Romanni", "Romnin", "Ronai", "Rooni", "Rorabi", "Rospoor", "Rostae", "Rotani", "Rovandi", "Sabiri", "Sabus", "Sadavi", "Sahan", "Sahir", "Sakar", "Samanni", "Samnin", "Sanai", "Saoni", "Sarabi", "Saspoor", "Sastae", "Satani", "Savandi", "Sibiri", "Sibus", "Sidavi", "Sihan", "Sihir", "Sikar", "Simanni", "Simnin", "Sinai", "Sioni", "Sirabi", "Sispoor", "Sistae", "Sitani", "Sivandi", "Solbiri", "Solbus", "Soldavi", "Solhan", "Solhir", "Solkar", "Solmanni", "Solmnin", "Solnai", "Soloni", "Solrabi", "Solspoor", "Solstae", "Soltani", "Solvandi", "Tavakbiri", "Tavakbus", "Tavakdavi", "Tavakhan", "Tavakhir", "Tavakkar", "Tavakmanni", "Tavakmnin", "Tavaknai", "Tavakoni", "Tavakrabi", "Tavakspoor", "Tavakstae", "Tavaktani", "Tavakvandi", "Zabiri", "Zabus", "Zadavi", "Zahan", "Zahir", "Zakar", "Zamanni", "Zamnin", "Zanai", "Zaoni", "Zarabi", "Zaspoor", "Zastae", "Zatani", "Zavandi", 
        ), 3)
    };

    let regions = vec!(
        RegionPrefab {
            id: 0,
            name: String::from("Coastal"),
            elevation: (0, 0),
            temperature: (0, 5),
            fauna: Vec::from([
                String::from("whale"),
                String::from("fish")
            ]),
            flora: Vec::from([
                String::from("kelp"),
                String::from("coral")
            ])
        },
        RegionPrefab {
            id: 1,
            name: String::from("Forest"),
            elevation: (1, 5),
            temperature: (0, 3),
            fauna: Vec::from([
                String::from("elk"),
                String::from("boar")
            ]),
            flora: Vec::from([
                String::from("pine"),
                String::from("birch")
            ])
        },
        RegionPrefab {
            id: 2,
            name: String::from("Desert"),
            elevation: (1, 5),
            temperature: (4, 5),
            fauna: Vec::from([
                String::from("scorpion"),
                String::from("vulture")
            ]),
            flora: Vec::from([
                String::from("cactus"),
                String::from("palm")
            ])
        },
    );

    let elapsed = now.elapsed();

    println!("");
    println!("Models created in {:.2?}", elapsed);

    let now = Instant::now();

    let world = generate_world(Rng::seeded("prototype"), 500, vec!(nords, khajit), &regions);

    let elapsed = now.elapsed();

    println!("");
    println!("World generated in {:.2?}", elapsed);
    println!(" {} people", world.people.len());
    println!(" {} settlements", world.settlements.len());
    println!(" {} events", world.events.len());

    loop {

                

        println!("");
        println!("Type something to filter");

        let mut filter = String::new();
        let _ = io::stdin().read_line(&mut filter);

        filter = filter.trim().to_string();

        println!();

        let mut anals: Vec<String> = Vec::new();

        for event in world.events.iter() {
            match event {
                Event::PersonBorn(year, person) => {
                    let person = world.people.get(person).unwrap();
                    if let Some(person_id) = person.find_next_of_kin(Relative::Parent) {
                        let parent = world.people.get(person_id).unwrap();
                        anals.push(format!("In {}, {} fathered {}", year, parent.name(), person.name()))
                    } else {
                        anals.push(format!("In {}, {} was born", year, person.name()))
                    }
                },
                Event::PersonDeath(year, person) => anals.push(format!("In {}, {} died", year, world.people.get(person).unwrap().name())),
                Event::SettlementFounded(year, settlement, person) => {
                    let settlement = world.settlements.get(settlement).unwrap();
                    anals.push(format!("In {}, {} found the city of {}", year, world.people.get(person).unwrap().name(), settlement.name))
                },
                Event::Marriage(year, person_a, person_b) => {
                    anals.push(format!("In {}, {} and {} married", year, world.people.get(person_a).unwrap().name(), world.people.get(person_b).unwrap().birth_name()))
                },
                Event::Inheritance(year, person_a, person_b) => anals.push(format!("In {}, {} inherited everything from {}", year, world.people.get(person_b).unwrap().name(), world.people.get(person_a).unwrap().name())),
                Event::RoseToPower(year, person) => anals.push(format!("In {}, {} rose to power", year, world.people.get(person).unwrap().name())),
            }
            
        }

        for gospel in anals {
            if filter.len() == 0 || gospel.contains(&filter) {
                println!("{}", gospel);
            }
        }

    }
}

fn generate_world(mut rng: Rng, world_age: u32, cultures: Vec<CulturePrefab>, regions: &Vec<RegionPrefab>) -> WorldGraph {
    let mut year: u32 = 1;
    let mut world_graph = WorldGraph {
        cultures: HashMap::new(),
        settlements: HashMap::new(),
        people: People::new(),
        events: vec!()
    };

    let mut culture_id = Id(0);
    for culture in cultures.iter() {
        let mut culture = culture.clone();
        culture.id = culture_id.next();
        world_graph.cultures.insert(culture.id, culture);
    }

    let world_map = generate_world_map(&rng, regions);

    let mut person_id = Id(0);
    let mut sett_id = Id(0);

    // Generate starter people
    for _ in 0..10 {
        rng.next();
        let id = person_id.next();
        let culture = world_graph.cultures.get(&Id(rng.randu_range(0, culture_id.0 as usize) as i32)).unwrap();
        let sex;
        if rng.rand_chance(0.5) {
            sex = PersonSex::Male;
        } else {
            sex = PersonSex::Female;
        }
        let person = generate_person(&rng, Importance::Important, id, year, sex, &culture, None);
        world_graph.events.push(Event::PersonBorn(year, person.id));
        world_graph.people.insert(person);
    }

    loop {
        year = year + 1;
        if year > world_age {
            break;
        }

        println!("Year {}, {} people to process", year, world_graph.people.alive.len());
        if year % 10 == 0 {
            print_world_map(&world_graph, &world_map);

            // println!("Year {}, {} people to process", year, world_graph.people.alive.len());
            // println!("Press anything to continue");
            // let mut filter = String::new();
            // let _ = io::stdin().read_line(&mut filter);
        }

        let mut new_people: Vec<Person> = Vec::new();

        for (_, person) in world_graph.people.iter() {
            let mut person = person.borrow_mut();
            let age = (year - person.birth) as f32;
            if rng.rand_chance(f32::min(1.0, (age/120.0).powf(5.0))) {
                person.death = year;
                world_graph.events.push(Event::PersonDeath(year, person.id));
                if person.leader {
                    let heirs_by_order = person.sorted_heirs();
                
                    let mut valid_heir = false;
                    for heir in heirs_by_order {
                        let mut heir = world_graph.people.get_mut(&heir.person_id).unwrap();
                        if heir.alive() {
                            // TODO: Leader of what?
                            heir.leader = true;
                            heir.importance = Importance::Important;
                            world_graph.events.push(Event::Inheritance(year, person.id, heir.id));
                            valid_heir = true;
                            break
                        }
                    }
                    if !valid_heir {
                        let culture = world_graph.cultures.get(&Id(rng.randu_range(0, culture_id.0 as usize) as i32)).unwrap();
                        let sex;
                        if rng.rand_chance(0.5) {
                            sex = PersonSex::Male;
                        } else {
                            sex = PersonSex::Female;
                        }
                        rng.next();
                        let mut new_leader = generate_person(&rng, Importance::Important, person_id.next(), year, sex, &culture, None);
                        new_leader.leader = true;
                        world_graph.events.push(Event::RoseToPower(year, new_leader.id));
                        new_people.push(new_leader);
                    }
                }

                continue
            }

            if age > 18.0 && person.spouse().is_none() && rng.rand_chance(0.1) {
                rng.next();
                let id = person_id.next();
                let spouse_age = rng.randu_range(18, age as usize + 10) as u32;
                let spouse_birth_year = year - u32::min(spouse_age, year);
                let culture = world_graph.cultures.get(&person.culture_id).unwrap();
                let mut spouse = generate_person(&rng, person.importance.lower(), id, spouse_birth_year, person.sex.opposite(), culture, None);
                spouse.last_name = person.last_name.clone();
                spouse.next_of_kin.push(NextOfKin {
                    person_id: person.id,
                    relative: Relative::Spouse
                });
                person.next_of_kin.push(NextOfKin {
                    person_id: spouse.id,
                    relative: Relative::Spouse
                });
                world_graph.events.push(Event::Marriage(year, person.id, spouse.id));
                new_people.push(spouse.clone());
                continue;
            }

            if age > 18.0 && person.spouse().is_some() {
                let spouse = world_graph.people.get_mut(person.spouse().unwrap()).unwrap();
                let couple_fertility = person.fertility(year) * spouse.fertility(year);

                if rng.rand_chance(couple_fertility * 0.5) {
                    let id = person_id.next();
                    rng.next();
                    let sex;
                    if rng.rand_chance(0.5) {
                        sex = PersonSex::Male;
                    } else {
                        sex = PersonSex::Female;
                    }
                    let culture = world_graph.cultures.get(&person.culture_id).unwrap();
                    let mut child = generate_person(&rng, person.importance.lower(), id, year, sex, culture, Some(&person.last_name));
                    child.next_of_kin.push(NextOfKin { 
                        person_id: person.id,
                        relative: Relative::Parent
                    });
                    world_graph.events.push(Event::PersonBorn(year, child.id));
                    person.next_of_kin.push(NextOfKin { 
                        person_id: child.id,
                        relative: Relative::Child
                    });
                    new_people.push(child);
                    continue;
                }
            }

            if age > 18.0 && !person.leader && rng.rand_chance(1.0/50.0) {
                rng.next();
                let culture = world_graph.cultures.get(&person.culture_id).unwrap();
                let settlement = generate_settlement(&rng, year, culture, &world_graph, &world_map, regions).clone();
                let id = sett_id.next();
                world_graph.events.push(Event::SettlementFounded(year, id, person.id));
                world_graph.settlements.insert(id, settlement);
                person.leader = true;
                continue;
            }

        }


        for new_person in new_people {
            world_graph.people.insert(new_person);
        }
        
        world_graph.people.reindex();

        for (_, settlement) in world_graph.settlements.iter_mut() {
            if settlement.demographics.population <= 0 {
                continue
            }

            // https://en.wikipedia.org/wiki/Estimates_of_historical_world_population
            let growth = rng.randf_range(-0.005, 0.03);
            let child_chance = (settlement.demographics.population as f32) * growth;
            if child_chance < 0.0 {
                if child_chance > -1.0 && rng.rand_chance(child_chance.abs()) {
                    settlement.demographics.change_population(-1);
                } else {
                    settlement.demographics.change_population(child_chance as i32);
                }
            } else {
                if child_chance < 1.0 && rng.rand_chance(child_chance) {
                    settlement.demographics.population = settlement.demographics.population + 1;
                } else {
                    settlement.demographics.change_population(child_chance as i32);
                }
            }

        }

    }

    return world_graph
}

fn generate_person(rng: &Rng, importance: Importance, next_id: Id, birth_year: u32, sex: PersonSex, culture: &CulturePrefab, surname: Option<&str>) -> Person {
    let mut rng = rng.derive("person");
    let first_name;
    match sex {
        PersonSex::Male => first_name = culture.first_name_male_model.generate(&rng.derive("first_name"), 4, 15),
        PersonSex::Female => first_name = culture.first_name_female_model.generate(&rng.derive("first_name"), 4, 15)
    }
    let first_name = Strings::capitalize(&first_name);
    let last_name;
    match surname {
        Some(str) => last_name = String::from(str),
        None => last_name = Strings::capitalize(&culture.last_name_model.generate(&rng.derive("last_name"), 4, 15))
    }
    return Person {
        id: next_id,
        importance,
        first_name,
        last_name: last_name.clone(),
        birth_last_name: last_name.clone(),
        birth: birth_year,
        sex,
        culture_id: culture.id,
        death: 0,
        next_of_kin: Vec::new(),
        leader: false
    }
}


fn generate_settlement(rng: &Rng, founding_year: u32, culture: &CulturePrefab, world_graph: &WorldGraph, world_map: &WorldMap, regions: &Vec<RegionPrefab>) -> Settlement {
    let mut rng = rng.derive("settlement");
    let mut xy = Point2D(0, 0);
    // TODO: What if there's no more places?
    'candidates: for _ in 1..10 {
        xy = Point2D(rng.randu_range(0, WORLD_MAP_WIDTH), rng.randu_range(0, WORLD_MAP_HEIGHT));
        for settlement in world_graph.settlements.values() {
            if settlement.xy.dist_squared(&xy) < 5.0_f32.powi(2) {
                continue 'candidates;
            }
        }
        break;
    }
    let region_id = world_map.get_world_tile(xy.0, xy.1).region_id as usize;
    let region = regions.get(region_id).unwrap();

    return SettlementBuilder::colony(&rng, xy, founding_year, culture, region).create()
}

fn generate_world_map(rng: &Rng, regions: &Vec<RegionPrefab>) -> WorldMap {
    let rng = rng.derive("world_map");
    let mut map = WorldMap {
        elevation: [0; WORLD_MAP_HEIGHT * WORLD_MAP_WIDTH],
        temperature: [0; WORLD_MAP_HEIGHT * WORLD_MAP_WIDTH],
        region_id: [0; WORLD_MAP_HEIGHT * WORLD_MAP_WIDTH],
    };
    let n_elev = Perlin::new(rng.derive("elevation").seed());
    let n_temp = Perlin::new(rng.derive("temperature").seed());
    let n_reg = Perlin::new(rng.derive("region").seed());
    for y in 0..WORLD_MAP_HEIGHT {
        for x in 0..WORLD_MAP_WIDTH {
            let i = (y * WORLD_MAP_WIDTH) + x;
            let xf = x as f64;
            let yf = y as f64;
            {
                let low = n_elev.get([xf / 10.0, yf / 10.0]);
                let med = n_elev.get([xf / 4.0, yf / 4.0]);
                map.elevation[i] = ((1.0+low+med) / 4.0 * 5.0) as u8;
            }
            {
                let low = n_temp.get([xf / 10.0, yf / 10.0]);
                // let med = n_temp.get([xf / 4.0, yf / 4.0]);
                map.temperature[i] = (low * 5.0) as u8;
            }
            {
                let mut region_candidates: Vec<u8> = Vec::new();
                for (j, region) in regions.iter().enumerate() {
                    if map.elevation[i] >= region.elevation.0 && map.elevation[i] <= region.elevation.1 && map.temperature[i] >= region.temperature.0 && map.temperature[i] <= region.temperature.1 {
                        region_candidates.push(j as u8);
                    }
                }
                match region_candidates.len() {
                    0 => panic!("No region candidate for elevation {} and temperature {}", map.elevation[i], map.temperature[i]),
                    1 => map.region_id[i] = region_candidates.pop().expect("Already checked"),
                    _ => {
                        let noise = n_reg.get([xf / 10.0, yf / 10.0]);
                        map.region_id[i] = region_candidates[(noise * region_candidates.len() as f64) as usize];
                    }
                }
            }
        }
    }
    return map;
}

fn print_world_map(world_graph: &WorldGraph, world_map: &WorldMap) {
    println!("--------------------------------------------------------------------------------------------------------------------------------");
    for y in 0..WORLD_MAP_HEIGHT {
        for x in 0..WORLD_MAP_WIDTH {
            let tile = world_map.get_world_tile(x, y);
            let mut string;
            match tile.elevation {
                0 => string = String::from(" "),
                1 => string = String::from(", "),
                2 => string = String::from("--"),
                3 => string = String::from("++"),
                4 => string = String::from("##"),
                _ => string = String::from("??")
            }

            for settlement in world_graph.settlements.values() {
                if settlement.xy.0 == x && settlement.xy.1 == y {
                    // 🏛️🛖🏘️🏰🕌⛪️🛕🕍⛺️🎪
                    if settlement.demographics.population < 50 {
                        string = String::from("🛖");    
                    } else if settlement.demographics.population < 150 {
                        string = String::from("🏘️");    
                    } else if settlement.demographics.population < 1000 {
                        string = String::from("🕍");
                    } else {
                        string = String::from("🕌");
                    }
                }
            }

            let colored_string;
            match tile.elevation {
                0 => colored_string = string.blue(),
                1 => colored_string = string.green(),
                2 => colored_string = string.yellow(),
                3 => colored_string = string.white(),
                4 => colored_string = string.white(),
                _ => colored_string = string.white()
            }
            print!("{}", colored_string);
        }
        println!("")
    }
}