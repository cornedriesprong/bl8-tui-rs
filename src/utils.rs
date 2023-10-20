pub const A4_FREQ: f32 = 440.0;
pub const A4_MIDI: u8 = 69;

pub fn midi_to_freq(pitch: i8) -> f32 {
    A4_FREQ * (2f32).powf((pitch as f32 - A4_MIDI as f32) as f32 / 12.0)
}

pub fn freq_to_midi(freq: f32) -> i8 {
    ((freq / A4_FREQ).log2() * 12.0 + A4_MIDI as f32).round() as i8
}

pub fn scale_log(value: f32, min: f32, max: f32) -> f32 {
    min * (max / min).powf(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_to_freq() {
        assert_eq!(midi_to_freq(0), 8.175798);
        assert_eq!(midi_to_freq(69), 440.0);
        assert_eq!(midi_to_freq(127), 12543.855);
    }

    #[test]
    fn test_freq_to_midi() {
        assert_eq!(freq_to_midi(8.17), 0);
        assert_eq!(freq_to_midi(440.0), 69);
        assert_eq!(freq_to_midi(12543.855), 127);
    }
}
