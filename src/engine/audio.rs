use std::{collections::HashMap, fs::File, io::BufReader};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use crate::{commons::rng::Rng, game::options::AudioOptions};

use super::{geometry::Vec2, scene::Update};


pub(crate) struct Audio {
    // Not directly used, but needs to have ownership otherwise the audio strea is dropped
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    options: AudioOptions,
    tracks: HashMap<TrackMood, (usize, Vec<SoundFile>)>,
    music_sink_a: Sink,
    music_sink_b: Sink,
    sink: MusicSink,
    transition: f32,
    currently_playing: TrackMood,
    current_mood: TrackMood
}

#[derive(Clone, Hash, PartialEq, Eq)]
enum MusicSink {
    SinkA, SinkB,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub(crate) enum TrackMood {
    Silence,
    Regular,
    Battle
}

impl Audio {

    pub(crate) fn new(options: AudioOptions) -> Audio {
        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let music_sink_a = Sink::try_new(&stream_handle).unwrap();
        let music_sink_b = Sink::try_new(&stream_handle).unwrap();
        Audio {
            _stream: stream,
            stream_handle,
            options,
            music_sink_a,
            music_sink_b,
            sink: MusicSink::SinkA,
            tracks: HashMap::new(),
            currently_playing: TrackMood::Silence,
            current_mood: TrackMood::Silence,
            transition: 1.,
        }
    }

    pub(crate) fn update(&mut self, update: &Update) {
        if self.currently_playing != self.current_mood {
            self.transition = 0.;
            self.currently_playing = self.current_mood.clone();
            self.sink = match &self.sink {
                MusicSink::SinkA => MusicSink::SinkB,
                MusicSink::SinkB => MusicSink::SinkA,
            };
        }
        if self.transition != -1. {
            self.transition = self.transition + update.delta_time as f32;
            let sink_fadeout = match &self.sink {
                MusicSink::SinkA => &self.music_sink_b,
                MusicSink::SinkB => &self.music_sink_a,
            };
            let sink_fadein = match &self.sink {
                MusicSink::SinkA => &self.music_sink_a,
                MusicSink::SinkB => &self.music_sink_b,
            };
            let volume = self.options.music_volume;
            sink_fadeout.set_volume(volume * (1. - self.transition));
            sink_fadein.set_volume(volume * self.transition);
            if self.transition > 1. {
                self.transition = -1.;
                sink_fadein.set_volume(volume);
                sink_fadeout.clear();
            }
        }
        let sink = match &self.sink {
            MusicSink::SinkA => &self.music_sink_a,
            MusicSink::SinkB => &self.music_sink_b,
        };
        if sink.empty() {
            let (i, vec) = self.tracks.get_mut(&self.current_mood).unwrap();
            *i = (*i + 1) % vec.len();
            let sound = vec.get(*i).unwrap();
            sink.append(sound.source().amplify(0.2));
            sink.play();
        }
    }

    pub(crate) fn register_track(&mut self, mood: TrackMood, sound: SoundFile) {
        if !self.tracks.contains_key(&mood) {
            self.tracks.insert(mood.clone(), (0, Vec::new()));
        }
        let (_, vec) = self.tracks.get_mut(&mood).unwrap();
        vec.push(sound);
    }

    pub(crate) fn switch_music(&mut self, mood: TrackMood) {
        self.current_mood = mood;
    }

    pub(crate) fn play_once(&self, sound: impl Sound) {
        if let Err(err) = self.stream_handle.play_raw(sound.source()) {
            println!("Error playing audio: {err}")
        }
    }

    pub(crate) fn play_positional(&self, sound: impl Sound, sound_origin: Vec2, camera: Vec2) {
        let dist = sound_origin.dist_squared(&camera);
        let volume = (dist / (48.*48.)).clamp(0., 1.);
        let volume = 1. - volume;
        if let Err(err) = self.stream_handle.play_raw(sound.source().amplify(volume)) {
            println!("Error playing audio: {err}")
        }
    }

}

pub(crate) trait Sound {

    fn source(&self) -> impl Source<Item = f32> + Send + 'static;

}

#[derive(Clone)]
pub(crate) struct SoundFile {
    path: String
}

impl SoundFile {

    pub(crate) fn new(path: &str) -> SoundFile {
        SoundFile { path: format!("assets/sounds/{path}") }
    }
    
}

impl Sound for SoundFile {
    fn source(&self) -> impl Source<Item = f32> + Send + 'static {
        // TODO: Ideally it wouldn't load the file everytime
        let file = BufReader::new(File::open(&self.path).unwrap());
        let source = Decoder::new(file).unwrap();
        return source.convert_samples();
        
    }
}

#[derive(Clone)]
pub(crate) struct SoundEffect {
    files: Vec<SoundFile>,
    pitch_rand: [f32; 2]
}

impl SoundEffect {

    pub(crate) fn new(paths: Vec<&str>) -> SoundEffect {
        SoundEffect { files: paths.iter().map(|f| SoundFile::new(f)).collect(), pitch_rand: [0.8, 1.2] }
    }

}

impl Sound for SoundEffect {
    fn source(&self) -> impl Source<Item = f32> + Send + 'static {
        let mut rng = Rng::rand();
        let file = self.files.get(rng.randu_range(0, self.files.len())).unwrap();
        let speed = rng.randf_range(self.pitch_rand[0], self.pitch_rand[1]);
        return file.source().speed(speed)
    }
}