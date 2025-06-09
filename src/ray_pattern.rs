// Ray pattern visualization inspired by the yellow and green rays image
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread;
use rodio::{OutputStream, Sink, Source};
use std::time::Duration;
use winit::monitor::MonitorHandle;

const RAY_COUNT: usize = 60;  // Number of rays for each source

// Audio visualizer settings
const AUDIO_VIZ_BARS: usize = 32;
const AUDIO_VIZ_BASE_HEIGHT: f32 = 60.0;  // Base height for 1080p screen
const AUDIO_VIZ_MIN_HEIGHT: f32 = 5.0;    // Minimum height for bars
const AUDIO_VIZ_DECAY_RATE: f32 = 2.0;    // How quickly the bars react to changes

// Static variables for ball positions and velocities
static mut YELLOW_POS: Option<(f32, f32)> = None;
static mut YELLOW_VEL: Option<(f32, f32)> = None;
static mut GREEN_POS: Option<(f32, f32)> = None;
static mut GREEN_VEL: Option<(f32, f32)> = None;
static mut LAST_TIME: Option<f32> = None;

// Sorting visualization data
const SORT_ARRAY_SIZE: usize = 200; // A more reasonable size that won't cause overflow
struct SortVisualizer {
    array: Vec<u8>,
    steps: usize,
    algorithm: SortAlgorithm,
    state: SortState,
    i: usize,
    j: usize,
    pivot: usize,
    stack: Vec<(usize, usize)>, // For quicksort
    comparisons: usize,
    accesses: usize,
}

#[derive(Debug, PartialEq)]
enum SortAlgorithm {
    Bogo,
    Bubble,
    Quick,
}

#[derive(Debug, PartialEq)]
enum SortState {
    Running,
    Completed,
    Restarting,
}

impl SortVisualizer {
    fn new(algorithm: SortAlgorithm) -> Self {
        // Create a properly sized array from 1 to SORT_ARRAY_SIZE
        let mut array = Vec::with_capacity(SORT_ARRAY_SIZE);
        for i in 1..=SORT_ARRAY_SIZE {
            array.push((i % 255) as u8); // Ensure values fit in u8
        }
        
        let mut rng = rand::thread_rng();
        // Shuffle the array for sorting
        for i in 0..array.len() {
            let j = rng.gen_range(0..array.len());
            array.swap(i, j);
        }
        
        Self {
            array,
            steps: 0,
            algorithm,
            state: SortState::Running,
            i: 0,
            j: 0,
            pivot: 0,
            stack: Vec::new(),
            comparisons: 0,
            accesses: 0,
        }
    }
    
    fn update(&mut self) {
        // Skip if completed
        if self.state == SortState::Completed {
            return;
        }
        
        // Reset if restarting
        if self.state == SortState::Restarting {
            // Shuffle the array again
            let mut rng = rand::thread_rng();
            for i in 0..self.array.len() {
                let j = rng.gen_range(0..self.array.len());
                self.array.swap(i, j);
            }
            
            // Reset state
            self.state = SortState::Running;
            self.steps = 0;
            self.i = 0;
            self.j = 0;
            self.comparisons = 0;
            self.accesses = 0;
            self.stack.clear();
            
            // For quicksort, initialize the stack
            if self.algorithm == SortAlgorithm::Quick {
                self.stack.push((0, self.array.len() - 1));
            }
            
            return;
        }
        
        // Update based on algorithm
        match self.algorithm {
            SortAlgorithm::Bogo => self.update_bogo(),
            SortAlgorithm::Bubble => self.update_bubble(),
            SortAlgorithm::Quick => self.update_quick(),
        }
        
        // Increment steps
        self.steps += 1;
    }
    
    fn update_bogo(&mut self) {
        // Bogo sort just randomly shuffles until it gets lucky
        // We'll check if it's sorted, and if not, shuffle again
        let mut is_sorted = true;
        for i in 1..self.array.len() {
            self.accesses += 2; // Reading two elements
            if self.array[i-1] > self.array[i] {
                is_sorted = false;
                break;
            }
        }
        
        if is_sorted {
            self.state = SortState::Completed;
        } else {
            // Shuffle the array
            let mut rng = rand::thread_rng();
            for i in 0..self.array.len() {
                let j = rng.gen_range(0..self.array.len());
                self.array.swap(i, j);
                self.accesses += 4; // Reading and writing two elements
            }
        }
    }
    
    fn update_bubble(&mut self) {
        // Bubble sort implementation
        // One step of bubble sort
        let n = self.array.len();
        
        // Check if we've completed a full pass
        if self.i >= n {
            // Check if any swaps were made in this pass
            if self.j == 0 {
                // No swaps were made, array is sorted
                self.state = SortState::Completed;
            } else {
                // Start a new pass
                self.i = 0;
                self.j = 0;
            }
            return;
        }
        
        // Compare adjacent elements
        if self.i < n - 1 {
            self.comparisons += 1;
            self.accesses += 2; // Reading two elements
            
            if self.array[self.i] > self.array[self.i + 1] {
                // Swap elements
                self.array.swap(self.i, self.i + 1);
                self.accesses += 2; // Writing two elements
                self.j += 1; // Count the swap
            }
        }
        
        // Move to next comparison
        self.i += 1;
    }
    
    fn update_quick(&mut self) {
        // Quick sort implementation (iterative version using a stack)
        if self.stack.is_empty() {
            self.state = SortState::Completed;
            return;
        }
        
        // Pop a subarray from stack
        let (low, high) = self.stack.pop().unwrap();
        
        // If there's only one element in this subarray, continue
        if low >= high {
            return;
        }
        
        // Partition the array
        let pivot = self.partition(low, high);
        
        // Push subarrays to stack
        if pivot > 0 && pivot - 1 > low {
            self.stack.push((low, pivot - 1));
        }
        if pivot + 1 < high {
            self.stack.push((pivot + 1, high));
        }
    }
    
    fn partition(&mut self, low: usize, high: usize) -> usize {
        let pivot = self.array[high];
        self.accesses += 1;
        
        let mut i = low;
        
        for j in low..high {
            self.comparisons += 1;
            self.accesses += 1;
            
            if self.array[j] <= pivot {
                self.array.swap(i, j);
                self.accesses += 4;
                i += 1;
            }
        }
        
        self.array.swap(i, high);
        self.accesses += 4;
        
        i
    }
    
    fn draw(&self, frame: &mut [u8], x: usize, y: usize, width: usize, height: usize, 
           horizontal: bool, x_offset: usize, buffer_width: u32) {
        let n = self.array.len();
        
        // Create a gradient from red to green based on sort completion
        let sorted_percent = self.get_sorted_percent();
        let r = ((1.0 - sorted_percent) * 255.0) as u8;
        let g = (sorted_percent * 255.0) as u8;
        let b = 50; // Add some blue for visual interest
        
        if horizontal {
            // Horizontal visualization (left to right)
            let bar_width = width as f32 / n as f32;
            
            for i in 0..n {
                let bar_height = (self.array[i] as f32 / 255.0) * height as f32;
                let bar_x = x + (i as f32 * bar_width) as usize;
                let bar_y = y + height - bar_height as usize;
                
                // Highlight current indices being compared
                let color = if (self.state == SortState::Running && 
                               ((self.algorithm == SortAlgorithm::Bubble && (i == self.i || i == self.i + 1)) || 
                                (self.algorithm == SortAlgorithm::Quick && i == self.pivot))) {
                    [255, 255, 0, 255] // Yellow for current indices
                } else {
                    [r, g, b, 255]
                };
                
                // Draw the bar
                for dy in 0..bar_height as usize {
                    for dx in 0..bar_width as usize {
                        let px = bar_x + dx;
                        let py = bar_y + dy;
                        
                        // Check bounds
                        if px < x + width && py < y + height {
                            let idx = 4 * (py * buffer_width as usize + (px + x_offset));
                            if idx + 3 < frame.len() {
                                frame[idx] = color[0];
                                frame[idx + 1] = color[1];
                                frame[idx + 2] = color[2];
                                frame[idx + 3] = color[3];
                            }
                        }
                    }
                }
            }
        } else {
            // Vertical visualization (top to bottom)
            let bar_height = height as f32 / n as f32;
            
            for i in 0..n {
                let bar_width = (self.array[i] as f32 / 255.0) * width as f32;
                let bar_x = x;
                let bar_y = y + (i as f32 * bar_height) as usize;
                
                // Highlight current indices being compared
                let color = if (self.state == SortState::Running && 
                               ((self.algorithm == SortAlgorithm::Bubble && (i == self.i || i == self.i + 1)) || 
                                (self.algorithm == SortAlgorithm::Quick && i == self.pivot))) {
                    [255, 255, 0, 255] // Yellow for current indices
                } else {
                    [r, g, b, 255]
                };
                
                // Draw the bar
                for dy in 0..bar_height as usize {
                    for dx in 0..bar_width as usize {
                        let px = bar_x + dx;
                        let py = bar_y + dy;
                        
                        // Check bounds
                        if px < x + width && py < y + height {
                            let idx = 4 * (py * buffer_width as usize + (px + x_offset));
                            if idx + 3 < frame.len() {
                                frame[idx] = color[0];
                                frame[idx + 1] = color[1];
                                frame[idx + 2] = color[2];
                                frame[idx + 3] = color[3];
                            }
                        }
                    }
                }
            }
        }
        
        // Draw algorithm name and stats
        let algo_name = match self.algorithm {
            SortAlgorithm::Bogo => "Bogo Sort",
            SortAlgorithm::Bubble => "Bubble Sort",
            SortAlgorithm::Quick => "Quick Sort",
        };
        
        let status = match self.state {
            SortState::Running => "Running",
            SortState::Completed => "Completed",
            SortState::Restarting => "Restarting",
        };
        
        // Draw algorithm name
        if horizontal {
            draw_text(frame, x as i32, (y - 20) as i32, algo_name, &[255, 255, 255], x_offset, buffer_width, 1.0);
            
            // Draw stats
            let stats = format!("Steps: {} | Comparisons: {} | Accesses: {} | {}", 
                              self.steps, self.comparisons, self.accesses, status);
            draw_text(frame, x as i32, (y - 10) as i32, &stats, &[200, 200, 200], x_offset, buffer_width, 0.8);
        } else {
            draw_text(frame, (x + width + 5) as i32, y as i32, algo_name, &[255, 255, 255], x_offset, buffer_width, 1.0);
            
            // Draw stats
            let stats = format!("Steps: {} | Comparisons: {} | Status: {}", 
                              self.steps, self.comparisons, status);
            draw_text(frame, (x + width + 5) as i32, (y + 10) as i32, &stats, &[200, 200, 200], x_offset, buffer_width, 0.8);
        }
    }
    
    fn get_sorted_percent(&self) -> f32 {
        let mut sorted_count = 0;
        
        for i in 1..self.array.len() {
            if self.array[i-1] <= self.array[i] {
                sorted_count += 1;
            }
        }
        
        sorted_count as f32 / (self.array.len() - 1) as f32
    }
    
    fn restart(&mut self) {
        self.state = SortState::Restarting;
    }
}

// Audio visualizer state
struct AudioVisualizer {
    spectrum: Vec<f32>,
    target_heights: Vec<f32>,
    current_heights: Vec<f32>,
    last_update: f32,
}

impl AudioVisualizer {
    fn new() -> Self {
        let mut spectrum = Vec::with_capacity(AUDIO_VIZ_BARS);
        let mut target_heights = Vec::with_capacity(AUDIO_VIZ_BARS);
        let mut current_heights = Vec::with_capacity(AUDIO_VIZ_BARS);
        
        for _ in 0..AUDIO_VIZ_BARS {
            spectrum.push(0.0);
            target_heights.push(0.0);
            current_heights.push(0.0);
        }
        
        Self {
            spectrum,
            target_heights,
            current_heights,
            last_update: 0.0,
        }
    }
    
    fn update(&mut self, time: f32) {
        let dt = if self.last_update > 0.0 {
            (time - self.last_update).min(0.1)
        } else {
            0.016
        };
        self.last_update = time;
        
        // Get the scaled height based on monitor dimensions
        let scaled_height = unsafe {
            match MONITOR_HEIGHT {
                Some(m_height) => {
                    // Scale the height based on screen height
                    let scale_factor = m_height as f32 / 1080.0; // 1080p reference
                    AUDIO_VIZ_BASE_HEIGHT * scale_factor
                },
                None => AUDIO_VIZ_BASE_HEIGHT // Default if no monitor dimensions
            }
        };
        
        // Check if we have audio spectrum data
        let mut use_audio_data = false;
        let mut audio_data = Vec::new();
        
        unsafe {
            if let Some(spectrum) = &AUDIO_SPECTRUM {
                if let Ok(data) = spectrum.lock() {
                    use_audio_data = true;
                    audio_data = data.clone();
                }
            }
        }
        
        for i in 0..AUDIO_VIZ_BARS {
            let target_height;
            
            if use_audio_data && i < audio_data.len() {
                // Use real audio data
                let audio_value = audio_data[i];
                target_height = AUDIO_VIZ_MIN_HEIGHT + audio_value * (scaled_height - AUDIO_VIZ_MIN_HEIGHT);
            } else {
                // Fallback to simulated data
                let time_phase = time * 0.5;
                let pos_factor = i as f32 / AUDIO_VIZ_BARS as f32;
                let freq_factor = (pos_factor * 10.0).sin() * 0.5 + 0.5;
                let time_factor = ((time_phase + pos_factor * 5.0).sin() * 0.5 + 0.5).powf(2.0);
                
                // Add some randomness for natural appearance
                let noise = rand::thread_rng().gen_range(0.0..0.2);
                
                // Calculate the target height for this bar
                let height_factor = time_factor * freq_factor + noise;
                target_height = AUDIO_VIZ_MIN_HEIGHT + height_factor * (scaled_height - AUDIO_VIZ_MIN_HEIGHT);
            }
            
            self.target_heights[i] = target_height;
            
            // Smoothly approach the target height
            let diff = self.target_heights[i] - self.current_heights[i];
            self.current_heights[i] += diff * (1.0 - (-dt * AUDIO_VIZ_DECAY_RATE).exp());
            
            // Store in spectrum for visualization (normalized)
            self.spectrum[i] = self.current_heights[i] / scaled_height;
        }
    }
    
    fn draw(&self, frame: &mut [u8], width: u32, height: u32, x_offset: usize, buffer_width: u32) {
        let bar_width = (width as f32 / AUDIO_VIZ_BARS as f32).floor() as usize;
        let margin = (width as usize - bar_width * AUDIO_VIZ_BARS) / 2;
        let y_baseline = height as usize - 10; // 10 pixels from bottom
        
        for i in 0..AUDIO_VIZ_BARS {
            // Convert to usize and ensure it doesn't exceed the baseline
            let bar_height = (self.current_heights[i] as usize).min(y_baseline);
            if bar_height == 0 {
                continue;
            }
            
            // Calculate color based on frequency (position) and height
            let hue = (i as f32 / AUDIO_VIZ_BARS as f32) * 270.0; // Blue to pink spectrum
            let saturation = 0.8;
            let value = 0.7 + 0.3 * (self.spectrum[i]);
            let color = hsv_to_rgb(hue, saturation, value);
            
            // Add alpha for neon effect
            let color_rgba = [color[0], color[1], color[2], 220];
            
            // Draw the bar
            let x_start = margin + i * bar_width;
            for y in 0..bar_height {
                for x in 0..bar_width {
                    let x_pos = x_start + x;
                    let y_pos = y_baseline - y;
                    
                    if x_pos < width as usize && y_pos < height as usize {
                        // Add glow effect - brighter in the middle of each bar
                        let glow_factor = 1.0 - (x as f32 / bar_width as f32 - 0.5).abs() * 2.0;
                        let glow_color = [
                            (color_rgba[0] as f32 * (0.7 + 0.3 * glow_factor)) as u8,
                            (color_rgba[1] as f32 * (0.7 + 0.3 * glow_factor)) as u8,
                            (color_rgba[2] as f32 * (0.7 + 0.3 * glow_factor)) as u8,
                            color_rgba[3]
                        ];
                        
                        put_pixel(frame, width, height, x_pos as i32, y_pos as i32, &glow_color, x_offset, buffer_width);
                    }
                }
            }
            
            // Draw the neon glow outline
            self.draw_glow(frame, width, height, x_start, y_baseline, bar_width, bar_height, &color, x_offset, buffer_width);
        }
    }
    
    fn draw_glow(&self, frame: &mut [u8], width: u32, height: u32, 
                x_start: usize, y_baseline: usize, bar_width: usize, bar_height: usize, 
                color: &[u8; 3], x_offset: usize, buffer_width: u32) {
        // Draw a subtle glow effect around each bar
        let glow_radius = 2;
        let glow_color = [color[0], color[1], color[2], 80]; // Semi-transparent
        
        for dy in -glow_radius..=glow_radius {
            for dx in -glow_radius..=glow_radius {
                // Skip the bar itself
                if dx == 0 && dy == 0 {
                    continue;
                }
                
                let distance_sq = dx * dx + dy * dy;
                if distance_sq <= glow_radius * glow_radius {
                    // Calculate alpha based on distance
                    let alpha = (1.0 - (distance_sq as f32 / (glow_radius * glow_radius) as f32).sqrt()) * 80.0;
                    let glow_alpha = [glow_color[0], glow_color[1], glow_color[2], alpha as u8];
                    
                    // Draw glow on top edge (carefully avoiding overflow)
                    if bar_height <= y_baseline {
                        let y_top = y_baseline - bar_height;
                        for x in 0..bar_width {
                            let x_pos = x_start + x;
                            let x_glow = (x_pos as i32 + dx).max(0).min(width as i32 - 1);
                            let y_glow = (y_top as i32 + dy).max(0).min(height as i32 - 1);
                            
                            put_pixel(frame, width, height, x_glow, y_glow, &glow_alpha, x_offset, buffer_width);
                        }
                    }
                    
                    // Draw glow on sides of the bar
                    for y in 0..bar_height {
                        // Left side glow
                        let x_glow_left = (x_start as i32 + dx).max(0).min(width as i32 - 1);
                        let y_glow = (y_baseline as i32 - y as i32 + dy).max(0).min(height as i32 - 1);
                        put_pixel(frame, width, height, x_glow_left, y_glow, &glow_alpha, x_offset, buffer_width);
                        
                        // Right side glow
                        let x_glow_right = (x_start as i32 + bar_width as i32 - 1 + dx).max(0).min(width as i32 - 1);
                        put_pixel(frame, width, height, x_glow_right, y_glow, &glow_alpha, x_offset, buffer_width);
                    }
                }
            }
        }
    }
}

// Static instance of audio visualizer
static mut AUDIO_VISUALIZER: Option<AudioVisualizer> = None;

// Audio spectrum data shared between audio thread and visualization
static mut AUDIO_SPECTRUM: Option<Arc<Mutex<Vec<f32>>>> = None;

// Static instances for sorting visualizers
static mut TOP_SORTER: Option<SortVisualizer> = None;
static mut BOTTOM_SORTER: Option<SortVisualizer> = None;
static mut LEFT_SORTER: Option<SortVisualizer> = None;
static mut RIGHT_SORTER: Option<SortVisualizer> = None;

// White noise generator for rodio
struct NoiseSource {
    sample_rate: u32,
    position: usize,
    amplitude: f32,
}

impl NoiseSource {
    fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            position: 0,
            amplitude: 0.25, // 25% volume to avoid being too loud
        }
    }
}

impl Iterator for NoiseSource {
    type Item = f32;
    
    fn next(&mut self) -> Option<f32> {
        self.position += 1;
        
        // Generate white noise
        let noise = rand::thread_rng().gen_range(-1.0..1.0) * self.amplitude;
        
        // Apply some simple filtering to make it more interesting
        // Pulsating effect based on position
        let pulse_freq = 0.5; // Hz
        let pulse = (self.position as f32 / self.sample_rate as f32 * pulse_freq * std::f32::consts::PI * 2.0).sin() * 0.5 + 0.5;
        
        // Low frequency oscillation for bass thumps
        let lfo_freq = 0.8; // Hz
        let lfo = (self.position as f32 / self.sample_rate as f32 * lfo_freq * std::f32::consts::PI * 2.0).sin() * 0.5 + 0.5;
        
        // Combine noise with pulse and lfo
        let sample = noise * (0.5 + pulse * 0.5) * (0.7 + lfo * 0.3);
        
        Some(sample)
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

// Start audio playback and analysis
fn start_audio_thread() -> Option<thread::JoinHandle<()>> {
    // Initialize the audio spectrum data
    let audio_spectrum = Arc::new(Mutex::new(vec![0.0; AUDIO_VIZ_BARS]));
    
    // Store it in the static variable
    unsafe {
        AUDIO_SPECTRUM = Some(audio_spectrum.clone());
    }
    
    // Create a thread for audio playback and analysis
    let handle = thread::spawn(move || {
        // Try to get the output stream
        let (_stream, stream_handle) = match OutputStream::try_default() {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Failed to get audio output stream: {}", e);
                return;
            }
        };
        
        // Create a sink to play our audio
        let sink = match Sink::try_new(&stream_handle) {
            Ok(sink) => sink,
            Err(e) => {
                eprintln!("Failed to create audio sink: {}", e);
                return;
            }
        };
        
        // Create our noise source
        let sample_rate = 44100;
        let noise = NoiseSource::new(sample_rate);
        
        // Set up a buffer to analyze
        let buffer_size = 1024;
        let mut audio_buffer = vec![0.0; buffer_size];
        let mut buffer_pos = 0;
        
        // Add the noise source to the sink
        sink.append(noise);
        
        // Keep the sink playing and analyze audio
        while !sink.empty() {
            // Sleep a bit to avoid hogging the CPU
            thread::sleep(Duration::from_millis(10));
            
            // Simulate audio capture and analysis
            for _ in 0..buffer_size/10 {  // Process some samples each time
                // Generate a new sample (similar to our noise source)
                let noise = rand::thread_rng().gen_range(-1.0..1.0) * 0.25;
                
                // Add to buffer
                audio_buffer[buffer_pos] = noise;
                buffer_pos = (buffer_pos + 1) % buffer_size;
                
                // Every time we fill the buffer, analyze it
                if buffer_pos == 0 {
                    analyze_audio(&audio_buffer, audio_spectrum.clone());
                }
            }
        }
    });
    
    Some(handle)
}

// Simple audio analysis function
fn analyze_audio(buffer: &[f32], spectrum: Arc<Mutex<Vec<f32>>>) {
    // Very simple "spectrum" analysis - we'll divide the buffer into frequency bands
    // In a real implementation, you'd use FFT for proper spectrum analysis
    
    let mut spectrum_data = spectrum.lock().unwrap();
    let num_bands = spectrum_data.len();
    
    // For each frequency band
    for i in 0..num_bands {
        // Calculate the range of samples for this band
        // (we're just dividing the buffer into equal parts)
        let start = (i * buffer.len()) / num_bands;
        let end = ((i + 1) * buffer.len()) / num_bands;
        
        // Calculate the energy (sum of squared amplitudes) in this band
        let mut energy = 0.0;
        for j in start..end {
            energy += buffer[j] * buffer[j];
        }
        
        // Normalize by the number of samples
        if end > start {
            energy /= (end - start) as f32;
        }
        
        // Apply some scaling to make it visually interesting
        let scaled_energy = energy.sqrt() * 4.0;
        
        // Add some randomness for more interesting visualization
        let noise = rand::thread_rng().gen_range(0.0..0.2);
        
        // Update the spectrum with smoothing
        spectrum_data[i] = spectrum_data[i] * 0.7 + (scaled_energy + noise) * 0.3;
    }
    
    // Apply some additional processing for visual interest
    
    // 1. Bass boost (lower frequencies)
    let bass_boost = 1.5;
    let bass_range = num_bands / 4;
    for i in 0..bass_range {
        let factor = 1.0 + bass_boost * (1.0 - i as f32 / bass_range as f32);
        spectrum_data[i] *= factor;
    }
    
    // 2. Ensure minimum and maximum levels for visual interest
    for value in spectrum_data.iter_mut() {
        *value = value.max(0.05).min(1.0);
    }
}

// HSV to RGB conversion helper for colorful visualization
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let h = h % 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    
    [
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8
    ]
}

// Sample text content for text fragments
static WIKIPEDIA_TEXT: &str = "Plug-in electric vehicle
Electric cars have lower operating and maintenance costs
All-electric vehicles have lower operating and maintenance costs
Compared to conventional internal combustion engine vehicles
Electric drive vehicles can contribute significantly
Less dependence on imported oil
Potential for zero emission transport
Electric motors are more efficient at converting energy
Better acceleration and torque characteristics
Norway has the highest market penetration per capita in the world
Electric vehicle battery technology continues to improve
Battery electric vehicles use chemical energy stored in battery packs
Plug-in hybrid electric vehicles are a good in-between option
Charging infrastructure continues to expand globally";

// Store for text fragments that will be displayed
struct TextFragment {
    text: String,
    x: i32,
    y: i32,
    color: [u8; 3],
    scale: f32,
    life: f32,
    max_life: f32,
}

// Static storage for text fragments
static mut TEXT_FRAGMENTS: Option<Vec<TextFragment>> = None;
static mut NEXT_FRAGMENT_TIME: Option<f32> = None;

// Draw the ray pattern visualization to the provided frame buffer
// Flag to track if audio thread has been started
static mut AUDIO_THREAD_STARTED: bool = false;

// Store information about the monitor
static mut MONITOR_WIDTH: Option<u32> = None;
static mut MONITOR_HEIGHT: Option<u32> = None;

// Function to get screen dimensions from monitor
pub fn set_monitor_dimensions(monitor: &MonitorHandle) {
    let size = monitor.size();
    unsafe {
        MONITOR_WIDTH = Some(size.width);
        MONITOR_HEIGHT = Some(size.height);
        println!("Monitor dimensions set: {}x{}", size.width, size.height);
    }
}

pub fn draw_frame(frame: &mut [u8], width: u32, height: u32, time: f32, x_offset: usize, buffer_width: u32) {
    // Get scale factors based on monitor dimensions
    let (scale_x, scale_y) = unsafe {
        match (MONITOR_WIDTH, MONITOR_HEIGHT) {
            (Some(m_width), Some(m_height)) => {
                // Scale based on monitor dimensions
                // Use a reasonable base size for reference
                let base_width = 1920.0;
                let base_height = 1080.0;
                (m_width as f32 / base_width, m_height as f32 / base_height)
            },
            _ => (1.0, 1.0) // Default scale if monitor dimensions aren't available
        }
    };
    // Initialize text fragments if needed
    unsafe {
        if TEXT_FRAGMENTS.is_none() {
            TEXT_FRAGMENTS = Some(Vec::with_capacity(20));
            NEXT_FRAGMENT_TIME = Some(time + 0.5); // Start adding fragments sooner
        }
        
        // Initialize audio visualizer if needed
        if AUDIO_VISUALIZER.is_none() {
            AUDIO_VISUALIZER = Some(AudioVisualizer::new());
        }
        
        // Start audio thread if it hasn't been started yet
        if !AUDIO_THREAD_STARTED {
            // Start the audio thread for noise generation and analysis
            if let Some(_handle) = start_audio_thread() {
                AUDIO_THREAD_STARTED = true;
                println!("Audio thread started successfully");
            }
        }
        
        // Initialize sorting visualizers if needed
        if TOP_SORTER.is_none() {
            TOP_SORTER = Some(SortVisualizer::new(SortAlgorithm::Bubble));
        }
        if BOTTOM_SORTER.is_none() {
            BOTTOM_SORTER = Some(SortVisualizer::new(SortAlgorithm::Quick));
        }
        if LEFT_SORTER.is_none() {
            LEFT_SORTER = Some(SortVisualizer::new(SortAlgorithm::Bogo));
        }
        if RIGHT_SORTER.is_none() {
            RIGHT_SORTER = Some(SortVisualizer::new(SortAlgorithm::Quick));
        }
        
        // Add new text fragments occasionally
        if let Some(next_time) = NEXT_FRAGMENT_TIME {
            if time >= next_time {
                // Add a new text fragment
                add_random_text_fragment(width, height, time);
                
                // Schedule next fragment - more frequent appearance (0.2-1.0 seconds)
                let delay = rand::thread_rng().gen_range(0.2..1.0);
                NEXT_FRAGMENT_TIME = Some(time + delay);
            }
        }
        
        // Initialize ball positions if they haven't been yet
        if YELLOW_POS.is_none() {
            let quarter_width = width as f32 / 4.0;
            let quarter_height = height as f32 / 4.0;
            
            // Scale ball velocities based on screen size
            let vel_scale = (scale_x + scale_y) / 2.0;
            let base_vel_x = 1.0 * vel_scale;
            let base_vel_y = 0.5 * vel_scale;
            
            // Position yellow ball in the upper-left quadrant
            YELLOW_POS = Some((quarter_width * 1.5, quarter_height * 1.5));
            YELLOW_VEL = Some((base_vel_x, base_vel_y));
            
            // Position green ball in the lower-right quadrant
            GREEN_POS = Some((width as f32 - quarter_width * 1.5, height as f32 - quarter_height * 1.5));
            GREEN_VEL = Some((-base_vel_x, -base_vel_y));
        }
        
        // Update ball positions
        let dt = if let Some(last) = LAST_TIME {
            let delta = time - last;
            // Limit delta time to avoid huge jumps if the app freezes
            if delta > 0.1 { 0.1 } else { delta }
        } else {
            0.016 // ~60 FPS if no previous time
        };
        
        LAST_TIME = Some(time);
        
        // Update yellow ball position
        if let (Some(pos), Some(vel)) = (YELLOW_POS.as_mut(), YELLOW_VEL.as_mut()) {
            // Scale movement speed based on screen size
            let speed_scale = (scale_x + scale_y) / 2.0;
            let base_speed = 50.0 * speed_scale;
            
            // Update position
            pos.0 += vel.0 * base_speed * dt;
            pos.1 += vel.1 * base_speed * dt;
            
            // Bounce off walls
            if pos.0 < 20.0 {
                pos.0 = 20.0;
                vel.0 = vel.0.abs();
            } else if pos.0 > width as f32 - 20.0 {
                pos.0 = width as f32 - 20.0;
                vel.0 = -vel.0.abs();
            }
            
            if pos.1 < 20.0 {
                pos.1 = 20.0;
                vel.1 = vel.1.abs();
            } else if pos.1 > height as f32 - 20.0 {
                pos.1 = height as f32 - 20.0;
                vel.1 = -vel.1.abs();
            }
        }
        
        // Update green ball position
        if let (Some(pos), Some(vel)) = (GREEN_POS.as_mut(), GREEN_VEL.as_mut()) {
            // Scale movement speed based on screen size
            let speed_scale = (scale_x + scale_y) / 2.0;
            let base_speed = 50.0 * speed_scale;
            
            // Update position
            pos.0 += vel.0 * base_speed * dt;
            pos.1 += vel.1 * base_speed * dt;
            
            // Bounce off walls
            if pos.0 < 20.0 {
                pos.0 = 20.0;
                vel.0 = vel.0.abs();
            } else if pos.0 > width as f32 - 20.0 {
                pos.0 = width as f32 - 20.0;
                vel.0 = -vel.0.abs();
            }
            
            if pos.1 < 20.0 {
                pos.1 = 20.0;
                vel.1 = vel.1.abs();
            } else if pos.1 > height as f32 - 20.0 {
                pos.1 = height as f32 - 20.0;
                vel.1 = -vel.1.abs();
            }
            
            // Ball collision detection with yellow ball
            if let Some(yellow_pos) = YELLOW_POS {
                let dx = pos.0 - yellow_pos.0;
                let dy = pos.1 - yellow_pos.1;
                let dist_sq = dx * dx + dy * dy;
                
                // Scale collision distance based on screen size
                let min_dist = 30.0 * ((scale_x + scale_y) / 2.0);
                
                if dist_sq < min_dist * min_dist {
                    // They're colliding, calculate new velocities
                    let dist = dist_sq.sqrt();
                    let nx = dx / dist;
                    let ny = dy / dist;
                    
                    // Move them apart to prevent sticking
                    pos.0 = yellow_pos.0 + nx * min_dist;
                    pos.1 = yellow_pos.1 + ny * min_dist;
                    
                    if let Some(yellow_vel) = YELLOW_VEL {
                        // Elastic collision
                        let dot_prod = vel.0 * nx + vel.1 * ny - yellow_vel.0 * nx - yellow_vel.1 * ny;
                        
                        // Update velocities
                        vel.0 -= dot_prod * nx;
                        vel.1 -= dot_prod * ny;
                        
                        if let Some(yellow_vel_mut) = YELLOW_VEL.as_mut() {
                            yellow_vel_mut.0 += dot_prod * nx;
                            yellow_vel_mut.1 += dot_prod * ny;
                        }
                    }
                }
            }
        }
    }
    
    // Clear frame with a dark background
    for pixel in frame.chunks_exact_mut(4) {
        pixel[0] = 5; // R
        pixel[1] = 5; // G
        pixel[2] = 10; // B
        pixel[3] = 255; // A
    }
    
    // Draw rays for yellow and green light sources
    unsafe {
        // Draw rays from both light sources
        if let Some(yellow_pos) = YELLOW_POS {
            let rays_yellow = [255, 255, 150, 255]; // Yellow color
            draw_rays(
                frame, width, height,
                yellow_pos.0 as i32, yellow_pos.1 as i32,
                width as i32 / 2, height as i32 / 2,
                width as i32 / 2 - 20, &rays_yellow,
                RAY_COUNT, time, x_offset, buffer_width
            );
            
            // Draw the yellow light source (scaled to screen size)
            let yellow_color = [255, 255, 0, 255];
            let ball_radius = (10.0 * scale_x.max(scale_y)) as i32;
            draw_filled_circle(
                frame, width, height,
                yellow_pos.0 as i32, yellow_pos.1 as i32,
                ball_radius, &yellow_color,
                x_offset, buffer_width
            );
            
            // Add glow around the yellow source (scaled to screen size)
            let glow_radius = (30.0 * scale_x.max(scale_y)) as i32;
            draw_shadow_glow(
                frame, width, height,
                yellow_pos.0 as i32, yellow_pos.1 as i32,
                glow_radius, &[255, 255, 100, 100],
                x_offset, buffer_width
            );
        }
        
        if let Some(green_pos) = GREEN_POS {
            let rays_green = [150, 255, 150, 255]; // Green color
            draw_rays(
                frame, width, height,
                green_pos.0 as i32, green_pos.1 as i32,
                width as i32 / 2, height as i32 / 2,
                width as i32 / 2 - 20, &rays_green,
                RAY_COUNT, time + 0.5, x_offset, buffer_width
            );
            
            // Draw the green light source (scaled to screen size)
            let green_color = [0, 255, 0, 255];
            let ball_radius = (10.0 * scale_x.max(scale_y)) as i32;
            draw_filled_circle(
                frame, width, height,
                green_pos.0 as i32, green_pos.1 as i32,
                ball_radius, &green_color,
                x_offset, buffer_width
            );
            
            // Add glow around the green source (scaled to screen size)
            let glow_radius = (30.0 * scale_x.max(scale_y)) as i32;
            draw_shadow_glow(
                frame, width, height,
                green_pos.0 as i32, green_pos.1 as i32,
                glow_radius, &[100, 255, 100, 100],
                x_offset, buffer_width
            );
        }
    }
    
    unsafe {
        // Update and draw the sorting visualizers along the edges of the screen
        // Scale the border thickness based on screen dimensions
        let scale_factor = (scale_x + scale_y) / 2.0;
        let border_thickness = (height as f32 * 0.05 * scale_factor) as usize; // 5% of height for top/bottom
        let side_width = (width as f32 * 0.05 * scale_factor) as usize; // 5% of width for sides
        
        // Update the sorters
        if let Some(sorter) = TOP_SORTER.as_mut() {
            sorter.update();
            
            // Restart if completed
            if sorter.state == SortState::Completed && (time * 10.0).floor() % 10.0 == 0.0 {
                sorter.restart();
            }
            
            // Draw top sorter
            sorter.draw(
                frame, 
                0, 0, // x, y position 
                width as usize, border_thickness, // width, height
                true, // horizontal
                x_offset, buffer_width
            );
        }
        
        if let Some(sorter) = BOTTOM_SORTER.as_mut() {
            sorter.update();
            
            // Restart if completed
            if sorter.state == SortState::Completed && (time * 10.0).floor() % 10.0 == 0.0 {
                sorter.restart();
            }
            
            // Draw bottom sorter
            sorter.draw(
                frame,
                0, height as usize - border_thickness, // x, y position
                width as usize, border_thickness, // width, height
                true, // horizontal
                x_offset, buffer_width
            );
        }
        
        if let Some(sorter) = LEFT_SORTER.as_mut() {
            sorter.update();
            
            // Restart if completed
            if sorter.state == SortState::Completed && (time * 10.0).floor() % 10.0 == 0.0 {
                sorter.restart();
            }
            
            // Draw left sorter
            sorter.draw(
                frame,
                0, border_thickness, // x, y position
                side_width, height as usize - border_thickness * 2, // width, height
                false, // vertical
                x_offset, buffer_width
            );
        }
        
        if let Some(sorter) = RIGHT_SORTER.as_mut() {
            sorter.update();
            
            // Restart if completed
            if sorter.state == SortState::Completed && (time * 10.0).floor() % 10.0 == 0.0 {
                sorter.restart();
            }
            
            // Draw right sorter
            sorter.draw(
                frame,
                width as usize - side_width, border_thickness, // x, y position
                side_width, height as usize - border_thickness * 2, // width, height
                false, // vertical
                x_offset, buffer_width
            );
        }
        
        // Update and draw the audio visualizer
        if let Some(audio_viz) = AUDIO_VISUALIZER.as_mut() {
            audio_viz.update(time);
            audio_viz.draw(frame, width, height, x_offset, buffer_width);
        }
        
        // Update and draw text fragments
        if let Some(fragments) = TEXT_FRAGMENTS.as_mut() {
            // Update each fragment's lifetime
            fragments.retain_mut(|fragment| {
                fragment.life -= 0.016; // Assume 60 FPS
                fragment.life > 0.0
            });
            
            // Draw all remaining fragments
            for fragment in fragments.iter() {
                draw_text_fragment(frame, width, height, fragment, x_offset, buffer_width);
            }
            
            // Limit the number of fragments based on time (increasing over time)
            let max_fragments = ((time * 0.1).min(5.0) + 10.0) as usize;
            if fragments.len() > max_fragments {
                fragments.sort_by(|a, b| a.life.partial_cmp(&b.life).unwrap());
                fragments.truncate(max_fragments);
            }
        }
    }
}

// Draw rays from a source point around a circle
fn draw_rays(frame: &mut [u8], width: u32, height: u32, 
            source_x: i32, source_y: i32, 
            center_x: i32, center_y: i32,
            radius: i32, color: &[u8; 4], 
            count: usize, time: f32, 
            x_offset: usize, buffer_width: u32) {
    
    // Get the current position of the other ball to check for shadows
    let (other_x, other_y, other_radius) = unsafe {
        if source_x == YELLOW_POS.unwrap_or((0.0, 0.0)).0 as i32 && 
           source_y == YELLOW_POS.unwrap_or((0.0, 0.0)).1 as i32 {
            // This is the yellow ball, so get green ball position
            (GREEN_POS.unwrap_or((0.0, 0.0)).0 as i32, 
             GREEN_POS.unwrap_or((0.0, 0.0)).1 as i32, 
             10) // Ball radius
        } else {
            // This is the green ball, so get yellow ball position
            (YELLOW_POS.unwrap_or((0.0, 0.0)).0 as i32, 
             YELLOW_POS.unwrap_or((0.0, 0.0)).1 as i32, 
             10) // Ball radius
        }
    };

    // Store shadow rays for drawing later
    let mut shadow_rays: Vec<((i32, i32), (i32, i32))> = Vec::new();
    
    for i in 0..count {
        // Calculate angle with small animation
        let base_angle = (i as f32 / count as f32) * 2.0 * std::f32::consts::PI;
        let angle = base_angle + (time * 0.2).sin() * 0.05; // Small oscillation
        
        // Calculate endpoint on circle
        let end_x = center_x as f32 + angle.cos() * radius as f32;
        let end_y = center_y as f32 + angle.sin() * radius as f32;
        
        // Check if ray intersects with the other ball
        // Using ray-circle intersection test
        let ray_dir_x = end_x as f32 - source_x as f32;
        let ray_dir_y = end_y as f32 - source_y as f32;
        let ray_length = (ray_dir_x * ray_dir_x + ray_dir_y * ray_dir_y).sqrt();
        
        // Normalize ray direction
        let ray_dir_x = ray_dir_x / ray_length;
        let ray_dir_y = ray_dir_y / ray_length;
        
        // Vector from ray origin to circle center
        let oc_x = source_x as f32 - other_x as f32;
        let oc_y = source_y as f32 - other_y as f32;
        
        // Quadratic formula components
        let a = 1.0; // Since ray direction is normalized
        let b = 2.0 * (ray_dir_x * oc_x + ray_dir_y * oc_y);
        let c = (oc_x * oc_x + oc_y * oc_y) - (other_radius * other_radius) as f32;
        
        let discriminant = b * b - 4.0 * a * c;
        
        if discriminant >= 0.0 {
            // Ray intersects the sphere
            let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
            let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
            
            // Check if intersection is along the ray (not behind it)
            if (t1 > 0.0 && t1 < ray_length) || (t2 > 0.0 && t2 < ray_length) {
                // Calculate intersection point (using the first intersection)
                let t = t1.max(0.0);
                let intersect_x = (source_x as f32 + ray_dir_x * t) as i32;
                let intersect_y = (source_y as f32 + ray_dir_y * t) as i32;
                
                // Draw ray only up to the intersection point
                draw_line(frame, width, height, source_x, source_y, intersect_x, intersect_y, 
                         color, x_offset, buffer_width);
                
                // Calculate shadow ray - extends from the intersection point to beyond the circle
                // Make the shadow ray extend to the edge of the main circle
                let shadow_length = radius as f32 * 1.2; // Longer than the circle radius
                let shadow_end_x = (intersect_x as f32 + ray_dir_x * shadow_length) as i32;
                let shadow_end_y = (intersect_y as f32 + ray_dir_y * shadow_length) as i32;
                
                // Store shadow ray for later drawing
                shadow_rays.push(((intersect_x, intersect_y), (shadow_end_x, shadow_end_y)));
            } else {
                // No intersection, draw full ray
                draw_line(frame, width, height, source_x, source_y, end_x as i32, end_y as i32, 
                         color, x_offset, buffer_width);
            }
        } else {
            // No intersection, draw full ray
            draw_line(frame, width, height, source_x, source_y, end_x as i32, end_y as i32, 
                     color, x_offset, buffer_width);
        }
    }
    
    // Draw shadow rays with faded color
    let shadow_color = [
        (color[0] as f32 * 0.2) as u8,
        (color[1] as f32 * 0.2) as u8,
        (color[2] as f32 * 0.2) as u8,
        128, // Semi-transparent
    ];
    
    for shadow in shadow_rays {
        draw_line(frame, width, height, 
                 shadow.0.0, shadow.0.1, 
                 shadow.1.0, shadow.1.1, 
                 &shadow_color, x_offset, buffer_width);
    }
}

// Draw a filled circle
fn draw_filled_circle(frame: &mut [u8], width: u32, height: u32, 
                     center_x: i32, center_y: i32, 
                     radius: i32, color: &[u8; 4],
                     x_offset: usize, buffer_width: u32) {
    
    for y in -radius..=radius {
        let h = (radius.pow(2) - y.pow(2)) as f32;
        if h < 0.0 { continue; }
        
        let w = h.sqrt() as i32;
        for x in -w..=w {
            put_pixel(frame, width, height, center_x + x, center_y + y, color, x_offset, buffer_width);
        }
    }
}

// Draw a line using Bresenham's algorithm
fn draw_line(frame: &mut [u8], width: u32, height: u32, 
            x0: i32, y0: i32, 
            x1: i32, y1: i32, 
            color: &[u8; 4],
            x_offset: usize, buffer_width: u32) {
    
    let mut x0 = x0;
    let mut y0 = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    
    loop {
        put_pixel(frame, width, height, x0, y0, color, x_offset, buffer_width);
        
        if x0 == x1 && y0 == y1 { break; }
        
        let e2 = 2 * err;
        if e2 >= dy {
            if x0 == x1 { break; }
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            if y0 == y1 { break; }
            err += dx;
            y0 += sy;
        }
    }
}

// Helper to put a pixel in the frame buffer
fn put_pixel(frame: &mut [u8], width: u32, height: u32, x: i32, y: i32, color: &[u8; 4], 
            x_offset: usize, buffer_width: u32) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }
    
    // Calculate the index in the buffer with the x_offset
    let x_pos = x as usize + x_offset;
    let idx = ((y as u32 * buffer_width + x_pos as u32) * 4) as usize;
    
    if idx + 3 < frame.len() {
        // Apply alpha blending
        frame[idx] = (frame[idx] as u16 * (255 - color[3]) as u16 / 255 + color[0] as u16 * color[3] as u16 / 255) as u8;
        frame[idx + 1] = (frame[idx + 1] as u16 * (255 - color[3]) as u16 / 255 + color[1] as u16 * color[3] as u16 / 255) as u8;
        frame[idx + 2] = (frame[idx + 2] as u16 * (255 - color[3]) as u16 / 255 + color[2] as u16 * color[3] as u16 / 255) as u8;
        frame[idx + 3] = 255;
    }
}

// Public functions to apply forces to the balls
pub fn apply_force_yellow(force_x: f32, force_y: f32) {
    unsafe {
        if let Some(vel) = YELLOW_VEL {
            let new_vel = (vel.0 + force_x, vel.1 + force_y);
            YELLOW_VEL = Some(new_vel);
        }
    }
}

pub fn apply_force_green(force_x: f32, force_y: f32) {
    unsafe {
        if let Some(vel) = GREEN_VEL {
            let new_vel = (vel.0 + force_x, vel.1 + force_y);
            GREEN_VEL = Some(new_vel);
        }
    }
}

pub fn teleport_yellow(x: f32, y: f32) {
    unsafe {
        YELLOW_POS = Some((x, y));
    }
}

pub fn teleport_green(x: f32, y: f32) {
    unsafe {
        GREEN_POS = Some((x, y));
    }
}

// Draw a shadow glow effect
fn draw_shadow_glow(frame: &mut [u8], width: u32, height: u32, 
                   center_x: i32, center_y: i32, 
                   radius: i32, color: &[u8; 4],
                   x_offset: usize, buffer_width: u32) {
    
    for y in -radius..=radius {
        for x in -radius..=radius {
            let dist_sq = x * x + y * y;
            if dist_sq <= radius * radius {
                // Calculate intensity based on distance from center
                let intensity = 1.0 - (dist_sq as f32 / (radius * radius) as f32).sqrt();
                
                // Apply fading effect
                let alpha = (intensity * color[3] as f32) as u8;
                
                // Only draw if visible
                if alpha > 5 {
                    let px = center_x + x;
                    let py = center_y + y;
                    
                    let shadow_color = [color[0], color[1], color[2], alpha];
                    put_pixel(frame, width, height, px, py, &shadow_color, x_offset, buffer_width);
                }
            }
        }
    }
}

// Function to draw text on the frame
fn draw_text(frame: &mut [u8], x: i32, y: i32, text: &str, color: &[u8; 3], 
            x_offset: usize, buffer_width: u32, scale: f32) {
    // Very simple 5x7 pixel font for ASCII characters
    let char_width = (6.0 * scale) as i32;
    let char_height = (8.0 * scale) as i32;
    
    for (i, c) in text.chars().enumerate() {
        let char_x = x + (i as i32 * char_width);
        
        // Skip non-printable or extended ASCII
        if c < ' ' || c > '~' {
            continue;
        }
        
        // Draw a scaled character
        for cy in 0..char_height {
            for cx in 0..char_width-1 {
                // Map scaled coordinates back to original font
                let font_x = (cx as f32 / scale) as i32;
                let font_y = (cy as f32 / scale) as i32;
                
                // Only draw character pixels, not the background
                if font_x < 6 && font_y < 8 && is_font_pixel_set(c, font_x, font_y) {
                    let px = char_x + cx;
                    let py = y + cy;
                    
                    if px >= 0 && px < buffer_width as i32 && py >= 0 && py < (buffer_width / 4) as i32 {
                        let idx = 4 * (py as usize * buffer_width as usize + (px as usize + x_offset));
                        if idx + 3 < frame.len() {
                            frame[idx] = color[0];
                            frame[idx + 1] = color[1];
                            frame[idx + 2] = color[2];
                            frame[idx + 3] = 255;
                        }
                    }
                }
            }
        }
    }
}

// Check if a pixel should be set for a given character and position
fn is_font_pixel_set(c: char, x: i32, y: i32) -> bool {
    // For simplicity, we'll just use a hardcoded implementation for a few characters
    // In a real implementation, you'd use a font bitmap or similar
    
    // Space character
    if c == ' ' {
        return false;
    }
    
    // Basic letters (simplistic representation)
    if c.is_alphabetic() {
        // Draw vertical line for most letters
        if x == 0 {
            return true;
        }
        
        // Draw horizontal line at top and bottom for many letters
        if (y == 0 || y == 6) && x > 0 && x < 4 {
            return true;
        }
        
        // Special case for some letters
        match c.to_ascii_lowercase() {
            'o' | 'c' | 'e' => return (x == 3 && y > 0 && y < 6) || (y == 0 || y == 6) && x > 0 && x < 3,
            's' => return (y == 0 || y == 3 || y == 6) && x > 0 && x < 4 || 
                          (y == 1 || y == 2) && x == 0 || 
                          (y == 4 || y == 5) && x == 3,
            'i' => return x == 2,
            't' => return x == 2 || (y == 0 && x > 0 && x < 4),
            _ => return (x == 0) || (x == 3 && y > 0 && y < 6) || (y == 0 || y == 6) && x > 0 && x < 3,
        }
    }
    
    // Numbers (very basic)
    if c.is_numeric() {
        // Draw box for numbers
        return x == 0 || x == 3 || y == 0 || y == 6;
    }
    
    // Simple symbols
    match c {
        '-' => return y == 3 && x > 0 && x < 4,
        '.' | ',' => return x == 1 && y >= 5,
        ':' => return x == 1 && (y == 2 || y == 5),
        '/' => return (x == 0 && y >= 4) || (x == 1 && y >= 2 && y < 4) || (x == 2 && y >= 0 && y < 2),
        _ => return x % 2 == 0 && y % 2 == 0, // Default pattern for other chars
    }
}

// Add a new random text fragment
fn add_random_text_fragment(width: u32, height: u32, time: f32) {
    unsafe {
        if let Some(ref mut fragments) = TEXT_FRAGMENTS {
            let mut rng = rand::thread_rng();
            
            // Pick a random segment from the Wikipedia text
            let wiki_text = WIKIPEDIA_TEXT;
            
            // Vary segment length more dramatically
            let segment_style = rng.gen_range(0..5);
            let segment_length = match segment_style {
                0 => rng.gen_range(3..10),      // Very short phrases
                1 => rng.gen_range(10..25),     // Short phrases
                2 => rng.gen_range(25..50),     // Medium phrases
                3 => rng.gen_range(50..100),    // Long sentences
                _ => rng.gen_range(5..30),      // Default range
            };
            
            if wiki_text.len() > segment_length {
                let start_pos = rng.gen_range(0..wiki_text.len() - segment_length);
                let end_pos = start_pos + segment_length;
                
                // Find word boundaries if possible
                let mut adjusted_start = start_pos;
                while adjusted_start > 0 && !wiki_text.as_bytes()[adjusted_start].is_ascii_whitespace() {
                    adjusted_start -= 1;
                }
                
                let mut adjusted_end = end_pos;
                while adjusted_end < wiki_text.len() && !wiki_text.as_bytes()[adjusted_end].is_ascii_whitespace() {
                    adjusted_end += 1;
                }
                
                // Extract text segment
                let segment = wiki_text[adjusted_start..adjusted_end].trim().to_string();
                
                if !segment.is_empty() {
                    // Positioning style
                    let position_style = rng.gen_range(0..5);
                    let (x, y) = match position_style {
                        0 => (rng.gen_range(0..width as i32), rng.gen_range(0..height as i32)), // Random
                        1 => (width as i32 / 2, rng.gen_range(0..height as i32)),  // Center horizontal
                        2 => (rng.gen_range(0..width as i32), height as i32 / 2),  // Center vertical
                        3 => (rng.gen_range(0..width as i32/4), rng.gen_range(0..height as i32)), // Left side
                        _ => (width as i32 - rng.gen_range(0..width as i32/4), rng.gen_range(0..height as i32)), // Right side
                    };
                    
                    // Choose color style
                    let color_style = rng.gen_range(0..5);
                    let color = match color_style {
                        0 => [255, 255, 255], // White
                        1 => [255, 255, 0],   // Yellow
                        2 => [0, 255, 0],     // Green
                        3 => [0, 255, 255],   // Cyan
                        _ => [               // Random light color
                            rng.gen_range(180..255),
                            rng.gen_range(180..255),
                            rng.gen_range(180..255),
                        ],
                    };
                    
                    // Vary scale more dramatically
                    let scale_style = rng.gen_range(0..5);
                    let scale = match scale_style {
                        0 => rng.gen_range(0.5..0.7),   // Small text
                        1 => rng.gen_range(0.7..0.9),   // Medium-small text
                        2 => rng.gen_range(0.9..1.1),   // Normal text
                        3 => rng.gen_range(1.1..1.5),   // Large text
                        _ => rng.gen_range(1.5..2.5),   // Very large text
                    };
                    
                    // Random lifetime (2-10 seconds)
                    let lifetime = rng.gen_range(2.0..10.0);
                    
                    fragments.push(TextFragment {
                        text: segment,
                        x,
                        y,
                        color,
                        scale,
                        life: lifetime,
                        max_life: lifetime,
                    });
                    
                    // Use time for dynamic fragment limits - more fragments during certain periods
                    let time_cycle = (time * 0.1).sin() * 0.5 + 0.5; // 0.0 to 1.0 cycle
                    let max_fragments = 10 + (time_cycle * 20.0) as usize; // 10 to 30 fragments
                    
                    // Limit the number of fragments to prevent overdrawing
                    if fragments.len() > max_fragments {
                        fragments.sort_by(|a, b| a.life.partial_cmp(&b.life).unwrap());
                        fragments.remove(0); // Remove the fragment with the shortest remaining life
                    }
                }
            }
        }
    }
}

// Draw a text fragment with fading effect
fn draw_text_fragment(frame: &mut [u8], _width: u32, _height: u32, fragment: &TextFragment, 
                     x_offset: usize, buffer_width: u32) {
    // Calculate opacity based on lifetime - fade in and out
    let fade_time = fragment.max_life * 0.2; // 20% of lifetime for fade in/out
    let opacity = if fragment.life > fragment.max_life - fade_time {
        // Fade in
        (fragment.max_life - fragment.life) / fade_time
    } else if fragment.life < fade_time {
        // Fade out
        fragment.life / fade_time
    } else {
        // Full opacity in the middle
        1.0
    };
    
    // Apply opacity to color
    let color = [
        (fragment.color[0] as f32 * opacity) as u8,
        (fragment.color[1] as f32 * opacity) as u8,
        (fragment.color[2] as f32 * opacity) as u8,
    ];
    
    // Draw the text
    draw_text(frame, fragment.x, fragment.y, &fragment.text, &color, x_offset, buffer_width, fragment.scale);
}

// Public function to restart all sorting visualizers
pub fn restart_sorters() {
    unsafe {
        if let Some(sorter) = TOP_SORTER.as_mut() {
            sorter.restart();
        }
        if let Some(sorter) = BOTTOM_SORTER.as_mut() {
            sorter.restart();
        }
        if let Some(sorter) = LEFT_SORTER.as_mut() {
            sorter.restart();
        }
        if let Some(sorter) = RIGHT_SORTER.as_mut() {
            sorter.restart();
        }
    }
} 