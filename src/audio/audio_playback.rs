use crate::audio::audio_download::ensure_audio_file;
use crate::audio::audio_handler::{analyze_audio, set_audio_spectrum, AUDIO_VIZ_BARS};
use crate::audio::white_noise::NoiseSource;
use rand::prelude::*;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;
static AUDIO_THREAD_STARTED: AtomicBool = AtomicBool::new(false);
static WHITE_NOISE_ENABLED: AtomicBool = AtomicBool::new(false);
static DOWNLOAD_ATTEMPTED: AtomicBool = AtomicBool::new(false);

pub fn start_audio_thread() -> Option<thread::JoinHandle<()>> {
    if AUDIO_THREAD_STARTED.load(Ordering::SeqCst) {
        return None;
    }
    AUDIO_THREAD_STARTED.store(true, Ordering::SeqCst);
    let audio_spectrum = Arc::new(Mutex::new(vec![0.0; AUDIO_VIZ_BARS]));
    set_audio_spectrum(audio_spectrum.clone());
    let handle = thread::spawn(move || {
        // Try to get the audio file - use blocking approach with futures executor
        // Only attempt download once per application run
        let audio_path = if !DOWNLOAD_ATTEMPTED.load(Ordering::SeqCst) {
            DOWNLOAD_ATTEMPTED.store(true, Ordering::SeqCst);
            match futures::executor::block_on(ensure_audio_file()) {
                Ok(path) => Some(path),
                Err(e) => {
                    eprintln!("Failed to ensure audio file: {}", e);
                    None
                }
            }
        } else {
            // Check if file exists without attempting download
            let potential_path = dirs::data_dir()
                .unwrap_or_else(|| std::env::current_dir().unwrap())
                .join("stimstation")
                .join("foregone_destruction_remastered.flac");
            if potential_path.exists() {
                Some(potential_path)
            } else {
                None
            }
        };
        let (_stream, stream_handle) = match OutputStream::try_default() {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Failed to get audio output stream: {}", e);
                AUDIO_THREAD_STARTED.store(false, Ordering::SeqCst);
                return;
            }
        };
        let sink = match Sink::try_new(&stream_handle) {
            Ok(sink) => sink,
            Err(e) => {
                eprintln!("Failed to create audio sink: {}", e);
                AUDIO_THREAD_STARTED.store(false, Ordering::SeqCst);
                return;
            }
        };

        // Try to load and play the audio file if available
        if let Some(path) = audio_path {
            match File::open(&path) {
                Ok(file) => {
                    match Decoder::new(BufReader::new(file)) {
                        Ok(source) => {
                            // Create a custom source that captures audio data for analysis
                            let analyzing_source =
                                AnalyzingSource::new(source, audio_spectrum.clone());
                            sink.append(analyzing_source);
                            sink.play();

                            // Keep the thread alive while audio is playing
                            while !sink.empty() && AUDIO_THREAD_STARTED.load(Ordering::SeqCst) {
                                thread::sleep(Duration::from_millis(100));
                            }

                            // Loop the audio by restarting
                            if AUDIO_THREAD_STARTED.load(Ordering::SeqCst) {
                                println!("Audio finished, restarting...");
                                AUDIO_THREAD_STARTED.store(false, Ordering::SeqCst);
                                let _ = start_audio_thread(); // Restart the audio
                            }
                            return;
                        }
                        Err(e) => {
                            eprintln!("Failed to decode audio file: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to open audio file: {}", e);
                }
            }
        }
        // Fallback to white noise if audio file couldn't be loaded
        fallback_audio_thread_with_sink(audio_spectrum, sink);
    });
    Some(handle)
}

fn fallback_audio_thread_with_sink(audio_spectrum: Arc<Mutex<Vec<f32>>>, sink: Sink) {
    if !WHITE_NOISE_ENABLED.load(Ordering::SeqCst) {
        println!("White noise disabled, stopping audio fallback");
        AUDIO_THREAD_STARTED.store(false, Ordering::SeqCst);
        return;
    }

    println!("Using fallback white noise audio (press 9 to disable)");
    let sample_rate = 44100;
    let noise = NoiseSource::new(sample_rate).with_amplitude(0.15);
    let buffer_size = 1024;
    let mut audio_buffer = vec![0.0; buffer_size];
    let mut buffer_pos = 0;
    sink.append(noise);
    while !sink.empty()
        && AUDIO_THREAD_STARTED.load(Ordering::SeqCst)
        && WHITE_NOISE_ENABLED.load(Ordering::SeqCst)
    {
        thread::sleep(Duration::from_millis(10));
        for _ in 0..buffer_size / 10 {
            let noise_val = if WHITE_NOISE_ENABLED.load(Ordering::SeqCst) {
                rand::thread_rng().gen_range(-1.0..1.0) * 0.15
            } else {
                0.0 // Silence if disabled
            };
            audio_buffer[buffer_pos] = noise_val;
            buffer_pos = (buffer_pos + 1) % buffer_size;
            if buffer_pos == 0 {
                analyze_audio(&audio_buffer, audio_spectrum.clone());
            }
        }
    }
    AUDIO_THREAD_STARTED.store(false, Ordering::SeqCst);
}

// AnalyzingSource wraps an audio source and analyzes the audio data for visualization
pub struct AnalyzingSource<S> {
    source: S,
    spectrum: Arc<Mutex<Vec<f32>>>,
    buffer: Vec<f32>,
    buffer_pos: usize,
    buffer_size: usize,
}

impl<S> AnalyzingSource<S> {
    pub fn new(source: S, spectrum: Arc<Mutex<Vec<f32>>>) -> Self {
        Self {
            source,
            spectrum,
            buffer: vec![0.0; 1024],
            buffer_pos: 0,
            buffer_size: 1024,
        }
    }
}

impl<S> Iterator for AnalyzingSource<S>
where
    S: Iterator<Item = i16>,
{
    type Item = i16;

    fn next(&mut self) -> Option<i16> {
        if let Some(sample) = self.source.next() {
            // Convert i16 sample to f32 for analysis
            let sample_f32 = sample as f32 / 32768.0;
            self.buffer[self.buffer_pos] = sample_f32;
            self.buffer_pos += 1;

            // When buffer is full, analyze it
            if self.buffer_pos >= self.buffer_size {
                analyze_audio(&self.buffer, self.spectrum.clone());
                self.buffer_pos = 0;
            }

            Some(sample)
        } else {
            None
        }
    }
}

impl<S> Source for AnalyzingSource<S>
where
    S: Source<Item = i16>,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.source.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.source.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.source.total_duration()
    }
}

pub fn is_audio_thread_started() -> bool {
    AUDIO_THREAD_STARTED.load(Ordering::SeqCst)
}
pub fn stop_audio_thread() {
    AUDIO_THREAD_STARTED.store(false, Ordering::SeqCst);
}

pub fn set_white_noise_enabled(enabled: bool) {
    WHITE_NOISE_ENABLED.store(enabled, Ordering::SeqCst);
}

pub fn is_white_noise_enabled() -> bool {
    WHITE_NOISE_ENABLED.load(Ordering::SeqCst)
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
        let sample = (self.position * 2.0 * std::f32::consts::PI * self.frequency
            / self.sample_rate as f32)
            .sin()
            * self.amplitude;
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
