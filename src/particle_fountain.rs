use crate::pixel_utils::*;

/// Convert HSV to RGB
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let c = v * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = v - c;
    
    let (r, g, b) = match (h * 6.0) as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    
    [
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    ]
}

/// Draw a rainbow particle fountain effect
pub fn draw_frame(frame: &mut [u8], width: u32, height: u32, elapsed: f32) {
    // Clear frame with black background
    frame.chunks_exact_mut(4).for_each(|pixel| {
        pixel[0] = 0;   // R
        pixel[1] = 0;   // G
        pixel[2] = 0;   // B
        pixel[3] = 255; // A
    });
    
    let fountain_x = width as f32 / 2.0;
    let fountain_y = height as f32 * 0.8; // Near bottom
    
    // Draw fountain base (concentric circles)
    for radius in 0..40 {
        let color_intensity = 255 - (radius * 5).min(255);
        draw_circle(frame, fountain_x as i32, fountain_y as i32, 40 - radius, 
                   [255, color_intensity as u8, 0, 255], width);
    }
    
    // Draw pulsing fountain text
    let blink = ((elapsed * 5.0).sin() * 0.5 + 0.5) * 255.0;
    draw_huge_text(frame, "FOUNTAIN", width as i32 / 2 - 200, 50, 
                  [255, blink as u8, blink as u8, 255], width);
    
    // Draw particles
    let particles = 1000;
    for i in 0..particles {
        let lifetime = 2.0; 
        let particle_time = (elapsed + i as f32 * 0.01) % lifetime;
        let progress = particle_time / lifetime;
        
        // Spray angle - wide fountain effect
        let angle = std::f32::consts::PI / 2.0 + (i as f32 / particles as f32 - 0.5) * std::f32::consts::PI * 1.5;
        let speed = 600.0 * (1.0 - progress * 0.3);
        let gravity = 800.0;
        
        // Calculate particle position
        let x = fountain_x + angle.cos() * speed * particle_time;
        let y = fountain_y - angle.sin() * speed * particle_time + 0.5 * gravity * particle_time * particle_time;
        
        // Only draw if particle is within bounds
        if x >= 0.0 && x < width as f32 && y >= 0.0 && y < height as f32 {
            // Calculate fade based on lifetime
            let fade = if progress < 0.1 {
                progress / 0.1
            } else if progress > 0.7 {
                (1.0 - progress) / 0.3
            } else {
                1.0
            };
            
            // Rainbow colors cycling through hue
            let hue = (i as f32 / particles as f32 + elapsed * 0.3) % 1.0;
            let color = hsv_to_rgb(hue, 1.0, 1.0);
            let size = 4 + (10.0 * (1.0 - progress)) as i32;
            
            draw_extra_bright_particle(frame, x as i32, y as i32, size, 
                                     [color[0], color[1], color[2], (255.0 * fade) as u8], width);
        }
    }
    
    // Draw pulsing border effect
    let pulse = (elapsed * 10.0).sin() > 0.0;
    let border_color = if pulse { [255, 0, 0, 255] } else { [255, 255, 255, 255] };
    draw_border(frame, 0, 0, width as i32, height as i32, border_color, width);
    
    // Add sparkle effect when pulsing
    if pulse {
        for y in 0..(height as usize) {
            for x in 0..(width as usize) {
                let idx = 4 * (y * width as usize + x);
                if idx + 3 < frame.len() && (x + y) % 20 == 0 {
                    frame[idx] = 255;
                    frame[idx + 1] = 255;
                    frame[idx + 2] = 0;
                    frame[idx + 3] = 255;
                }
            }
        }
    }
}
