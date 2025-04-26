#[derive(Clone, Debug)]
pub(crate) struct GameOptions {
    pub(crate) audio: AudioOptions
}

#[derive(Clone, Debug)]
pub(crate) struct AudioOptions {
    pub(crate) music_volume: f32
}