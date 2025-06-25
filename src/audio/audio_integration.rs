use crate::audio::audio_handler::AudioVisualizer;
use crate::audio::audio_playback::{is_audio_thread_started, start_audio_thread};
pub struct AudioIntegration {
    visualizer: Option<AudioVisualizer>,
}
impl AudioIntegration {
    pub fn new() -> Self {
        Self { visualizer: None }
    }
    pub fn initialize(&mut self) {
        if self.visualizer.is_none() {
            self.visualizer = Some(AudioVisualizer::new());
        }
        if !is_audio_thread_started() {
            if let Some(_handle) = start_audio_thread() {
                println!("Audio thread started successfully");
            }
        }
    }
    pub fn update(&mut self, time: f32, monitor_height: Option<u32>) {
        if let Some(audio_viz) = self.visualizer.as_mut() {
            audio_viz.update(time, monitor_height);
        }
    }
    pub fn draw(
        &mut self,
        frame: &mut [u8],
        width: u32,
        height: u32,
        x_offset: usize,
        buffer_width: u32,
    ) {
        if let Some(audio_viz) = self.visualizer.as_mut() {
            audio_viz.draw(frame, width, height, x_offset, buffer_width);
        }
    }
}
