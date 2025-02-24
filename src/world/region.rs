#[derive(Debug, Clone)]
pub struct Region {
    pub name: String,
    pub id: usize,
    pub elevation: (i32, i32),
    pub temperature: (u8, u8),
    pub vegetation: (f32, f32),
    pub soil_fertility_range: (f32, f32),
    pub gold_generation_range: (f32, f32),
    pub fauna: Vec<String>,
    pub flora: Vec<String>,
}