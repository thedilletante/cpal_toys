use cpal::{Sample, FromSample};

const DEFAULT_FREQUENCY: f32 = 440.0; // A4 note
const DEFAULT_SAMPLE_RATE: u32 = 48000; // Common sample rate
const DEFAULT_CHANNELS: usize = 1; // Mono by default

pub struct TonePlayer {
    config: TonePlayerConfig,
    current_sample_clock: f32,
}

impl TonePlayer {
    pub fn with_config(config: TonePlayerConfig) -> Self {
        Self {
            config,
            current_sample_clock: 0.0,
        }
    }

    pub fn fill_buffer<T>(&mut self, buffer: &mut [T])
        where T: Sample + FromSample<f32>
    {
        for frame in buffer.chunks_mut(self.config.channels) {
            self.current_sample_clock += 1.0;
            self.current_sample_clock %= self.config.sample_rate as f32;

            let value = (
                (
                    self.current_sample_clock *
                    self.config.frequency *
                    2.0 *
                    std::f32::consts::PI
                ) /
                self.config.sample_rate as f32
            ).sin();

            let sample_value: T = T::from_sample(value);

            for channel in frame.iter_mut() {
                *channel = sample_value;
            }
        }
    }
}

pub struct TonePlayerConfig {
    frequency: f32,
    sample_rate: u32,
    channels: usize,
}

impl TonePlayerConfig {
    pub fn builder() -> TonePlayerConfigBuilder {
        TonePlayerConfigBuilder {
            frequency: None,
            sample_rate: None,
            channels: None,
        }
    }
}

impl Default for TonePlayerConfig {
    fn default() -> Self {
        Self {
            frequency: DEFAULT_FREQUENCY,
            sample_rate: DEFAULT_SAMPLE_RATE,
            channels: DEFAULT_CHANNELS,
        }
    }
}

pub struct TonePlayerConfigBuilder {
    frequency: Option<f32>,
    sample_rate: Option<u32>,
    channels: Option<usize>,
}

impl TonePlayerConfigBuilder {

    pub fn frequency(mut self, frequency: f32) -> Self {
        self.frequency = Some(frequency);
        self
    }

    pub fn sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }

    pub fn channels(mut self, channels: usize) -> Self {
        self.channels = Some(channels);
        self
    }

    pub fn build(self) -> TonePlayerConfig {
        TonePlayerConfig {
            frequency: self.frequency.unwrap_or(DEFAULT_FREQUENCY),
            sample_rate: self.sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE),
            channels: self.channels.unwrap_or(DEFAULT_CHANNELS),
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tone_player() {
        let mut player = TonePlayer::with_config(
            TonePlayerConfig::builder()
                .frequency(440.0)
                .sample_rate(48000)
                .channels(1)
                .build(),
        );

        let mut buffer: Vec<f32> = vec![0.0; 48]; // 48 samples for 1 ms at 48 kHz
        player.fill_buffer(&mut buffer);
        assert_eq!(
            buffer,
            vec![
                0.057564028, 0.11493716, 0.17192909, 0.22835088,
                0.28401536, 0.3387379, 0.3923371, 0.44463518,
                0.4954587, 0.54463905, 0.5920132, 0.637424,
                0.68072087, 0.7217602, 0.76040596, 0.79652995,
                0.8300123, 0.86074203, 0.8886173, 0.9135455,
                0.93544406, 0.9542403, 0.969872, 0.9822872,
                0.9914449, 0.99731445, 0.9998766, 0.99912286,
                0.99505556, 0.98768836, 0.9770456, 0.96316254,
                0.94608533, 0.9258706, 0.90258527, 0.8763066,
                0.84712195, 0.81512773, 0.78043044, 0.7431448,
                0.7033948, 0.66131186, 0.61703575, 0.5707136,
                0.5224985, 0.47255078, 0.42103574, 0.3681246
            ]
        );
    }
}
