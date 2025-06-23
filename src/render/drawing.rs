// Rendering utilities
use crate::types::{Color, Position, WIDTH, HEIGHT};

/// Frame buffer for optimized drawing
pub struct FrameBuffer {
    buffer: Vec<u8>,
    width: u32,
    height: u32,
}

impl FrameBuffer {
    /// Create a new frame buffer
    pub fn new(width: u32, height: u32) -> Self {
        let buffer = vec![0; (width * height * 4) as usize];
        Self { buffer, width, height }
    }
    
    /// Get the underlying buffer
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }
    
    /// Get the underlying buffer mutably
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
    
    /// Get the width of the frame buffer
    pub fn width(&self) -> u32 {
        self.width
    }
    
    /// Get the height of the frame buffer
    pub fn height(&self) -> u32 {
        self.height
    }
    
    /// Clear the frame buffer with a specific color
    pub fn clear(&mut self, color: Color) {
        for pixel in self.buffer.chunks_mut(4) {
            pixel[0] = color.red;
            pixel[1] = color.green;
            pixel[2] = color.blue;
            pixel[3] = 255;
        }
    }
    
    /// Set a pixel at the specified coordinates
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let index = ((y as u32 * self.width + x as u32) * 4) as usize;
            if index + 3 < self.buffer.len() {
                self.buffer[index] = color.red;
                self.buffer[index + 1] = color.green;
                self.buffer[index + 2] = color.blue;
                self.buffer[index + 3] = 255;
            }
        }
    }
    
    /// Copy to a target frame buffer
    pub fn copy_to(&self, target: &mut [u8]) {
        if target.len() == self.buffer.len() {
            target.copy_from_slice(&self.buffer);
        } else {
            // Handle different sizes if needed
            let target_width = WIDTH;
            let target_height = HEIGHT;
            
            let min_width = self.width.min(target_width);
            let min_height = self.height.min(target_height);
            
            for y in 0..min_height {
                for x in 0..min_width {
                    let src_idx = ((y * self.width + x) * 4) as usize;
                    let dst_idx = ((y * target_width + x) * 4) as usize;
                    
                    if src_idx + 3 < self.buffer.len() && dst_idx + 3 < target.len() {
                        target[dst_idx..dst_idx + 4].copy_from_slice(&self.buffer[src_idx..src_idx + 4]);
                    }
                }
            }
        }
    }
}

/// Draw a line
pub fn draw_line(fb: &mut FrameBuffer, x0: i32, y0: i32, x1: i32, y1: i32, color: Color, thickness: f32) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    
    let mut x = x0;
    let mut y = y0;
    
    let thickness_half = thickness / 2.0;
    let thickness_squared = thickness_half * thickness_half;
    
    while x != x1 + sx || y != y1 + sy {
        // Draw thick line
        for py in (y - thickness as i32)..=(y + thickness as i32) {
            for px in (x - thickness as i32)..=(x + thickness as i32) {
                let dist_squared = (px - x) as f32 * (px - x) as f32 + (py - y) as f32 * (py - y) as f32;
                if dist_squared <= thickness_squared {
                    fb.set_pixel(px, py, color);
                }
            }
        }
        
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

/// Draw a circle
pub fn draw_circle(fb: &mut FrameBuffer, x0: i32, y0: i32, radius: i32, color: Color, fill: bool) {
    let mut x = radius;
    let mut y = 0;
    let mut err = 0;
    
    while x >= y {
        if fill {
            // Fill the circle by drawing horizontal lines
            for i in -x..=x {
                fb.set_pixel(x0 + i, y0 + y, color);
                fb.set_pixel(x0 + i, y0 - y, color);
            }
            for i in -y..=y {
                fb.set_pixel(x0 + i, y0 + x, color);
                fb.set_pixel(x0 + i, y0 - x, color);
            }
        } else {
            // Draw the outline
            fb.set_pixel(x0 + x, y0 + y, color);
            fb.set_pixel(x0 + y, y0 + x, color);
            fb.set_pixel(x0 - y, y0 + x, color);
            fb.set_pixel(x0 - x, y0 + y, color);
            fb.set_pixel(x0 - x, y0 - y, color);
            fb.set_pixel(x0 - y, y0 - x, color);
            fb.set_pixel(x0 + y, y0 - x, color);
            fb.set_pixel(x0 + x, y0 - y, color);
        }
        
        if err <= 0 {
            y += 1;
            err += 2 * y + 1;
        }
        if err > 0 {
            x -= 1;
            err -= 2 * x + 1;
        }
    }
}

/// Draw a fast circle
pub fn draw_circle_fast(fb: &mut FrameBuffer, center: Position, radius: f32, color: Color) {
    let x0 = center.x as i32;
    let y0 = center.y as i32;
    let r = radius as i32;
    
    for y in (y0-r).max(0)..(y0+r+1).min(fb.height() as i32) {
        for x in (x0-r).max(0)..(x0+r+1).min(fb.width() as i32) {
            let dist_squared = ((x - x0) * (x - x0) + (y - y0) * (y - y0)) as f32;
            if dist_squared <= radius * radius {
                fb.set_pixel(x, y, color);
            }
        }
    }
}

/// Draw a triangle
pub fn draw_triangle_filled(fb: &mut FrameBuffer, p1: Position, p2: Position, p3: Position, color: Color) {
    let min_x = p1.x.min(p2.x.min(p3.x)).max(0.0) as i32;
    let max_x = p1.x.max(p2.x.max(p3.x)).min(fb.width() as f32) as i32;
    let min_y = p1.y.min(p2.y.min(p3.y)).max(0.0) as i32;
    let max_y = p1.y.max(p2.y.max(p3.y)).min(fb.height() as f32) as i32;
    
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if point_in_triangle(x as f32, y as f32, p1, p2, p3) {
                fb.set_pixel(x, y, color);
            }
        }
    }
}

/// Check if a point is inside a triangle
fn point_in_triangle(x: f32, y: f32, p1: Position, p2: Position, p3: Position) -> bool {
    let p = Position::new(x, y);
    let area = triangle_area(p1, p2, p3);
    let s1 = triangle_area(p, p2, p3) / area;
    let s2 = triangle_area(p1, p, p3) / area;
    let s3 = triangle_area(p1, p2, p) / area;
    
    s1 >= 0.0 && s1 <= 1.0 && s2 >= 0.0 && s2 <= 1.0 && s3 >= 0.0 && s3 <= 1.0 && (s1 + s2 + s3 - 1.0).abs() < 0.001
}

/// Calculate the area of a triangle
fn triangle_area(p1: Position, p2: Position, p3: Position) -> f32 {
    0.5 * ((p2.x - p1.x) * (p3.y - p1.y) - (p3.x - p1.x) * (p2.y - p1.y)).abs()
}

/// Draw text (simplified version)
pub fn draw_text_fast(fb: &mut FrameBuffer, text: &str, x: i32, y: i32, color: Color, scale: f32) {
    // This is a simple placeholder for text rendering
    // In a real implementation, you'd use font_kit or another text rendering library
    let font_size = 8.0 * scale;
    let mut curr_x = x;
    
    for c in text.chars() {
        // Draw a simple rectangle for each character
        if c != ' ' {
            let x0 = curr_x;
            let y0 = y;
            let x1 = curr_x + (font_size as i32);
            let y1 = y + (font_size as i32);
            
            for py in y0..y1 {
                for px in x0..x1 {
                    if px >= 0 && px < fb.width() as i32 && py >= 0 && py < fb.height() as i32 {
                        // Make a simple pattern for each character
                        if (px - x0) % 2 == (py - y0) % 2 {
                            fb.set_pixel(px, py, color);
                        }
                    }
                }
            }
        }
        
        curr_x += (font_size as i32) + (2.0 * scale) as i32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_frame_buffer_creation() {
        let fb = FrameBuffer::new(100, 100);
        
        // Check dimensions
        assert_eq!(fb.width(), 100);
        assert_eq!(fb.height(), 100);
        
        // Check buffer length
        assert_eq!(fb.buffer().len(), 100 * 100 * 4);
    }
    
    #[test]
    fn test_frame_buffer_clear() {
        let mut fb = FrameBuffer::new(100, 100);
        let color = Color::new(255, 0, 0); // Red
        
        // Clear with red
        fb.clear(color);
        
        // Check first and last pixels
        let first_pixel = &fb.buffer()[0..4];
        let last_pixel = &fb.buffer()[fb.buffer().len() - 4..];
        
        assert_eq!(first_pixel, &[255, 0, 0, 255]);
        assert_eq!(last_pixel, &[255, 0, 0, 255]);
    }
    
    #[test]
    fn test_set_pixel() {
        let mut fb = FrameBuffer::new(10, 10);
        let color = Color::new(0, 255, 0); // Green
        
        // Set a pixel
        fb.set_pixel(5, 5, color);
        
        // Check the pixel color
        let index = ((5 * 10 + 5) * 4) as usize;
        assert_eq!(&fb.buffer()[index..index + 4], &[0, 255, 0, 255]);
    }
}
