// Pythagoras theorem visualization
use crate::render::drawing::{FrameBuffer, draw_triangle_filled, draw_text_fast};
use crate::types::Color;
use crate::viz::common::{Visualization, VisualizationBase};

/// Pythagoras visualization 
pub struct PythagorasViz {
    base: VisualizationBase,
}

impl PythagorasViz {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            base: VisualizationBase::new(width, height, "Pythagoras Theorem"),
        }
    }
}

impl Visualization for PythagorasViz {
    fn init(&mut self) {
        // No specific initialization needed
    }
    
    fn update(&mut self, _time: f32) {
        // No state to update
    }
    
    fn render(&mut self, elapsed: f32) -> FrameBuffer {
        let mut fb = self.base.create_frame_buffer();
        
        // Clear the frame with a white background
        fb.clear(Color::new(255, 255, 255));
        
        // Parameters
        let a = 100.0f32;
        let b = 150.0f32;
        let c = (a*a + b*b).sqrt();
        let center_x = self.base.width as f32 / 2.0;
        let center_y = self.base.height as f32 / 2.0;
        let angle = elapsed * 0.5; // Rotation angle
        
        // Draw big square (c × c) - light gray
        let square_color = Color::new(200, 200, 200);
        let half_c = (c / 2.0) as i32;
        
        for y in -half_c..half_c {
            for x in -half_c..half_c {
                fb.set_pixel(
                    center_x as i32 + x,
                    center_y as i32 + y,
                    square_color
                );
            }
        }
        
        // Draw four triangles (blue)
        let triangle_color = Color::new(0, 0, 255);
        
        for i in 0..4 {
            let theta = angle + i as f32 * std::f32::consts::FRAC_PI_2;
            
            // Triangle vertices using glam::Vec2
            let p1 = glam::Vec2::new(
                center_x + theta.cos() * (c / 2.0),
                center_y + theta.sin() * (c / 2.0)
            );
            
            let p2 = glam::Vec2::new(
                center_x + (theta + (b as f32).to_radians()).cos() * (a / 2.0),
                center_y + (theta + b.to_radians()).sin() * (a / 2.0)
            );
            
            let p3 = glam::Vec2::new(
                center_x + (theta - (a as f32).to_radians()).cos() * (b / 2.0),
                center_y + (theta - a.to_radians()).sin() * (b / 2.0)
            );
            
            // Draw filled triangle
            draw_triangle_filled(
                &mut fb,
                p1, p2, p3,
                triangle_color
            );
        }
        
        // Draw explanatory text
        let text_color = Color::new(0, 0, 0);
        draw_text_fast(&mut fb, "Pythagoras Theorem: a² + b² = c²", 
                     20, 30, 
                     text_color, 1.0);
        
        let a_squared = (a * a).round() as i32;
        let b_squared = (b * b).round() as i32;
        let c_squared = (c * c).round() as i32;
        
        draw_text_fast(&mut fb, &format!("{} + {} = {}", a_squared, b_squared, c_squared), 
                     20, 50, 
                     text_color, 1.0);
                     
        fb
    }
    
    fn name(&self) -> &'static str {
        self.base.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pythagoras_creation() {
        let viz = PythagorasViz::new(800, 600);
        assert_eq!(viz.name(), "Pythagoras Theorem");
    }
    
    #[test]
    fn test_pythagoras_render() {
        let mut viz = PythagorasViz::new(800, 600);
        let fb = viz.render(0.0);
        
        // Check buffer dimensions
        assert_eq!(fb.width(), 800);
        assert_eq!(fb.height(), 600);
        
        // The first pixel should be white (background color)
        let buffer = fb.buffer();
        assert_eq!(buffer[0..4], [255, 255, 255, 255]);
    }
}
