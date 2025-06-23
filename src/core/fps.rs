// FPS counter isolated into its own module
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// FPS counter with optimized storage
#[derive(Debug)]
pub struct FpsCounter {
    pub frame_times: VecDeque<Instant>,
    pub last_update: Instant,
    pub current_fps: f32,
    pub update_interval: Duration,
}

impl FpsCounter {
    /// Create a new FPS counter
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(100),
            last_update: Instant::now(),
            current_fps: 0.0,
            update_interval: Duration::from_millis(500),
        }
    }
    
    /// Update the FPS counter with a new frame
    pub fn update(&mut self) -> f32 {
        let now = Instant::now();
        
        // Add current frame time
        self.frame_times.push_back(now);
        
        // Remove frames older than 1 second
        while let Some(time) = self.frame_times.front() {
            if now.duration_since(*time).as_secs_f32() > 1.0 {
                self.frame_times.pop_front();
            } else {
                break;
            }
        }
        
        // Update FPS calculation every update interval
        if now.duration_since(self.last_update) >= self.update_interval {
            let frame_count = self.frame_times.len() as f32;
            if frame_count > 0.0 {
                self.current_fps = frame_count;
            }
            self.last_update = now;
        }
        
        self.current_fps
    }
    
    /// Get the current FPS
    pub fn fps(&self) -> f32 {
        self.current_fps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time::Duration};
    
    #[test]
    fn test_new_fps_counter() {
        let fps = FpsCounter::new();
        assert_eq!(fps.current_fps, 0.0);
        assert!(fps.frame_times.is_empty());
    }
    
    #[test]
    fn test_fps_updates_properly() {
        let mut fps = FpsCounter::new();
        
        // Simulate a few frames
        for _ in 0..10 {
            fps.update();
            thread::sleep(Duration::from_millis(10));
        }
        
        // FPS should be non-zero after updates
        assert!(fps.fps() > 0.0);
    }
}
