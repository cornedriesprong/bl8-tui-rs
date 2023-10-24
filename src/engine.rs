use crate::history::Grid;
use crate::limiter::Limiter;
use crate::utils::midi_to_freq;
use crossbeam::channel::*;
use mi_plaits_dsp::dsp::drums::*;
use mi_plaits_dsp::dsp::voice::{Modulations, Patch, Voice};

pub const SEQ_TRACK_COUNT: usize = 8;
pub const INITIAL_STEP_COUNT: usize = 16;
const BLOCK_SIZE: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Parameters {
    pub engine: Option<f32>,
    pub harmonics: Option<f32>,
    pub morph: Option<f32>,
    pub timbre: Option<f32>,
}

impl Parameters {
    pub fn new() -> Parameters {
        Parameters {
            engine: None,
            harmonics: None,
            morph: None,
            timbre: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Note {
    pub timestamp: f32,
    pub pitch: i8,
    pub velocity: i8,
    pub parameters: Parameters,
}

impl Note {
    pub fn new(timestamp: f32, pitch: i8, velocity: i8) -> Note {
        Note {
            timestamp,
            pitch,
            velocity,
            parameters: Parameters::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Track {
    pub notes: [Option<Note>; 16],
}

pub type State = [Track; SEQ_TRACK_COUNT];

struct Kick {
    engine: analog_bass_drum::AnalogBassDrum,
    pitch: i8,
    trigger: bool,
}

impl Kick {
    pub fn new() -> Self {
        return Self {
            engine: analog_bass_drum::AnalogBassDrum::new(),
            pitch: 40,
            trigger: false,
        };
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        let mut out = [0.0; BLOCK_SIZE];

        let f0 = midi_to_freq(self.pitch) / 48000.0;
        self.engine
            .render(false, self.trigger, 1.0, f0, 1.0, 0.5, 0.0, 0.0, &mut out);
        self.trigger = false;

        out[0]
    }

    fn play(&mut self, pitch: i8, velocity: i8) {
        self.pitch = pitch;
        self.trigger = true;
    }
}

struct Snare {
    engine: analog_snare_drum::AnalogSnareDrum,
    pitch: i8,
    trigger: bool,
}

impl Snare {
    pub fn new() -> Self {
        return Self {
            engine: analog_snare_drum::AnalogSnareDrum::new(),
            pitch: 40,
            trigger: false,
        };
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        let mut out = [0.0; BLOCK_SIZE];

        let f0 = midi_to_freq(self.pitch) / 48000.0;
        self.engine
            .render(false, self.trigger, 1.0, f0, 0.5, 0.5, 0.5, &mut out);
        self.trigger = false;

        out[0]
    }

    fn play(&mut self, pitch: i8, velocity: i8) {
        self.pitch = pitch;
        self.trigger = true;
    }
}

struct Hihat {
    engine: hihat::Hihat,
    pitch: i8,
    trigger: bool,
}

impl Hihat {
    pub fn new() -> Self {
        return Self {
            engine: hihat::Hihat::new(),
            pitch: 40,
            trigger: false,
        };
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        let mut out = [0.0; BLOCK_SIZE];
        let mut temp_1 = [0.0; BLOCK_SIZE];
        let mut temp_2 = [0.0; BLOCK_SIZE];

        let f0 = midi_to_freq(self.pitch) / 48000.0;
        self.engine.render(
            false,
            self.trigger,
            1.0,
            f0,
            0.5,
            0.5,
            0.5,
            &mut temp_1,
            &mut temp_2,
            &mut out,
            hihat::NoiseType::RingMod,
            hihat::VcaType::Swing,
            false,
            false,
        );
        self.trigger = false;

        out[0]
    }

    fn play(&mut self, pitch: i8, velocity: i8) {
        self.pitch = pitch;
        self.trigger = true;
    }
}

// 0 virtual_analog_vcf_engine
// 1 phase_distortion_engine
// 2 six_op_engie
// 3 six_op_engine
// 4 six_op_engine
// 5 waveshaping_engine
// 6 six_op_engine
// 7 waveterrain_engine
// 8 string_machine_engine
// 9 chiptune_engine
// 10 virtual_analog_engine
// 11 waveshaping_engine
// 12 fm_engine
// 13 grain_engine
// 14 additive_engine
// 15 wavetable_engine
// 16 chord_engine
// 17 speech_engine
// 18 swarm_engine
// 19 noise_engine
// 20 particle_engine
// 21 string_engine
// 22 modal_engine
// 23 bass_drum_engine
// 24 snare_drum_engine
// 25 hihat_engine
struct Synth<'a> {
    voice: Voice<'a>,
    patch: Patch,
    modulations: Modulations,
    engine: usize,
    harmonics: f32,
    morph: f32,
    timbre: f32,
}

impl Synth<'_> {
    fn new() -> Self {
        Self {
            voice: Voice::new(&std::alloc::System, BLOCK_SIZE),
            patch: Patch::default(),
            modulations: Modulations::default(),
            engine: 1,
            morph: 0.5,
            harmonics: 0.5,
            timbre: 0.5,
        }
    }

    fn init(&mut self) {
        self.modulations.trigger_patched = true;
        self.modulations.level_patched = true;
        self.voice.init();
        self.reset_params();
    }

    fn reset_params(&mut self) {
        // reset params to saved settings (after changing them in sequence)
        self.patch.engine = self.engine;
        self.patch.harmonics = self.harmonics;
        self.patch.timbre = self.timbre;
        self.patch.morph = self.morph;
    }

    fn play(&mut self, pitch: i8, velocity: i8) {
        // note off
        self.modulations.trigger = 0.0;
        self.modulations.level = 0.0;

        // TODO: fix this
        self.tick();

        // note on
        self.patch.note = pitch as f32;
        self.modulations.trigger = 1.0;
        self.modulations.level = velocity as f32 / 127.0;
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        let mut out = [0.0; BLOCK_SIZE];
        let mut aux = [0.0; BLOCK_SIZE];

        self.voice
            .render(&self.patch, &self.modulations, &mut out, &mut aux);

        out[0]
    }
}

pub struct Engine<'a> {
    kick: Kick,
    snare: Snare,
    hihat: Hihat,
    channels: [Synth<'a>; SEQ_TRACK_COUNT],
    tracks: [Track; SEQ_TRACK_COUNT],
    limiter: Limiter,
    time: f32,
    prev_time: i8,
    length: f32,
    pub ui_channel: (Sender<i8>, Receiver<i8>),
}

impl Engine<'_> {
    pub fn new() -> Self {
        Self {
            kick: Kick::new(),
            snare: Snare::new(),
            hihat: Hihat::new(),
            channels: [
                Synth::new(),
                Synth::new(),
                Synth::new(),
                Synth::new(),
                Synth::new(),
                Synth::new(),
                Synth::new(),
                Synth::new(),
            ],
            tracks: [
                Track { notes: [None; 16] },
                Track { notes: [None; 16] },
                Track { notes: [None; 16] },
                Track { notes: [None; 16] },
                Track { notes: [None; 16] },
                Track { notes: [None; 16] },
                Track { notes: [None; 16] },
                Track { notes: [None; 16] },
            ],
            limiter: Limiter::new(10.0, 500.0, 1.0),
            time: 0.0,
            prev_time: 0,
            length: 16.0,
            ui_channel: crossbeam::channel::unbounded(),
        }
    }

    pub fn init(&mut self) {
        for track in &mut self.channels {
            track.init()
        }
    }

    #[inline]
    pub fn tick(&mut self) -> f32 {
        self.increment_time();

        if self.time as i8 != self.prev_time {
            self.prev_time = self.time as i8;
            self.ui_channel.0.send(self.time as i8).unwrap();
        }

        for track_idx in 0..SEQ_TRACK_COUNT {
            for note_idx in 0..16 {
                if note_idx as f32 != self.time {
                    continue;
                }
                if let Some(note) = self.tracks[track_idx].notes[note_idx] {
                    if track_idx == 0 {
                        self.kick.play(note.pitch, note.velocity);
                    } else if track_idx == 1 {
                        self.snare.play(note.pitch, note.velocity);
                    } else if track_idx == 2 {
                        self.hihat.play(note.pitch, note.velocity);
                    } else {
                        let t = &mut self.channels[track_idx];
                        t.reset_params();
                        t.play(note.pitch, note.velocity);
                        note.parameters.engine.map(|v| t.patch.engine = v as usize);
                        note.parameters.harmonics.map(|v| t.patch.harmonics = v);
                        note.parameters.morph.map(|v| t.patch.morph = v);
                        note.parameters.timbre.map(|v| t.patch.timbre = v);
                    }
                }
            }
        }
        // TODO: render and mix all tracks
        // let mut mix = self.tracks.iter().reduce(|a, b| a + b);
        let mult = 1.0 / 3.0;
        let mut mix = self.kick.tick() * mult;
        mix += self.snare.tick() * mult;
        mix += self.hihat.tick() * mult;

        mix = self.limiter.tick(mix);

        mix
    }

    pub fn set_state(&mut self, state: State) {
        self.tracks = state;
    }

    pub fn clear_track(&mut self, track_index: usize) {
        // self.seq.clear_track(track_index);
    }

    pub fn clear_all(&mut self) {
        // clear all tracks
        for i in 0..SEQ_TRACK_COUNT {
            self.clear_track(i);
        }
    }

    fn increment_time(&mut self) {
        self.time += 1.0 / 16384.0;
        if self.time >= self.length {
            self.time = 0.0;
        }
    }
}
