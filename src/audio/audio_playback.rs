use rodio::{OutputStream, Sink, Source};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;
use rand::prelude::*;
use crate::audio::audio_handler::{analyze_audio, set_audio_spectrum, AUDIO_VIZ_BARS};
static AUDIO_THREAD_STARTED: AtomicBool = AtomicBool::new(false);
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
            amplitude: 0.25,
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
        1
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
pub fn start_audio_thread() -> Option<thread::JoinHandle<()>> {
    if AUDIO_THREAD_STARTED.load(Ordering::SeqCst) {
        return None;
    }
    AUDIO_THREAD_STARTED.store(true, Ordering::SeqCst);
    let audio_spectrum = Arc::new(Mutex::new(vec![0.0; AUDIO_VIZ_BARS]));
    set_audio_spectrum(audio_spectrum.clone());
    let handle = thread::spawn(move || {
        let (_stream, stream_handle) = match OutputStream::try_default() {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Failed to get audio output stream: {}", e);
                return;
            }
        };
        let sink = match Sink::try_new(&stream_handle) {
            Ok(sink) => sink,
            Err(e) => {
                eprintln!("Failed to create audio sink: {}", e);
                return;
            }
        };
        let sample_rate = 44100;
        let noise = NoiseSource::new(sample_rate).with_amplitude(0.15);
        let buffer_size = 1024;
        let mut audio_buffer = vec![0.0; buffer_size];
        let mut buffer_pos = 0;
        sink.append(noise);
        while !sink.empty() {
            thread::sleep(Duration::from_millis(10));
            for _ in 0..buffer_size/10 {
                let noise = rand::thread_rng().gen_range(-1.0..1.0) * 0.15;
                audio_buffer[buffer_pos] = noise;
                buffer_pos = (buffer_pos + 1) % buffer_size;
                if buffer_pos == 0 {
                    analyze_audio(&audio_buffer, audio_spectrum.clone());
                }
            }
        }
        AUDIO_THREAD_STARTED.store(false, Ordering::SeqCst);
    });
    Some(handle)
}
pub fn is_audio_thread_started() -> bool {
    AUDIO_THREAD_STARTED.load(Ordering::SeqCst)
}
pub fn stop_audio_thread() {
    AUDIO_THREAD_STARTED.store(false, Ordering::SeqCst);
}
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
        1
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
