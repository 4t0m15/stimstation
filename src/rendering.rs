use crate::types::{Color, Position, color_to_rgba};
use glam::Vec2;

// Optimized frame buffer with bounds checking done once
pub struct FrameBuffer {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            data: vec![0; (width * height * 4) as usize],
            width,
            height,
        }
    }

    pub fn clear(&mut self, color: Color) {
        let rgba = color_to_rgba(color);
        self.data.chunks_exact_mut(4).for_each(|pixel| {
            pixel.copy_from_slice(&rgba);
        });
    }

    pub fn fade(&mut self, factor: f32) {
        self.data.chunks_exact_mut(4).for_each(|pixel| {
            pixel[0] = (pixel[0] as f32 * factor) as u8;
            pixel[1] = (pixel[1] as f32 * factor) as u8;
            pixel[2] = (pixel[2] as f32 * factor) as u8;
        });
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    // Fast pixel access with bounds checking
    #[inline]
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let idx = 4 * (y as usize * self.width as usize + x as usize);
            let rgba = color_to_rgba(color);
            self.data[idx..idx + 4].copy_from_slice(&rgba);
        }
    }

    // Fast pixel access with alpha blending
    #[inline]
    pub fn blend_pixel(&mut self, x: i32, y: i32, color: Color, intensity: f32) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let idx = 4 * (y as usize * self.width as usize + x as usize);
            let rgba = color_to_rgba(color);
            
            let r = (intensity * rgba[0] as f32) as u16;
            let g = (intensity * rgba[1] as f32) as u16;
            let b = (intensity * rgba[2] as f32) as u16;
            
            self.data[idx] = (self.data[idx] as u16 + r).min(255) as u8;
            self.data[idx + 1] = (self.data[idx + 1] as u16 + g).min(255) as u8;
            self.data[idx + 2] = (self.data[idx + 2] as u16 + b).min(255) as u8;
        }
    }
}

// Optimized line drawing using Bresenham's algorithm
pub fn draw_line_fast(frame: &mut FrameBuffer, start: Position, end: Position, color: Color, width: i32) {
    let x0 = start.x as i32;
    let y0 = start.y as i32;
    let x1 = end.x as i32;
    let y1 = end.y as i32;
    
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut x = x0;
    let mut y = y0;
    
    let glow_radius = width * 3;
    
    // Early culling
    if (x0 < -glow_radius && x1 < -glow_radius) || (x0 >= frame.width() as i32 + glow_radius && x1 >= frame.width() as i32 + glow_radius) ||
       (y0 < -glow_radius && y1 < -glow_radius) || (y0 >= frame.height() as i32 + glow_radius && y1 >= frame.height() as i32 + glow_radius) {
        return;
    }

    while x >= -glow_radius && x < frame.width() as i32 + glow_radius && 
          y >= -glow_radius && y < frame.height() as i32 + glow_radius {
        
        // Draw glow effect
        for w_y in -glow_radius..=glow_radius {
            for w_x in -glow_radius..=glow_radius {
                let distance_squared = w_x * w_x + w_y * w_y;
                let distance = (distance_squared as f32).sqrt();
                
                if distance > glow_radius as f32 { continue; }
                
                let intensity = if distance <= width as f32 {
                    1.0
                } else {
                    let falloff = 1.0 - (distance - width as f32) / (glow_radius as f32 - width as f32);
                    falloff * falloff
                };
                
                frame.blend_pixel(x + w_x, y + w_y, color, intensity);
            }
        }

        if x == x1 && y == y1 { break; }

        let e2 = 2 * err;
        if e2 > -dy { err -= dy; x += sx; }
        if e2 < dx { err += dx; y += sy; }
    }
}

// Optimized circle drawing
pub fn draw_circle_fast(frame: &mut FrameBuffer, center: Position, radius: i32, color: Color) {
    let cx = center.x as i32;
    let cy = center.y as i32;
    
    // Early culling
    if cx + radius < 0 || cx - radius >= frame.width() as i32 || 
       cy + radius < 0 || cy - radius >= frame.height() as i32 {
        return;
    }
    
    let radius_sq = radius * radius;
    
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx*dx + dy*dy <= radius_sq {
                frame.set_pixel(cx + dx, cy + dy, color);
            }
        }
    }
}

// Optimized particle drawing
pub fn draw_particle_fast(frame: &mut FrameBuffer, pos: Position, size: i32, color: Color) {
    let x = pos.x as i32;
    let y = pos.y as i32;
    let glow_radius = size * 2;
    
    // Early culling
    if x + glow_radius < 0 || x - glow_radius >= frame.width() as i32 || 
       y + glow_radius < 0 || y - glow_radius >= frame.height() as i32 {
        return;
    }
    
    for dy in -glow_radius..=glow_radius {
        for dx in -glow_radius..=glow_radius {
            let distance_squared = dx * dx + dy * dy;
            let distance = (distance_squared as f32).sqrt();
            
            if distance > glow_radius as f32 { continue; }
            
            let intensity = if distance <= size as f32 {
                1.0
            } else {
                let falloff = 1.0 - (distance - size as f32) / (glow_radius as f32 - size as f32);
                falloff * falloff
            };
            
            frame.blend_pixel(x + dx, y + dy, color, intensity);
        }
    }
}

// Optimized triangle drawing using scanline algorithm
pub fn draw_triangle_filled(
    frame: &mut FrameBuffer,
    v1: Position,
    v2: Position,
    v3: Position,
    color: Color,
) {
    let mut vertices = [v1, v2, v3];
    vertices.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
    
    let [v1, v2, v3] = vertices;
    
    // Draw the top half of the triangle
    if v2.y > v1.y {
        let slope1 = (v2.x - v1.x) / (v2.y - v1.y);
        let slope2 = (v3.x - v1.x) / (v3.y - v1.y);
        
        for y in (v1.y as i32)..=(v2.y as i32) {
            let dy = y as f32 - v1.y;
            let start_x = v1.x + slope1 * dy;
            let end_x = v1.x + slope2 * dy;
            
            let start = start_x.min(end_x) as i32;
            let end = start_x.max(end_x) as i32;
            
            for x in start..=end {
                frame.set_pixel(x, y, color);
            }
        }
    }
    
    // Draw the bottom half of the triangle
    if v3.y > v2.y {
        let slope1 = (v3.x - v2.x) / (v3.y - v2.y);
        let slope2 = (v3.x - v1.x) / (v3.y - v1.y);
        
        for y in (v2.y as i32 + 1)..=(v3.y as i32) {
            let dy1 = y as f32 - v2.y;
            let dy2 = y as f32 - v1.y;
            let start_x = v2.x + slope1 * dy1;
            let end_x = v1.x + slope2 * dy2;
            
            let start = start_x.min(end_x) as i32;
            let end = start_x.max(end_x) as i32;
            
            for x in start..=end {
                frame.set_pixel(x, y, color);
            }
        }
    }
}

// Optimized text rendering with simple font
pub fn draw_text_fast(
    frame: &mut FrameBuffer,
    text: &str,
    pos: Position,
    color: Color,
    scale: f32,
) {
    let char_width = (8.0 * scale) as i32;
    let char_height = (15.0 * scale) as i32;
    let x = pos.x as i32;
    let y = pos.y as i32;
    
    // Early return if text would be completely off-screen
    if y + char_height < 0 || y >= frame.height() as i32 {
        return;
    }
    
    for (i, c) in text.chars().enumerate() {
        let cx = x + (i as i32 * char_width);
        
        // Skip if this character is outside frame
        if cx + char_width < 0 || cx >= frame.width() as i32 {
            continue;
        }
        
        // Draw character using 7-segment style
        draw_character_fast(frame, c, Vec2::new(cx as f32, y as f32), color, scale);
    }
}

// Fast character rendering
fn draw_character_fast(frame: &mut FrameBuffer, c: char, pos: Vec2, color: Color, scale: f32) {
    let char_width = (8.0 * scale) as i32;
    let char_height = (15.0 * scale) as i32;
    let thickness = (2.0 * scale) as i32;
    let x = pos.x as i32;
    let y = pos.y as i32;
    
    match c {
        'F' => {
            draw_segment_fast(frame, x, y, true, false, true, false, true, false, false, color, thickness, char_width, char_height);
        },
        'P' => {
            draw_segment_fast(frame, x, y, true, false, true, true, true, false, false, color, thickness, char_width, char_height);
        },
        'S' => {
            draw_segment_fast(frame, x, y, true, true, false, true, false, true, true, color, thickness, char_width, char_height);
        },
        '0' => {
            draw_segment_fast(frame, x, y, true, true, true, false, true, true, true, color, thickness, char_width, char_height);
        },
        '1' => {
            draw_segment_fast(frame, x, y, false, false, true, false, false, true, false, color, thickness, char_width, char_height);
        },
        '2' => {
            draw_segment_fast(frame, x, y, true, true, false, true, true, false, true, color, thickness, char_width, char_height);
        },
        '3' => {
            draw_segment_fast(frame, x, y, true, true, true, true, false, false, true, color, thickness, char_width, char_height);
        },
        '4' => {
            draw_segment_fast(frame, x, y, false, false, true, true, false, true, true, color, thickness, char_width, char_height);
        },
        '5' => {
            draw_segment_fast(frame, x, y, true, true, true, true, false, true, false, color, thickness, char_width, char_height);
        },
        '6' => {
            draw_segment_fast(frame, x, y, true, true, true, true, true, true, false, color, thickness, char_width, char_height);
        },
        '7' => {
            draw_segment_fast(frame, x, y, true, false, true, false, false, true, false, color, thickness, char_width, char_height);
        },
        '8' => {
            draw_segment_fast(frame, x, y, true, true, true, true, true, true, true, color, thickness, char_width, char_height);
        },
        '9' => {
            draw_segment_fast(frame, x, y, true, true, true, true, false, true, true, color, thickness, char_width, char_height);
        },
        ':' => {
            // Draw two dots
            for dy in 0..thickness {
                for dx in 0..thickness {
                    frame.set_pixel(x + 3 + dx, y + 4 + dy, color);
                    frame.set_pixel(x + 3 + dx, y + 10 + dy, color);
                }
            }
        },
        '.' => {
            // Draw a single dot
            for dy in 0..thickness {
                for dx in 0..thickness {
                    frame.set_pixel(x + 3 + dx, y + char_height - 3 + dy, color);
                }
            }
        },
        _ => {
            // Draw a rectangle for unknown characters
            for dy in 0..char_height {
                for dx in 0..char_width {
                    if dx == 0 || dx == char_width - 1 || dy == 0 || dy == char_height - 1 {
                        frame.set_pixel(x + dx, y + dy, color);
                    }
                }
            }
        }
    }
}

// Fast 7-segment display rendering
fn draw_segment_fast(
    frame: &mut FrameBuffer,
    x: i32, y: i32,
    a: bool, b: bool, c: bool, d: bool, e: bool, f: bool, g: bool,
    color: Color,
    thickness: i32,
    char_width: i32,
    char_height: i32,
) {
    // Segment a (top horizontal)
    if a {
        for dy in 0..thickness {
            for dx in 0..(char_width - 2) {
                frame.set_pixel(x + 1 + dx, y + dy, color);
            }
        }
    }
    
    // Segment b (top-right vertical)
    if b {
        for dy in 0..(char_height / 2 - 1) {
            for dx in 0..thickness {
                frame.set_pixel(x + char_width - 1 - dx, y + 1 + dy, color);
            }
        }
    }
    
    // Segment c (bottom-right vertical)
    if c {
        for dy in 0..(char_height / 2 - 1) {
            for dx in 0..thickness {
                frame.set_pixel(x + char_width - 1 - dx, y + char_height / 2 + dy, color);
            }
        }
    }
    
    // Segment d (bottom horizontal)
    if d {
        for dy in 0..thickness {
            for dx in 0..(char_width - 2) {
                frame.set_pixel(x + 1 + dx, y + char_height - 1 - dy, color);
            }
        }
    }
    
    // Segment e (bottom-left vertical)
    if e {
        for dy in 0..(char_height / 2 - 1) {
            for dx in 0..thickness {
                frame.set_pixel(x + dx, y + char_height / 2 + dy, color);
            }
        }
    }
    
    // Segment f (top-left vertical)
    if f {
        for dy in 0..(char_height / 2 - 1) {
            for dx in 0..thickness {
                frame.set_pixel(x + dx, y + 1 + dy, color);
            }
        }
    }
    
    // Segment g (middle horizontal)
    if g {
        for dy in 0..thickness {
            for dx in 0..(char_width - 2) {
                frame.set_pixel(x + 1 + dx, y + char_height / 2 - thickness / 2 + dy, color);
            }
        }
    }
} 