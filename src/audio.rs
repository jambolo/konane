#[cfg(feature = "audio")]
use kira::{
    manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
#[cfg(feature = "audio")]
use std::io::Cursor;

/// Audio manager for game sound effects
pub struct GameAudio {
    #[cfg(feature = "audio")]
    manager: Option<AudioManager>,
    #[cfg(feature = "audio")]
    move_sound: Option<StaticSoundData>,
    #[cfg(feature = "audio")]
    capture_sound: Option<StaticSoundData>,
}

impl GameAudio {
    #[cfg(feature = "audio")]
    pub fn new() -> Self {
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).ok();

        // Generate simple sounds programmatically
        let move_sound = generate_click_sound(440.0, 0.1);
        let capture_sound = generate_click_sound(330.0, 0.15);

        Self {
            manager,
            move_sound,
            capture_sound,
        }
    }

    #[cfg(not(feature = "audio"))]
    pub fn new() -> Self {
        Self {}
    }

    /// Play sound when a stone is moved
    #[cfg(feature = "audio")]
    pub fn play_move(&mut self) {
        if let (Some(manager), Some(sound)) = (&mut self.manager, &self.move_sound) {
            let _ = manager.play(sound.clone());
        }
    }

    #[cfg(not(feature = "audio"))]
    pub fn play_move(&mut self) {}

    /// Play sound when a stone is captured/removed
    #[cfg(feature = "audio")]
    pub fn play_capture(&mut self) {
        if let (Some(manager), Some(sound)) = (&mut self.manager, &self.capture_sound) {
            let _ = manager.play(sound.clone());
        }
    }

    #[cfg(not(feature = "audio"))]
    pub fn play_capture(&mut self) {}
}

impl Default for GameAudio {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "audio")]
/// Generate a simple click/tap sound as WAV data
fn generate_click_sound(frequency: f32, duration: f32) -> Option<StaticSoundData> {
    let sample_rate = 44100u32;
    let num_samples = (sample_rate as f32 * duration) as usize;

    // Generate samples
    let mut samples: Vec<i16> = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        // Sine wave with exponential decay envelope
        let envelope = (-t * 20.0).exp();
        let sample = (frequency * 2.0 * std::f32::consts::PI * t).sin() * envelope;
        samples.push((sample * 32767.0 * 0.5) as i16);
    }

    // Create WAV data in memory
    let wav_data = create_wav_data(&samples, sample_rate);
    let cursor = Cursor::new(wav_data);

    StaticSoundData::from_cursor(cursor, StaticSoundSettings::default()).ok()
}

#[cfg(feature = "audio")]
/// Create a simple mono WAV file in memory
fn create_wav_data(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
    let block_align = num_channels * bits_per_sample / 8;
    let data_size = samples.len() as u32 * 2;
    let file_size = 36 + data_size;

    let mut wav = Vec::with_capacity(44 + samples.len() * 2);

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt subchunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size
    wav.extend_from_slice(&1u16.to_le_bytes()); // AudioFormat (PCM)
    wav.extend_from_slice(&num_channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data subchunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    for sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }

    wav
}
