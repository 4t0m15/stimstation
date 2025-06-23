use pixels::Pixels;
use crate::pixel_utils::*;

/// Draw a Fibonacci spiral visualization
pub fn draw_frame(pixels: &mut Pixels, elapsed: f32) {
    // Store dimensions first
    let width = pixels.texture().width();
    let height = pixels.texture().height();
    // Then get frame
    let frame = pixels.frame_mut();
    
    // Use elapsed time to add subtle animation effect
    let animation_offset = (elapsed * 0.5).sin() * 5.0;
    
    // Clear frame with white background
    for pixel in frame.chunks_exact_mut(4) {
        pixel[0] = 255; // R
        pixel[1] = 255; // G
        pixel[2] = 255; // B
        pixel[3] = 255; // A
    }
    
    // Calculate first few Fibonacci numbers
    let mut fibonacci = vec![1, 1];
    for i in 2..12 {
        fibonacci.push(fibonacci[i-1] + fibonacci[i-2]);
    }
    
    // Colors for each square
    let colors = [
        [255, 0, 0, 255],    // Red
        [0, 255, 0, 255],    // Green
        [0, 0, 255, 255],    // Blue
        [255, 255, 0, 255],  // Yellow
        [255, 0, 255, 255],  // Magenta
        [0, 255, 255, 255],  // Cyan
        [255, 128, 0, 255],  // Orange
        [128, 0, 255, 255],  // Purple
        [0, 128, 0, 255],    // Dark green
        [128, 128, 255, 255],// Light blue
        [128, 64, 0, 255],   // Brown
        [255, 128, 128, 255],// Pink
    ];
    
    let scale_factor = 4.0; // Scale the spiral to fit the window
    let center_x = width as i32 / 2;
    let center_y = height as i32 / 2;
    let offset_x = center_x - (fibonacci[fibonacci.len()-1] as f32 * scale_factor / 2.0) as i32 + animation_offset as i32;
    let offset_y = center_y - (fibonacci[fibonacci.len()-1] as f32 * scale_factor / 2.0) as i32;
    
    // Draw the squares
    let mut x = 0;
    let mut y = 0;
    let mut direction = 0; // 0: right, 1: down, 2: left, 3: up
    
    for (i, &fib) in fibonacci.iter().enumerate() {
        let size = (fib as f32 * scale_factor) as i32;
        let color = colors[i % colors.len()];
        
        // Draw the square
        for sx in 0..size {
            for sy in 0..size {
                let px = offset_x + x + sx;
                let py = offset_y + y + sy;
                
                // Draw border
                if sx == 0 || sx == size - 1 || sy == 0 || sy == size - 1 {
                    set_pixel_safe(frame, px, py, width, height, [0, 0, 0, 255]);
                } else {
                    // Fill with a lighter version of the color
                    set_pixel_safe(frame, px, py, width, height, 
                                  [color[0]/2 + 128, color[1]/2 + 128, color[2]/2 + 128, 255]);
                }
            }
        }
        
        // Draw a quarter circle in each square to form the spiral
        let radius = size;
        let center_spiral_x;
        let center_spiral_y;
        
        match direction {
            0 => { // right
                center_spiral_x = x + size;
                center_spiral_y = y + size;
                
                // Draw arc - bottom right corner
                for angle in 0..90 {
                    let rad_angle = (angle as f32) * std::f32::consts::PI / 180.0;
                    let arc_x = center_spiral_x as f32 - rad_angle.sin() * radius as f32;
                    let arc_y = center_spiral_y as f32 - rad_angle.cos() * radius as f32;
                    
                    set_pixel_safe(frame, 
                                  offset_x + arc_x as i32, 
                                  offset_y + arc_y as i32, 
                                  width, height, 
                                  [0, 0, 0, 255]);
                }
                
                // Update for next square
                x += size;
            },
            1 => { // down
                center_spiral_x = x;
                center_spiral_y = y + size;
                
                // Draw arc - bottom left corner
                for angle in 0..90 {
                    let rad_angle = (angle as f32) * std::f32::consts::PI / 180.0;
                    let arc_x = center_spiral_x as f32 + rad_angle.cos() * radius as f32;
                    let arc_y = center_spiral_y as f32 - rad_angle.sin() * radius as f32;
                    
                    set_pixel_safe(frame, 
                                  offset_x + arc_x as i32, 
                                  offset_y + arc_y as i32, 
                                  width, height, 
                                  [0, 0, 0, 255]);
                }
                
                // Update for next square
                y += size;
            },
            2 => { // left
                center_spiral_x = x;
                center_spiral_y = y;
                
                // Draw arc - top left corner
                for angle in 0..90 {
                    let rad_angle = (angle as f32) * std::f32::consts::PI / 180.0;
                    let arc_x = center_spiral_x as f32 + rad_angle.sin() * radius as f32;
                    let arc_y = center_spiral_y as f32 + rad_angle.cos() * radius as f32;
                    
                    set_pixel_safe(frame, 
                                  offset_x + arc_x as i32, 
                                  offset_y + arc_y as i32, 
                                  width, height, 
                                  [0, 0, 0, 255]);
                }
                
                // Update for next square
                x -= size;
            },
            3 => { // up
                center_spiral_x = x + size;
                center_spiral_y = y;
                
                // Draw arc - top right corner
                for angle in 0..90 {
                    let rad_angle = (angle as f32) * std::f32::consts::PI / 180.0;
                    let arc_x = center_spiral_x as f32 - rad_angle.cos() * radius as f32;
                    let arc_y = center_spiral_y as f32 + rad_angle.sin() * radius as f32;
                    
                    set_pixel_safe(frame, 
                                  offset_x + arc_x as i32, 
                                  offset_y + arc_y as i32, 
                                  width, height, 
                                  [0, 0, 0, 255]);
                }
                
                // Update for next square
                y -= size;
            },
            _ => unreachable!(),
        }
        
        // Change direction for next square
        direction = (direction + 1) % 4;
    }
    
    // Draw explanatory text
    let text_color = [0, 0, 0, 255];
    draw_simple_text(frame, "Fibonacci Spiral", 
                  20, 30, 
                  width, height, 
                  text_color);
    
    draw_simple_text(frame, &format!("Fibonacci sequence: {:?}", &fibonacci[..10]), 
              20, 50, 
              width, height, 
              text_color);
}

/// Helper function to draw simple text
fn draw_simple_text(
    frame: &mut [u8],
    text: &str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    color: [u8; 4]
) {
    // Simple ASCII character rendering with fixed-width font
    let char_width = 8;
    let char_height = 10;
    
    for (i, c) in text.chars().enumerate() {
        let cx = x + i as i32 * char_width;
        
        // Skip if out of view
        if cx < 0 || cx >= width as i32 || y < 0 || y >= height as i32 {
            continue;
        }
        
        // Draw each character
        match c {
            'A' => {
                for dy in 0..char_height {
                    for dx in 0..char_width {
                        if (dx == 0 || dx == char_width-1) && dy > 0 || // vertical lines
                           dy == 0 && dx > 0 && dx < char_width-1 ||     // top
                           dy == char_height/2 {                         // middle
                            set_pixel_safe(frame, cx + dx, y + dy, width, height, color);
                        }
                    }
                }
            },
            // Add more characters as needed
            _ => {
                // Simple box for unimplemented characters
                for dy in 0..char_height {
                    for dx in 0..char_width {
                        if dy == 0 || dy == char_height-1 || dx == 0 || dx == char_width-1 {
                            set_pixel_safe(frame, cx + dx, y + dy, width, height, color);
                        }
                    }
                }
            }
        }
    }
}
