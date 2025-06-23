// Configuration structs for the application
use crate::types::{WIDTH, HEIGHT};

/// Configuration for visualizations
pub struct VisualizationConfig {
    /// Width of the visualization area
    pub width: u32,
    /// Height of the visualization area
    pub height: u32,
    /// Enable audio visualization
    pub enable_audio: bool,
    /// Enable particle effects
    pub enable_particles: bool,
    /// Maximum FPS
    pub max_fps: Option<u32>,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            width: WIDTH,
            height: HEIGHT,
            enable_audio: true,
            enable_particles: true,
            max_fps: None,
        }
    }
}

/// Configuration for audio
pub struct AudioConfig {
    /// Sample rate for audio playback
    pub sample_rate: u32,
    /// Audio buffer size
    pub buffer_size: usize,
    /// Audio amplitude
    pub amplitude: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            buffer_size: 1024,
            amplitude: 0.15,
        }
    }
}

/// Application configuration
pub struct AppConfig {
    /// Visualization configuration
    pub visualization: VisualizationConfig,
    /// Audio configuration
    pub audio: AudioConfig,
    /// Window title
    pub window_title: String,
    /// Start in fullscreen mode
    pub start_fullscreen: bool,
    /// Show FPS counter
    pub show_fps: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            visualization: VisualizationConfig::default(),
            audio: AudioConfig::default(),
            window_title: "StimStation".to_string(),
            start_fullscreen: false,
            show_fps: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_configs() {
        let app_config = AppConfig::default();
        
        // Test visualization config defaults
        assert_eq!(app_config.visualization.width, WIDTH);
        assert_eq!(app_config.visualization.height, HEIGHT);
        assert_eq!(app_config.visualization.enable_audio, true);
        
        // Test audio config defaults
        assert_eq!(app_config.audio.sample_rate, 44100);
        assert_eq!(app_config.audio.buffer_size, 1024);
        
        // Test app config defaults
        assert_eq!(app_config.window_title, "StimStation");
        assert_eq!(app_config.start_fullscreen, false);
    }
}
