// Audio analysis and spectrum generation
use std::sync::{Arc, Mutex};
use rand::prelude::*;

// Audio visualizer settings
pub const AUDIO_VIZ_BARS: usize = 32;
pub const AUDIO_VIZ_BASE_HEIGHT: f32 = 60.0;  // Base height for 1080p screen
pub const AUDIO_VIZ_MIN_HEIGHT: f32 = 5.0;    // Minimum height for bars
pub const AUDIO_VIZ_DECAY_RATE: f32 = 2.0;    // How quickly the bars react to changes

/// Audio spectrum analyzer
pub struct AudioHandler {
    audio_spectrum: Arc<Mutex<Vec<f32>>>,
}

impl AudioHandler {
    /// Create a new audio handler
    pub fn new() -> Self {
        let audio_spectrum = Arc::new(Mutex::new(vec![0.0; AUDIO_VIZ_BARS]));
        Self { audio_spectrum }
    }
    
    /// Get shared reference to audio spectrum data
    pub fn get_spectrum(&self) -> Arc<Mutex<Vec<f32>>> {
        self.audio_spectrum.clone()
    }
    
    /// Analyze audio buffer and update spectrum data
    pub fn analyze_audio(&self, buffer: &[f32]) {
        if let Ok(mut spectrum) = self.audio_spectrum.lock() {
            let num_bars = spectrum.len();
            
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
                spectrum[i] = (spectrum[i] * 0.7 + avg * 3.0 * 0.3).min(1.0);
            }
        }
    }
    
    /// Get spectrum data as a vector
    pub fn get_spectrum_data(&self) -> Vec<f32> {
        if let Ok(spectrum) = self.audio_spectrum.lock() {
            spectrum.clone()
        } else {
            vec![0.0; AUDIO_VIZ_BARS]
        }
    }
}

/// Audio visualizer for rendering audio spectrum
pub struct AudioVisualizer {
    spectrum: Vec<f32>,
    target_heights: Vec<f32>,
    current_heights: Vec<f32>,
    last_update: f32,
    audio_handler: Arc<AudioHandler>,
}

impl AudioVisualizer {
    pub fn new(audio_handler: Arc<AudioHandler>) -> Self {
        let spectrum = vec![0.0; AUDIO_VIZ_BARS];
        let target_heights = vec![0.0; AUDIO_VIZ_BARS];
        let current_heights = vec![0.0; AUDIO_VIZ_BARS];
        
        Self {
            spectrum,
            target_heights,
            current_heights,
            last_update: 0.0,
            audio_handler,
        }
    }
    
    pub fn update(&mut self, time: f32, monitor_height: Option<u32>) {
        let dt = if self.last_update > 0.0 {
            (time - self.last_update).min(0.1)
        } else {
            0.016
        };
        self.last_update = time;
        
        // Get the scaled height based on monitor dimensions
        let scaled_height = match monitor_height {
            Some(m_height) => {
                // Scale the height based on screen height
                let scale_factor = m_height as f32 / 1080.0; // 1080p reference
                AUDIO_VIZ_BASE_HEIGHT * scale_factor
            },
            None => AUDIO_VIZ_BASE_HEIGHT // Default if no monitor dimensions
        };
        
        // Get spectrum data
        let audio_data = self.audio_handler.get_spectrum_data();
        
        for i in 0..AUDIO_VIZ_BARS {
            let target_height;
            
            if i < audio_data.len() {
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
            self.current_heights[i] += diff * dt * AUDIO_VIZ_DECAY_RATE;
            
            // Store spectrum value (normalized 0-1)
            self.spectrum[i] = (self.current_heights[i] - AUDIO_VIZ_MIN_HEIGHT) / 
                                (scaled_height - AUDIO_VIZ_MIN_HEIGHT);
        }
    }
    
    pub fn draw(&self, frame: &mut [u8], width: u32, height: u32, x_offset: usize, buffer_width: u32) {
        let bar_width = (width as usize / AUDIO_VIZ_BARS).max(1);
        let bar_spacing = 1; // Space between bars
        
        // Draw each bar
        for i in 0..AUDIO_VIZ_BARS {
            let bar_height = (self.current_heights[i] * height as f32 / 4.0) as usize;
            let bar_x = x_offset + i * (bar_width + bar_spacing);
            
            // Calculate a color based on height
            let hue = i as f32 / AUDIO_VIZ_BARS as f32;
            let r = ((1.0 - hue) * 255.0) as u8;
            let g = ((0.5 + hue * 0.5) * 255.0) as u8;
            let b = ((hue * 0.8) * 255.0) as u8;
            
            // Draw the bar
            for x in 0..bar_width {
                if bar_x + x >= buffer_width as usize {
                    break;
                }
                
                for y in 0..bar_height {
                    if height as usize - y < height as usize {
                        let pixel_index = ((height as usize - y - 1) * buffer_width as usize + bar_x + x) * 4;
                        
                        // Only draw if in bounds
                        if pixel_index + 3 < frame.len() {
                            frame[pixel_index] = r;
                            frame[pixel_index + 1] = g;
                            frame[pixel_index + 2] = b;
                            frame[pixel_index + 3] = 255;
                        }
                    }
                }
            }
        }
    }
    
    pub fn get_spectrum(&self) -> &[f32] {
        &self.spectrum
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audio_handler_new() {
        let handler = AudioHandler::new();
        let spectrum = handler.get_spectrum_data();
        
        // Check initial state
        assert_eq!(spectrum.len(), AUDIO_VIZ_BARS);
        assert!(spectrum.iter().all(|&v| v == 0.0));
    }
    
    #[test]
    fn test_analyze_audio() {
        let handler = AudioHandler::new();
        
        // Create a test buffer with alternating values
        let mut buffer = vec![0.0; 1024];
        for i in 0..buffer.len() {
            buffer[i] = if i % 2 == 0 { 0.5 } else { -0.5 };
        }
        
        // Analyze the buffer
        handler.analyze_audio(&buffer);
        
        // Get spectrum data after analysis
        let spectrum = handler.get_spectrum_data();
        
        // Check that values were updated (should be non-zero)
        assert!(spectrum.iter().any(|&v| v > 0.0));
    }
}
