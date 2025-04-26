#[derive(Debug, Clone)]
pub(crate) struct Region {
    pub(crate) name: String,
    pub(crate) id: usize,
    pub(crate) elevation: (i32, i32),
    pub(crate) temperature: (u8, u8),
    pub(crate) vegetation: (f32, f32),
    pub(crate) soil_fertility_range: (f32, f32),
    pub(crate) gold_generation_range: (f32, f32),
    pub(crate) fauna: Vec<String>,
    pub(crate) flora: Vec<String>,
}