pub struct Attributes {
    pub strength: u8
}

impl Attributes {

    pub fn simplified_offensive_power(&self) -> f32 {
        return self.strength as f32
    }

    pub fn simplified_health(&self) -> f32 {
        return self.strength as f32 * 2.;
    }

}