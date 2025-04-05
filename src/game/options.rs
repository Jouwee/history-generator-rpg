#[derive(Clone, Debug)]
pub struct GameOptions {
    pub audio: AudioOptions
}

#[derive(Clone, Debug)]
pub struct AudioOptions {
    pub music_volume: f32
}