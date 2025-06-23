// Audio integration module for ray pattern visualization
use std::sync::Arc;
use crate::io::audio::handler::{AudioHandler, AudioVisualizer};
use crate::io::audio::playback::{AudioBackend, AudioPlaybackManager};

pub struct AudioIntegration {
    visualizer: Option<AudioVisualizer>,
    audio_handler: Arc<AudioHandler>,
    audio_playback: AudioPlaybackManager,
}

impl AudioIntegration {
    pub fn new() -> Self {
        // Create audio handler
        let audio_handler = Arc::new(AudioHandler::new());
        
        // Create playback manager
        let audio_playback = AudioPlaybackManager::new(audio_handler.get_spectrum());
        
        Self {
            visualizer: None,
            audio_handler,
            audio_playback,
        }
    }

    pub fn initialize(&mut self) -> Result<(), String> {
        // Initialize audio visualizer if needed
        if self.visualizer.is_none() {
            self.visualizer = Some(AudioVisualizer::new(self.audio_handler.clone()));
        }
        
        // Start audio playback if not already running
        if !self.audio_playback.is_running() {
            self.audio_playback.start()?;
        }
        
        Ok(())
    }

    pub fn update(&mut self, time: f32, monitor_height: Option<u32>) {
        if let Some(audio_viz) = self.visualizer.as_mut() {
            audio_viz.update(time, monitor_height);
        }
    }

    pub fn draw(&self, frame: &mut [u8], width: u32, height: u32, x_offset: usize, buffer_width: u32) {
        if let Some(audio_viz) = self.visualizer.as_ref() {
            audio_viz.draw(frame, width, height, x_offset, buffer_width);
        }
    }
    
    pub fn shutdown(&mut self) {
        self.audio_playback.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_integration() {
        let integration = AudioIntegration::new();
        
        // Check initial state
        assert!(integration.visualizer.is_none());
    }
}
