use rand::prelude::*;
use rodio::Source;
use std::time::Duration;

/// White noise generation module for StimStation
///
/// This module provides white noise generation capabilities that can be used
/// for audio processing, testing, or background sound generation.
///
/// Note: This module is currently not actively used in the main application
/// but is available for future use and experimentation.

/// A source that generates white noise audio samples.
/// This struct implements the rodio::Source trait to provide continuous white noise generation
/// for audio applications. The noise is uniformly distributed random values scaled by amplitude.
pub struct NoiseSource {
    sample_rate: u32,
    position: usize,
    amplitude: f32,
}

impl NoiseSource {
    /// Creates a new NoiseSource with the specified sample rate.
    ///
    /// # Arguments
    /// * `sample_rate` - The audio sample rate in Hz (e.g., 44100)
    ///
    /// # Returns
    /// A new NoiseSource instance with default amplitude of 0.25
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            position: 0,
            amplitude: 0.25,
        }
    }

    /// Sets the amplitude (volume) of the white noise.
    ///
    /// # Arguments
    /// * `amplitude` - The amplitude value, clamped between 0.0 and 1.0
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude.clamp(0.0, 1.0);
        self
    }

    /// Gets the current amplitude value.
    pub fn amplitude(&self) -> f32 {
        self.amplitude
    }

    /// Sets the amplitude value.
    ///
    /// # Arguments
    /// * `amplitude` - The new amplitude value, clamped between 0.0 and 1.0
    pub fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude.clamp(0.0, 1.0);
    }
}

impl Iterator for NoiseSource {
    type Item = f32;

    /// Generates the next white noise sample.
    /// Each call returns a new random sample scaled by the amplitude.
    fn next(&mut self) -> Option<f32> {
        self.position += 1;
        let mut rng = rand::thread_rng();
        let noise = rng.gen_range(-1.0..1.0) * self.amplitude;
        Some(noise)
    }
}

impl Source for NoiseSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// Generates a buffer of white noise samples.
///
/// # Arguments
/// * `buffer` - Mutable slice to fill with noise samples
/// * `amplitude` - The amplitude (volume) of the noise, between 0.0 and 1.0
///
/// # Example
/// ```
/// let mut buffer = vec![0.0; 1024];
/// generate_white_noise_buffer(&mut buffer, 0.5);
/// ```
pub fn generate_white_noise_buffer(buffer: &mut [f32], amplitude: f32) {
    let amplitude = amplitude.clamp(0.0, 1.0);
    let mut rng = rand::thread_rng();

    for sample in buffer.iter_mut() {
        *sample = rng.gen_range(-1.0..1.0) * amplitude;
    }
}

/// Generates a single white noise sample.
///
/// # Arguments
/// * `amplitude` - The amplitude (volume) of the noise, between 0.0 and 1.0
///
/// # Returns
/// A single white noise sample as f32
pub fn generate_white_noise_sample(amplitude: f32) -> f32 {
    let amplitude = amplitude.clamp(0.0, 1.0);
    let mut rng = rand::thread_rng();
    rng.gen_range(-1.0..1.0) * amplitude
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_source_creation() {
        let noise = NoiseSource::new(44100);
        assert_eq!(noise.sample_rate(), 44100);
        assert_eq!(noise.amplitude(), 0.25);
    }

    #[test]
    fn test_noise_source_with_amplitude() {
        let noise = NoiseSource::new(44100).with_amplitude(0.5);
        assert_eq!(noise.amplitude(), 0.5);
    }

    #[test]
    fn test_amplitude_clamping() {
        let noise = NoiseSource::new(44100).with_amplitude(2.0);
        assert_eq!(noise.amplitude(), 1.0);

        let noise = NoiseSource::new(44100).with_amplitude(-0.5);
        assert_eq!(noise.amplitude(), 0.0);
    }

    #[test]
    fn test_generate_white_noise_buffer() {
        let mut buffer = vec![0.0; 100];
        generate_white_noise_buffer(&mut buffer, 0.5);

        // Check that buffer is no longer all zeros
        assert!(buffer.iter().any(|&x| x != 0.0));

        // Check that all values are within expected range
        for &sample in &buffer {
            assert!(sample >= -0.5 && sample <= 0.5);
        }
    }

    #[test]
    fn test_generate_white_noise_sample() {
        let sample = generate_white_noise_sample(0.5);
        assert!(sample >= -0.5 && sample <= 0.5);
    }
}
