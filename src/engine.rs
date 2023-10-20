use crate::history::{Note, Track};

use crate::history::State;
use crate::limiter::Limiter;
use mi_plaits_dsp::dsp::voice::{Modulations, Patch, Voice};

pub const SEQ_TRACK_COUNT: usize = 8;
const BLOCK_SIZE: usize = 1;

struct Params {
    engine: usize,
    harmonics: f32,
    morph: f32,
    timbre: f32,
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
struct Channel<'a> {
    voice: Voice<'a>,
    patch: Patch,
    modulations: Modulations,
    params: Params,
}

impl Channel<'_> {
    fn new() -> Self {
        Self {
            voice: Voice::new(&std::alloc::System, BLOCK_SIZE),
            patch: Patch::default(),
            modulations: Modulations::default(),
            params: Params {
                engine: 1,
                morph: 0.5,
                harmonics: 0.5,
                timbre: 0.5,
            },
        }
    }

    fn init(&mut self) {
        self.voice.init();
        self.reset_params()
    }

    fn reset_params(&mut self) {
        // reset params to saved settings (after changing them in sequence)
        self.patch.engine = self.params.engine;
        self.patch.harmonics = self.params.harmonics;
        self.patch.timbre = self.params.timbre;
        self.patch.morph = self.params.morph;
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
    channels: [Channel<'a>; SEQ_TRACK_COUNT],
    tracks: [Track; SEQ_TRACK_COUNT],
    limiter: Limiter,
    time: f32,
    length: f32,
}

impl Engine<'_> {
    pub fn new() -> Self {
        Self {
            channels: [
                Channel::new(),
                Channel::new(),
                Channel::new(),
                Channel::new(),
                Channel::new(),
                Channel::new(),
                Channel::new(),
                Channel::new(),
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
            length: 16.0,
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

        for track_index in 0..SEQ_TRACK_COUNT {
            for note_index in 0..16 {
                if note_index as f32 != self.time {
                    continue;
                }
                if let Some(note) = self.tracks[track_index].notes[note_index] {
                    let t = &mut self.channels[track_index];
                    t.reset_params();
                    t.play(note.pitch, note.velocity);
                    note.parameters.engine.map(|v| t.patch.engine = v as usize);
                    note.parameters.harmonics.map(|v| t.patch.harmonics = v);
                    note.parameters.morph.map(|v| t.patch.morph = v);
                    note.parameters.timbre.map(|v| t.patch.timbre = v);
                }
            }
        }
        // TODO: render and mix all tracks
        // let mut mix = self.tracks.iter().reduce(|a, b| a + b);
        let mut mix = self.channels[0].tick();
        mix = self.limiter.tick(mix);

        mix
    }

    pub fn set_state(&mut self, state: State) {
        for (track_index, track) in state.iter().enumerate() {
            for (note_index, note) in track.notes.iter().enumerate() {
                self.tracks[track_index].notes[note_index] = *note;
            }
        }
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
