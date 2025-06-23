use pixels::Pixels;
use crate::pixel_utils::*;

/// Draw a Pythagorean theorem visualization
pub fn draw_frame(pixels: &mut Pixels, elapsed: f32) {
    // Get dimensions first
    let width = pixels.texture().width();
    let height = pixels.texture().height();
    
    // Then get the frame
    let frame = pixels.frame_mut();
    
    // Clear the frame with a white background
    for pixel in frame.chunks_exact_mut(4) {
        pixel[0] = 255; // R
        pixel[1] = 255; // G
        pixel[2] = 255; // B
        pixel[3] = 255; // A
    }
    
    // Parameters
    let a = 100.0f32;
    let b = 150.0f32;
    let c = (a*a + b*b).sqrt();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let angle = elapsed * 0.5; // Rotation angle
    
    // Draw big square (c × c) - light gray
    let square_color = [200, 200, 200, 255];
    let half_c = (c / 2.0) as i32;
    
    for y in -half_c..half_c {
        for x in -half_c..half_c {
            set_pixel_safe(frame, 
                           center_x as i32 + x, 
                           center_y as i32 + y, 
                           width, height, 
                           square_color);
        }
    }
    
    // Draw four triangles (blue)
    let triangle_color = [0, 0, 255, 255];
    
    for i in 0..4 {
        let theta = angle + i as f32 * std::f32::consts::FRAC_PI_2;
        
        // Triangle vertices
        let p1_x = center_x + theta.cos() * (c / 2.0);
        let p1_y = center_y + theta.sin() * (c / 2.0);
        
        let p2_x = center_x + (theta + (b as f32).to_radians()).cos() * (a / 2.0);
        let p2_y = center_y + (theta + b.to_radians()).sin() * (a / 2.0);
        
        let p3_x = center_x + (theta - (a as f32).to_radians()).cos() * (b / 2.0);
        let p3_y = center_y + (theta - a.to_radians()).sin() * (b / 2.0);
        
        // Draw filled triangle using the function from pixel_utils
        draw_triangle_filled(
            frame,
            p1_x as i32, p1_y as i32,
            p2_x as i32, p2_y as i32,
            p3_x as i32, p3_y as i32,
            width, height,
            triangle_color
        );
    }
    
    // Draw explanatory text
    let text_color = [0, 0, 0, 255];
    draw_simple_text(frame, "Pythagoras Theorem: a² + b² = c²", 
                   20, 30, 
                   width, height, 
                   text_color);
    
    let a_squared = (a * a).round() as i32;
    let b_squared = (b * b).round() as i32;
    let c_squared = (c * c).round() as i32;
    
    draw_simple_text(frame, &format!("{} + {} = {}", a_squared, b_squared, c_squared), 
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
