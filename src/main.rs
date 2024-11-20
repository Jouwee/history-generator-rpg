use std::{collections::HashMap, io, vec};
use colored::Colorize;
use commons::{markovchains::MarkovChainSingleWordModel, rng::{self, Rng}, strings::Strings};
use noise::{NoiseFn, Perlin};

pub mod commons;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct PersonId(i32);

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct Point(usize, usize);

impl Point {

    pub fn dist_squared(&self, another: &Point) -> f32 {
        let x = another.0 as f32 - self.0 as f32;
        let y = another.1 as f32 - self.1 as f32;
        return x*x + y*y
    }
}

impl PersonId {
    pub fn next(&mut self) -> PersonId {
        let clone = self.clone();
        self.0 = self.0 + 1;
        clone
    }
}

struct CulturePrefab {
    name: String,
    language: LanguagePrefab,
    first_name_male_model: MarkovChainSingleWordModel,
    first_name_female_model: MarkovChainSingleWordModel,
    last_name_model: MarkovChainSingleWordModel,
}

struct LanguagePrefab {
    dictionary: HashMap<String, String>
}


#[derive(Debug)]
struct RegionPrefab {
    name: String,
    elevation: (u8, u8),
    temperature: (u8, u8),
    fauna: Vec<String>,
    flora: Vec<String>,
}

#[derive(Clone)]
struct Settlement<'a> {
    xy: Point,
    name: String,
    founding_year: u32,
    culture: &'a CulturePrefab,
    region_id: usize,
}

struct WorldGraph<'a> {
    nodes: Vec<WorldGraphNode<'a>>,
    people: HashMap<PersonId, Person<'a>>,
    events: Vec<Event<'a>>
}

enum WorldGraphNode<'a> {
    SettlementNode(Settlement<'a>)
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
            xy: Point(x, y),
            elevation: self.elevation[i],
            temperature: self.temperature[i],
            region_id: self.region_id[i],
        }
    }

}

struct WorldTileData {
    xy: Point,
    elevation: u8,
    temperature: u8,
    region_id: u8
}

#[derive(Clone)]
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

#[derive(Clone)]
struct Person<'a> {
    id: PersonId,
    name: String,
    birth: u32,
    sex: PersonSex,
    death: u32,
    culture: &'a CulturePrefab,
    spouse: Option<PersonId>,
    heirs: Vec<PersonId>,
    leader: bool
}

enum Event<'a> {
    PersonBorn(u32, PersonId),
    PersonDeath(u32, PersonId),
    Marriage(u32, PersonId, PersonId),
    Inheritance(u32, PersonId, PersonId),
    SettlementFounded(u32, Settlement<'a>, PersonId)
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


    use std::time::Instant;
    let now = Instant::now();

    let world = generate_world(seed, 200, vec!(&nords, &khajit), &regions);

    let elapsed = now.elapsed();

    println!("");
    println!("World generated in {:.2?}", elapsed);

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
                Event::PersonBorn(year, person) => anals.push(format!("In {}, {} was born", year, world.people.get(person).unwrap().name)),
                Event::PersonDeath(year, person) => anals.push(format!("In {}, {} died", year, world.people.get(person).unwrap().name)),
                Event::SettlementFounded(year, settlement, person) => anals.push(format!("In {}, {} found the city of {}", year, world.people.get(person).unwrap().name, settlement.name)),
                Event::Marriage(year, person_a, person_b) => anals.push(format!("In {}, {} and {} married", year, world.people.get(person_a).unwrap().name, world.people.get(person_b).unwrap().name)),
                Event::Inheritance(year, person_a, person_b) => anals.push(format!("In {}, {} inherited everything from {}", year, world.people.get(person_a).unwrap().name, world.people.get(person_b).unwrap().name)),
            }
            
        }

        for gospel in anals {
            // if filter.len() == 0 || gospel.contains(&filter) {
                println!("{}", gospel);
            // }
        }

    }
}

fn generate_world<'a>(seed: u32, world_age: u32, cultures: Vec<&'a CulturePrefab>, regions: &'a Vec<RegionPrefab>) -> WorldGraph<'a> {
    let mut year: u32 = 1;
    let mut world_graph = WorldGraph {
        nodes: vec!(),
        people: HashMap::new(),
        events: vec!()
    };
    let world_map = generate_world_map(seed, regions);

    let mut rng = Rng::new(seed);

    let mut id = PersonId(0);

    // Generate starter people
    for _ in 0..10 {
        let id = id.next();
        let seed = seed * id.0 as u32;
        let culture = cultures[rng.randu_range(0, cultures.len())];
        let mut rng = Rng::new(seed);
        let sex;
        if rng.rand_chance(0.5) {
            sex = PersonSex::Male;
        } else {
            sex = PersonSex::Female;
        }
        let person = generate_person(seed, id, year, sex, culture);
        world_graph.events.push(Event::PersonBorn(year, person.id));
        world_graph.people.insert(person.id, person);
    }

    loop {
        year = year + 1;
        if year > world_age {
            break;
        }
//        print_world_map(&world_graph, &world_map);

        let mut new_people: Vec<Person> = Vec::new();

        // TODO: Rethink this. Can't read and modify people at the same time. My current approach doesn't allow two modifications for the same person in the same year
        for (_, person) in world_graph.people.iter() {
            // TODO: More performant approach
            if person.death > 0 {
                continue
            }

            let age = (year - person.birth) as f32;
            if rng.rand_chance(f32::min(1.0, (age/120.0).powf(5.0))) {
                let mut person = person.clone();
                person.death = year;
                world_graph.events.push(Event::PersonDeath(year, person.id));
                if let Some(spouse) = person.spouse {
                    let mut spouse = world_graph.people.get(&spouse).unwrap().clone();
                    spouse.spouse = None;
                    new_people.push(spouse);    
                }

                if person.leader {
                    if let Some(heir_id) = person.heirs.first() {
                        let mut heir = world_graph.people.get(&heir_id).unwrap().clone();
                        // TODO: Leader of what?
                        heir.leader = true;
                        world_graph.events.push(Event::Inheritance(year, person.id, heir.id));
                        new_people.push(heir);    
                    }
                }

                new_people.push(person);

                continue
            }

            if age > 18.0 && person.spouse.is_none() && rng.rand_chance(0.1) {
                let id = id.next();
                let seed = seed * id.0 as u32;
                let spouse_age = rng.randu_range(18, age as usize + 10) as u32;
                let spouse_birth_year = year - u32::min(spouse_age, year);
                let mut spouse = generate_person(seed, id, spouse_birth_year, person.sex.opposite(), person.culture);
                let mut person = person.clone();
                spouse.spouse = Some(person.id);
                person.spouse = Some(spouse.id);
                world_graph.events.push(Event::Marriage(year, person.id, spouse.id));
                new_people.push(person);
                new_people.push(spouse.clone());
            }

            if age > 18.0 && person.spouse.is_some() && rng.rand_chance(0.01) {
                let id = id.next();
                let seed = seed * id.0 as u32;
                let mut rng = Rng::new(seed);
                let sex;
                if rng.rand_chance(0.5) {
                    sex = PersonSex::Male;
                } else {
                    sex = PersonSex::Female;
                }
                let child = generate_person(seed, id, year, sex, person.culture);
                world_graph.events.push(Event::PersonBorn(year, child.id));
                let mut person = person.clone();
                person.heirs.push(child.id);
                new_people.push(child);
                new_people.push(person);
            }

            if age > 18.0 && !person.leader && rng.rand_chance(1.0/50.0) {

                let settlement = generate_settlement(seed, year, person.culture, &world_graph, &world_map, regions);
                
                world_graph.events.push(Event::SettlementFounded(year, settlement.clone(), person.id));
                world_graph.nodes.push(WorldGraphNode::SettlementNode(settlement));

                let mut person = person.clone();
                person.leader = true;
                new_people.push(person);
                
            }

        }

        for new_person in new_people {
            world_graph.people.insert(new_person.id, new_person);
        }

    }

    return world_graph
}

fn generate_person<'a>(seed: u32, next_id: PersonId, birth_year: u32, sex: PersonSex, culture: &'a CulturePrefab) -> Person<'a> {
    return Person {
        id: next_id,
        name: generate_name(seed, &sex, culture),
        birth: birth_year,
        sex,
        culture,
        death: 0,
        spouse: None,
        heirs: Vec::new(),
        leader: false
    }
}

fn generate_name<'a>(seed: u32, sex: &PersonSex, culture: &'a CulturePrefab) -> String {
    let first_name;
    match sex {
        PersonSex::Male => first_name = culture.first_name_male_model.generate(seed, 4, 15),
        PersonSex::Female => first_name = culture.first_name_female_model.generate(seed, 4, 15)
    }
    let first_name = Strings::capitalize(&first_name);
    let last_name = Strings::capitalize(&culture.last_name_model.generate(seed, 4, 15));
    return format!("{first_name} {last_name}");
}

fn generate_settlement<'a>(seed: u32, founding_year: u32, culture: &'a CulturePrefab, world_graph: &WorldGraph, world_map: &WorldMap, regions: &'a Vec<RegionPrefab>) -> Settlement<'a> {
    let seed = seed + founding_year as u32;
    let mut rng = Rng::new(seed);
    let mut xy;
    'candidates: loop {
        xy = Point(rng.randu_range(0, WORLD_MAP_WIDTH), rng.randu_range(0, WORLD_MAP_HEIGHT));
        for node in world_graph.nodes.iter() {
            match node {
                WorldGraphNode::SettlementNode(settlement) => {
                    if settlement.xy.dist_squared(&xy) < 5.0_f32.powi(2) {
                        continue 'candidates;
                    }
                }
            }
        }
        break;
    }
    let region_id = world_map.get_world_tile(xy.0, xy.1).region_id as usize;
    let region = regions.get(region_id).unwrap();
    return Settlement {
        xy, 
        name: String::from(generate_location_name(seed, culture, region)),
        founding_year: founding_year,
        culture: culture,
        region_id,
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
    return String::from("Settlement")
}

fn generate_world_map(seed: u32, regions: &Vec<RegionPrefab>) -> WorldMap {
    let mut map = WorldMap {
        elevation: [0; WORLD_MAP_HEIGHT * WORLD_MAP_WIDTH],
        temperature: [0; WORLD_MAP_HEIGHT * WORLD_MAP_WIDTH],
        region_id: [0; WORLD_MAP_HEIGHT * WORLD_MAP_WIDTH],
    };
    let n_elev = Perlin::new(seed + 37);
    let n_temp = Perlin::new(seed + 101);
    let n_reg = Perlin::new(seed + 537);
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

            for node in world_graph.nodes.iter() {
                match node {
                    WorldGraphNode::SettlementNode(settlement) => {
                        if settlement.xy.0 == x && settlement.xy.1 == y {
                            string = String::from("@");
                        }
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