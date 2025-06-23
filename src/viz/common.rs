// Common visualization utilities and traits
use crate::render::drawing::FrameBuffer;

/// Trait for visualizations
pub trait Visualization {
    /// Initialize the visualization
    fn init(&mut self);
    
    /// Update the visualization state
    fn update(&mut self, time: f32);
    
    /// Render the visualization to a frame buffer
    fn render(&mut self, time: f32) -> FrameBuffer;
    
    /// Handle input (optional)
    fn handle_input(&mut self, _x: f32, _y: f32, _clicked: bool) {}
    
    /// Name of the visualization
    fn name(&self) -> &'static str;
}

/// Base for visualizations
pub struct VisualizationBase {
    pub width: u32,
    pub height: u32,
    pub name: &'static str,
}

impl VisualizationBase {
    pub fn new(width: u32, height: u32, name: &'static str) -> Self {
        Self {
            width,
            height,
            name,
        }
    }
    
    pub fn create_frame_buffer(&self) -> FrameBuffer {
        FrameBuffer::new(self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestViz {
        base: VisualizationBase,
    }
    
    impl TestViz {
        fn new() -> Self {
            Self {
                base: VisualizationBase::new(100, 100, "TestViz"),
            }
        }
    }
    
    impl Visualization for TestViz {
        fn init(&mut self) {}
        
        fn update(&mut self, _time: f32) {}
        
        fn render(&mut self, _time: f32) -> FrameBuffer {
            self.base.create_frame_buffer()
        }
        
        fn name(&self) -> &'static str {
            self.base.name
        }
    }
    
    #[test]
    fn test_visualization_trait() {
        let mut viz = TestViz::new();
        
        // Check name
        assert_eq!(viz.name(), "TestViz");
        
        // Test creation of frame buffer
        let fb = viz.render(0.0);
        assert_eq!(fb.width(), 100);
        assert_eq!(fb.height(), 100);
    }
}
