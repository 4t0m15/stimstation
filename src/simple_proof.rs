use pixels::Pixels;
use crate::pixel_utils::*;

/// Draw a simple mathematical proof visualization (triangular numbers)
pub fn draw_frame(pixels: &mut Pixels, elapsed: f32) {
    // Store dimensions first
    let width = pixels.texture().width();
    let height = pixels.texture().height();
    // Then get frame
    let frame = pixels.frame_mut();
    
    // Clear frame with white background
    for pixel in frame.chunks_exact_mut(4) {
        pixel[0] = 255; // R
        pixel[1] = 255; // G
        pixel[2] = 255; // B
        pixel[3] = 255; // A
    }
    
    // Visual proof that 1 + 2 + 3 + ... + n = n(n+1)/2
    let n = ((elapsed.sin() * 4.0 + 10.0) as i32).max(5).min(15); // Vary between 5-15
    let sum = n * (n + 1) / 2;
    
    // Draw title
    let text_color = [0, 0, 0, 255];
    draw_simple_text(frame, &format!("Visual proof: 1 + 2 + 3 + ... + {} = {}*({} + 1)/2 = {}", 
                            n, n, n, sum), 
              20, 30, 
              width, height, 
              text_color);
    
    // Draw triangular pattern of dots
    let dot_size = 5;
    let spacing = 15;
    let start_x = (width as i32 / 2) - (n * spacing / 2);
    let start_y = 100;
    
    // Draw the triangular arrangement
    for i in 1..=n {
        for j in 1..=i {
            let x = start_x + (j - 1) * spacing;
            let y = start_y + (i - 1) * spacing;
            
            // Draw a dot (small filled circle)
            for dy in -dot_size..=dot_size {
                for dx in -dot_size..=dot_size {
                    if dx*dx + dy*dy <= dot_size*dot_size {
                        set_pixel_safe(frame, x + dx, y + dy, width, height, [255, 0, 0, 255]);
                    }
                }
            }
        }
        
        // Draw the row sum
        draw_simple_text(frame, &format!("Row {}: {}", i, i), 
                  start_x + n * spacing + 20, 
                  start_y + (i - 1) * spacing, 
                  width, height, 
                  text_color);
    }
    
    // Draw the rectangle proof (n by n+1 rectangle split into two triangles)
    let rect_start_x = start_x;
    let rect_start_y = start_y + (n + 3) * spacing;
    
    draw_simple_text(frame, "Alternative proof: n(n+1)/2 is half of an n × (n+1) rectangle", 
              20, rect_start_y - 30, 
              width, height, 
              text_color);
    
    // Draw the rectangle
    for i in 0..n {
        for j in 0..n+1 {
            let x = rect_start_x + j * spacing;
            let y = rect_start_y + i * spacing;
            
            // Draw a dot (small filled circle)
            for dy in -dot_size..=dot_size {
                for dx in -dot_size..=dot_size {
                    if dx*dx + dy*dy <= dot_size*dot_size {
                        // Different colors for upper and lower triangles
                        let color = if i + j < n {
                            [0, 0, 255, 255]  // Blue for lower triangle
                        } else {
                            [0, 150, 0, 255]  // Green for upper triangle
                        };
                        
                        set_pixel_safe(frame, x + dx, y + dy, width, height, color);
                    }
                }
            }
        }
    }
    
    // Draw the diagonal line separating the triangles
    for i in 0..=n {
        let x = rect_start_x + i * spacing;
        let y = rect_start_y + (n - i) * spacing;
        
        for dy in -2..=2 {
            for dx in -2..=2 {
                if dx*dx + dy*dy <= 4 {
                    set_pixel_safe(frame, x + dx, y + dy, width, height, [0, 0, 0, 255]);
                }
            }
        }
    }
    
    // Show the formula
    draw_simple_text(frame, &format!("Rectangle area: {} × {} = {}", n, n+1, n*(n+1)), 
              rect_start_x, rect_start_y + n * spacing + 30, 
              width, height, 
              text_color);
              
    draw_simple_text(frame, &format!("Triangle area (half): {}/{} = {}", n*(n+1), 2, n*(n+1)/2), 
              rect_start_x, rect_start_y + n * spacing + 50, 
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
