#[derive(Debug)]
pub struct Region {
    pub name: String,
    pub id: usize,
    pub elevation: (u8, u8),
    pub temperature: (u8, u8),
    pub soil_fertility_range: (f32, f32),
    pub gold_generation_range: (f32, f32),
    pub fauna: Vec<String>,
    pub flora: Vec<String>,
}