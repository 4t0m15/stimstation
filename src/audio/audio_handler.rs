use std::sync::{Arc, Mutex};
use rand::prelude::*;
pub const AUDIO_VIZ_BARS: usize = 32;
pub const AUDIO_VIZ_BASE_HEIGHT: f32 = 60.0;
pub const AUDIO_VIZ_MIN_HEIGHT: f32 = 5.0;
pub const AUDIO_VIZ_DECAY_RATE: f32 = 2.0;
static mut AUDIO_SPECTRUM: Option<Arc<Mutex<Vec<f32>>>> = None;
pub struct AudioVisualizer {
    spectrum: Vec<f32>,
    target_heights: Vec<f32>,
    current_heights: Vec<f32>,
    last_update: f32,
}
impl AudioVisualizer {
    pub fn new() -> Self {
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
    pub fn update(&mut self, time: f32, monitor_height: Option<u32>) {
        let dt = if self.last_update > 0.0 {
            (time - self.last_update).min(0.1)
        } else {
            0.016
        };
        self.last_update = time;
        let scaled_height = match monitor_height {
            Some(m_height) => {
                let scale_factor = m_height as f32 / 1080.0;
                AUDIO_VIZ_BASE_HEIGHT * scale_factor
            },
            None => AUDIO_VIZ_BASE_HEIGHT
        };
        let mut use_audio_data = false;
        let mut audio_data = Vec::new();
        unsafe {
            let spectrum_ptr: *const Option<Arc<Mutex<Vec<f32>>>> = &raw const AUDIO_SPECTRUM;
            if let Some(spectrum) = (*spectrum_ptr).as_ref() {
                if let Ok(data) = spectrum.lock() {
                    use_audio_data = true;
                    audio_data = data.clone();
                }
            }
        }
        for i in 0..AUDIO_VIZ_BARS {
            let target_height;
            if use_audio_data && i < audio_data.len() {
                let audio_value = audio_data[i];
                target_height = AUDIO_VIZ_MIN_HEIGHT + audio_value * (scaled_height - AUDIO_VIZ_MIN_HEIGHT);
            } else {
                let time_phase = time * 0.5;
                let pos_factor = i as f32 / AUDIO_VIZ_BARS as f32;
                let freq_factor = (pos_factor * 10.0).sin() * 0.5 + 0.5;
                let time_factor = ((time_phase + pos_factor * 5.0).sin() * 0.5 + 0.5).powf(2.0);
                let noise = rand::thread_rng().gen_range(0.0..0.2);
                let height_factor = time_factor * freq_factor + noise;
                target_height = AUDIO_VIZ_MIN_HEIGHT + height_factor * (scaled_height - AUDIO_VIZ_MIN_HEIGHT);
            }
            self.target_heights[i] = target_height;
            let diff = self.target_heights[i] - self.current_heights[i];
            self.current_heights[i] += diff * (1.0 - (-dt * AUDIO_VIZ_DECAY_RATE).exp());
            self.spectrum[i] = self.current_heights[i] / scaled_height;
        }
    }
    pub fn draw(&self, frame: &mut [u8], width: u32, height: u32, x_offset: usize, buffer_width: u32) {
        let bar_width = width as usize / AUDIO_VIZ_BARS;
        let y_baseline = height as usize - 50;
        let time = 0.1;
        for i in 0..AUDIO_VIZ_BARS {
            let bar_height = (self.current_heights[i] * (height as f32 / 200.0)).max(AUDIO_VIZ_MIN_HEIGHT) as usize;
            let x_start = i * bar_width;
            let mut rng = thread_rng();
            let noise = rng.gen_range(0.0..0.2);
            let hue = (i as f32 / AUDIO_VIZ_BARS as f32 + time * 0.1 + noise) % 1.0;
            let color = hsv_to_rgb(hue, 0.9, 1.0);
            self.draw_glow(frame, width, height, x_start, y_baseline, bar_width, bar_height, &color, x_offset, buffer_width);
        }
    }
    fn draw_glow(&self, frame: &mut [u8], width: u32, height: u32, 
                x_start: usize, y_baseline: usize, bar_width: usize, bar_height: usize, 
                color: &[u8; 3], x_offset: usize, buffer_width: u32) {
        let glow_radius = 2;
        let glow_color = [color[0], color[1], color[2], 80];
        for dy in -glow_radius..=glow_radius {
            for dx in -glow_radius..=glow_radius {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let distance_sq = dx * dx + dy * dy;
                if distance_sq <= glow_radius * glow_radius {
                    let alpha = (1.0 - (distance_sq as f32 / (glow_radius * glow_radius) as f32).sqrt()) * 80.0;
                    let glow_alpha = [glow_color[0], glow_color[1], glow_color[2], alpha as u8];
                    if bar_height <= y_baseline {
                        let y_top = y_baseline - bar_height;
                        for x in 0..bar_width {
                            let x_pos = x_start + x;
                            let x_glow = (x_pos as i32 + dx).max(0).min(width as i32 - 1);
                            let y_glow = (y_top as i32 + dy).max(0).min(height as i32 - 1);
                            put_pixel(frame, width, height, x_glow, y_glow, &glow_alpha, x_offset, buffer_width);
                        }
                    }
                    for y in (y_baseline - bar_height)..y_baseline {
                        let x_glow_left = (x_start as i32 + dx).max(0).min(width as i32 - 1);
                        let y_glow = (y_baseline as i32 - y as i32 + dy).max(0).min(height as i32 - 1);
                        put_pixel(frame, width, height, x_glow_left, y_glow, &glow_alpha, x_offset, buffer_width);
                        let x_glow_right = (x_start as i32 + bar_width as i32 - 1 + dx).max(0).min(width as i32 - 1);
                        put_pixel(frame, width, height, x_glow_right, y_glow, &glow_alpha, x_offset, buffer_width);
                    }
                }
            }
        }
    }
}
pub fn analyze_audio(buffer: &[f32], spectrum: Arc<Mutex<Vec<f32>>>) {
    let mut spectrum_data = spectrum.lock().unwrap();
    let num_bands = spectrum_data.len();
    for i in 0..num_bands {
        let start = (i * buffer.len()) / num_bands;
        let end = ((i + 1) * buffer.len()) / num_bands;
        let mut energy = 0.0;
        for j in start..end {
            energy += buffer[j] * buffer[j];
        }
        if end > start {
            energy /= (end - start) as f32;
        }
        let scaled_energy = energy.sqrt() * 4.0;
        let noise = rand::thread_rng().gen_range(0.0..0.2);
        spectrum_data[i] = spectrum_data[i] * 0.7 + (scaled_energy + noise) * 0.3;
    }
    let bass_boost = 1.5;
    let bass_range = num_bands / 4;
    for i in 0..bass_range {
        let factor = 1.0 + bass_boost * (1.0 - i as f32 / bass_range as f32);
        spectrum_data[i] *= factor;
    }
    for value in spectrum_data.iter_mut() {
        *value = value.max(0.05).min(1.0);
    }
}
#[allow(dead_code)]
pub fn get_audio_spectrum() -> Option<Arc<Mutex<Vec<f32>>>> {
    unsafe {
        let ptr: *const Option<Arc<Mutex<Vec<f32>>>> = &raw const AUDIO_SPECTRUM;
        (*ptr).clone()
    }
}
pub fn set_audio_spectrum(spectrum: Arc<Mutex<Vec<f32>>>) {
    unsafe {
        AUDIO_SPECTRUM = Some(spectrum);
    }
}
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let h = h % 1.0;
    let c = v * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = if h < 1.0/6.0 {
        (c, x, 0.0)
    } else if h < 2.0/6.0 {
        (x, c, 0.0)
    } else if h < 3.0/6.0 {
        (0.0, c, x)
    } else if h < 4.0/6.0 {
        (0.0, x, c)
    } else if h < 5.0/6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    [
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    ]
}
fn put_pixel(frame: &mut [u8], width: u32, height: u32, x: i32, y: i32, color: &[u8; 4], x_offset: usize, buffer_width: u32) {
    if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
        let actual_x = x as usize + x_offset;
        let actual_y = y as usize;
        if actual_x < buffer_width as usize && actual_y < height as usize {
            let idx = 4 * (actual_y * buffer_width as usize + actual_x);
            if idx + 3 < frame.len() {
                let alpha = color[3] as f32 / 255.0;
                let inv_alpha = 1.0 - alpha;
                frame[idx] = ((frame[idx] as f32 * inv_alpha + color[0] as f32 * alpha) as u8).min(255);
                frame[idx + 1] = ((frame[idx + 1] as f32 * inv_alpha + color[1] as f32 * alpha) as u8).min(255);
                frame[idx + 2] = ((frame[idx + 2] as f32 * inv_alpha + color[2] as f32 * alpha) as u8).min(255);
                frame[idx + 3] = 255;
            }
        }
    }
}
