// Audio playback and noise generation
use rodio::{OutputStream, Sink, Source};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use rand::prelude::*;

/// Audio backend trait for abstraction
pub trait AudioBackend {
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self);
    fn is_running(&self) -> bool;
}

/// Audio playback manager
pub struct AudioPlaybackManager {
    audio_thread: Option<thread::JoinHandle<()>>,
    is_running: Arc<Mutex<bool>>,
    audio_spectrum: Arc<Mutex<Vec<f32>>>,
}

impl AudioPlaybackManager {
    /// Create a new audio playback manager
    pub fn new(audio_spectrum: Arc<Mutex<Vec<f32>>>) -> Self {
        Self {
            audio_thread: None,
            is_running: Arc::new(Mutex::new(false)),
            audio_spectrum,
        }
    }
}

impl AudioBackend for AudioPlaybackManager {
    /// Start the audio playback thread
    fn start(&mut self) -> Result<(), String> {
        // Check if already started
        if *self.is_running.lock().unwrap() {
            return Ok(());
        }
        
        let is_running = self.is_running.clone();
        let audio_spectrum = self.audio_spectrum.clone();
        
        // Create a thread for audio playback and analysis
        let handle = thread::spawn(move || {
            // Mark as started
            *is_running.lock().unwrap() = true;
            
            // Try to get the output stream
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Failed to get audio output stream: {}", e);
                    *is_running.lock().unwrap() = false;
                    return;
                }
            };
            
            // Create a sink to play our audio
            let sink = match Sink::try_new(&stream_handle) {
                Ok(sink) => sink,
                Err(e) => {
                    eprintln!("Failed to create audio sink: {}", e);
                    *is_running.lock().unwrap() = false;
                    return;
                }
            };
            
            // Create our noise source
            let sample_rate = 44100;
            let noise = NoiseSource::new(sample_rate).with_amplitude(0.15); // Lower volume
            
            // Set up a buffer to analyze
            let buffer_size = 1024;
            let mut audio_buffer = vec![0.0; buffer_size];
            let mut buffer_pos = 0;
            
            // Add the noise source to the sink
            sink.append(noise);
            
            // Keep the sink playing and analyze audio
            while !sink.empty() && *is_running.lock().unwrap() {
                // Sleep a bit to avoid hogging the CPU
                thread::sleep(Duration::from_millis(10));
                
                // Simulate audio capture and analysis
                for _ in 0..buffer_size/10 {  // Process some samples each time
                    // Generate a new sample (similar to our noise source)
                    let noise = rand::thread_rng().gen_range(-1.0..1.0) * 0.15;
                    
                    // Add to buffer
                    audio_buffer[buffer_pos] = noise;
                    buffer_pos = (buffer_pos + 1) % buffer_size;
                    
                    // Every time we fill the buffer, analyze it
                    if buffer_pos == 0 {
                        Self::analyze_audio(&audio_buffer, audio_spectrum.clone());
                    }
                }
            }
            
            // Mark as stopped when thread exits
            *is_running.lock().unwrap() = false;
        });
        
        self.audio_thread = Some(handle);
        Ok(())
    }
    
    /// Stop the audio thread
    fn stop(&mut self) {
        *self.is_running.lock().unwrap() = false;
        
        // Wait for thread to finish if needed
        if let Some(handle) = self.audio_thread.take() {
            let _ = handle.join();
        }
    }
    
    /// Check if audio is running
    fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap()
    }
}

impl AudioPlaybackManager {
    /// Analyze audio buffer and update spectrum data
    fn analyze_audio(buffer: &[f32], spectrum: Arc<Mutex<Vec<f32>>>) {
        // Get a lock on the spectrum data
        if let Ok(mut spec) = spectrum.lock() {
            let num_bars = spec.len();
            
            // Simple amplitude-based analysis
            // In a real app, you might use FFT here
            for i in 0..num_bars {
                let start = buffer.len() * i / num_bars;
                let end = buffer.len() * (i + 1) / num_bars;
                
                // Calculate average amplitude for this frequency band
                let mut sum = 0.0;
                let mut count = 0;
                
                for j in start..end {
                    if j < buffer.len() {
                        sum += buffer[j].abs();
                        count += 1;
                    }
                }
                
                let avg = if count > 0 { sum / count as f32 } else { 0.0 };
                
                // Apply some scaling and smoothing
                spec[i] = (spec[i] * 0.7 + avg * 3.0 * 0.3).min(1.0);
            }
        }
    }
}

/// White noise generator for rodio
pub struct NoiseSource {
    sample_rate: u32,
    position: usize,
    amplitude: f32,
}

impl NoiseSource {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            position: 0,
            amplitude: 0.25, // 25% volume to avoid being too loud
        }
    }
    
    pub fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude.clamp(0.0, 1.0);
        self
    }
}

impl Iterator for NoiseSource {
    type Item = f32;
    
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
        1 // Mono
    }
    
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    
    fn total_duration(&self) -> Option<Duration> {
        None // Infinite
    }
}

/// Create a simple tone generator (alternative to noise)
pub struct ToneSource {
    sample_rate: u32,
    frequency: f32,
    amplitude: f32,
    position: f32,
}

impl ToneSource {
    pub fn new(sample_rate: u32, frequency: f32) -> Self {
        Self {
            sample_rate,
            frequency,
            amplitude: 0.1,
            position: 0.0,
        }
    }
    
    pub fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude.clamp(0.0, 1.0);
        self
    }
}

impl Iterator for ToneSource {
    type Item = f32;
    
    fn next(&mut self) -> Option<f32> {
        let sample = (self.position * 2.0 * std::f32::consts::PI * self.frequency / self.sample_rate as f32).sin() * self.amplitude;
        self.position += 1.0;
        if self.position >= self.sample_rate as f32 {
            self.position = 0.0;
        }
        Some(sample)
    }
}

impl Source for ToneSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    
    fn channels(&self) -> u16 {
        1 // Mono
    }
    
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    
    fn total_duration(&self) -> Option<Duration> {
        None // Infinite
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_noise_source() {
        let mut noise = NoiseSource::new(44100);
        
        // Test that it generates values in the expected range
        for _ in 0..100 {
            if let Some(sample) = noise.next() {
                assert!(sample >= -0.25);
                assert!(sample <= 0.25);
            }
        }
    }
    
    #[test]
    fn test_tone_source() {
        let mut tone = ToneSource::new(44100, 440.0);
        
        // Test that it generates sine wave
        for _ in 0..100 {
            if let Some(sample) = tone.next() {
                assert!(sample >= -0.1);
                assert!(sample <= 0.1);
            }
        }
    }
}
