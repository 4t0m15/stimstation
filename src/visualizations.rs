use crate::types::{Color, Position, hsv_to_rgb, WIDTH, HEIGHT};
use crate::rendering::{FrameBuffer, draw_triangle_filled, draw_text_fast, draw_circle_fast};

// Pythagoras theorem visualization
pub fn draw_pythagoras(frame: &mut FrameBuffer, elapsed: f32) {
    frame.clear(Color::new(255, 255, 255));
    
    let a = 100.0f32;
    let b = 150.0f32;
    let c = (a*a + b*b).sqrt();
    let center_x = frame.width() as f32 / 2.0;
    let center_y = frame.height() as f32 / 2.0;
    let angle = elapsed * 0.5;
    
    // Draw big square (c × c) - light gray
    let square_color = Color::new(200, 200, 200);
    let half_c = (c / 2.0) as i32;
    
    for y in -half_c..half_c {
        for x in -half_c..half_c {
            frame.set_pixel(
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
        
        // Triangle vertices
        let p1 = Position::new(
            center_x + theta.cos() * (c / 2.0),
            center_y + theta.sin() * (c / 2.0)
        );
        
        let p2 = Position::new(
            center_x + (theta + (b as f32).to_radians()).cos() * (a / 2.0),
            center_y + (theta + b.to_radians()).sin() * (a / 2.0)
        );
        
        let p3 = Position::new(
            center_x + (theta - (a as f32).to_radians()).cos() * (b / 2.0),
            center_y + (theta - a.to_radians()).sin() * (b / 2.0)
        );
        
        draw_triangle_filled(frame, p1, p2, p3, triangle_color);
    }
    
    // Draw explanatory text
    let text_color = Color::new(0, 0, 0);
    draw_text_fast(frame, "Pythagoras Theorem: a² + b² = c²", 
                   Position::new(20.0, 30.0), text_color, 1.0);
    
    let a_squared = (a * a).round() as i32;
    let b_squared = (b * b).round() as i32;
    let c_squared = (c * c).round() as i32;
    
    draw_text_fast(frame, &format!("{} + {} = {}", a_squared, b_squared, c_squared), 
                   Position::new(20.0, 50.0), text_color, 1.0);
}

// Fibonacci spiral visualization
pub fn draw_fibonacci_spiral(frame: &mut FrameBuffer, elapsed: f32) {
    frame.clear(Color::new(255, 255, 255));
    
    let animation_offset = (elapsed * 0.5).sin() * 5.0;
    
    // Calculate first few Fibonacci numbers
    let mut fibonacci = vec![1, 1];
    for i in 2..12 {
        fibonacci.push(fibonacci[i-1] + fibonacci[i-2]);
    }
    
    // Colors for each square
    let colors = [
        Color::new(255, 0, 0),    // Red
        Color::new(0, 255, 0),    // Green
        Color::new(0, 0, 255),    // Blue
        Color::new(255, 255, 0),  // Yellow
        Color::new(255, 0, 255),  // Magenta
        Color::new(0, 255, 255),  // Cyan
        Color::new(255, 128, 0),  // Orange
        Color::new(128, 0, 255),  // Purple
        Color::new(0, 128, 0),    // Dark green
        Color::new(128, 128, 255),// Light blue
        Color::new(128, 64, 0),   // Brown
        Color::new(255, 128, 128),// Pink
    ];
    
    let scale_factor = 4.0;
    let center_x = frame.width() as i32 / 2;
    let center_y = frame.height() as i32 / 2;
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
                    frame.set_pixel(px, py, Color::new(0, 0, 0));
                } else {
                    // Fill with a lighter version of the color
                    frame.set_pixel(px, py, Color::new(
                        color.red/2 + 128,
                        color.green/2 + 128,
                        color.blue/2 + 128
                    ));
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
                    
                    frame.set_pixel(
                        offset_x + arc_x as i32,
                        offset_y + arc_y as i32,
                        Color::new(0, 0, 0)
                    );
                }
                
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
                    
                    frame.set_pixel(
                        offset_x + arc_x as i32,
                        offset_y + arc_y as i32,
                        Color::new(0, 0, 0)
                    );
                }
                
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
                    
                    frame.set_pixel(
                        offset_x + arc_x as i32,
                        offset_y + arc_y as i32,
                        Color::new(0, 0, 0)
                    );
                }
                
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
                    
                    frame.set_pixel(
                        offset_x + arc_x as i32,
                        offset_y + arc_y as i32,
                        Color::new(0, 0, 0)
                    );
                }
                
                y -= size;
            },
            _ => unreachable!(),
        }
        
        direction = (direction + 1) % 4;
    }
    
    // Draw explanatory text
    let text_color = Color::new(0, 0, 0);
    draw_text_fast(frame, "Fibonacci Spiral", 
                   Position::new(20.0, 30.0), text_color, 1.0);
    
    draw_text_fast(frame, &format!("Fibonacci sequence: {:?}", &fibonacci[..10]), 
                   Position::new(20.0, 50.0), text_color, 1.0);
}

// Simple proof visualization
pub fn draw_simple_proof(frame: &mut FrameBuffer, elapsed: f32) {
    frame.clear(Color::new(255, 255, 255));
    
    // Visual proof that 1 + 2 + 3 + ... + n = n(n+1)/2
    let n = ((elapsed.sin() * 4.0 + 10.0) as i32).max(5).min(15);
    let sum = n * (n + 1) / 2;
    
    // Draw title
    let text_color = Color::new(0, 0, 0);
    draw_text_fast(frame, &format!("Visual proof: 1 + 2 + 3 + ... + {} = {}*({} + 1)/2 = {}", 
                            n, n, n, sum), 
                   Position::new(20.0, 30.0), text_color, 1.0);
    
    // Draw triangular pattern of dots
    let dot_size = 5;
    let spacing = 15;
    let start_x = (frame.width() as i32 / 2) - (n * spacing / 2);
    let start_y = 100;
    
    // Draw the triangular arrangement
    for i in 1..=n {
        for j in 1..=i {
            let x = start_x + (j - 1) * spacing;
            let y = start_y + (i - 1) * spacing;
            
            // Draw a dot (small filled circle)
            draw_circle_fast(frame, Position::new(x as f32, y as f32), dot_size, Color::new(255, 0, 0));
        }
        
        // Draw the row sum
        draw_text_fast(frame, &format!("Row {}: {}", i, i), 
                       Position::new((start_x + n * spacing + 20) as f32, (start_y + (i - 1) * spacing) as f32), 
                       text_color, 1.0);
    }
    
    // Draw the rectangle proof
    let rect_start_x = start_x;
    let rect_start_y = start_y + (n + 3) * spacing;
    
    draw_text_fast(frame, "Alternative proof: n(n+1)/2 is half of an n × (n+1) rectangle", 
                   Position::new(20.0, (rect_start_y - 30) as f32), text_color, 1.0);
    
    // Draw the rectangle
    for i in 0..n {
        for j in 0..n+1 {
            let x = rect_start_x + j * spacing;
            let y = rect_start_y + i * spacing;
            
            // Different colors for upper and lower triangles
            let color = if i + j < n {
                Color::new(0, 0, 255)  // Blue for lower triangle
            } else {
                Color::new(0, 150, 0)  // Green for upper triangle
            };
            
            draw_circle_fast(frame, Position::new(x as f32, y as f32), dot_size, color);
        }
    }
    
    // Draw the diagonal line separating the triangles
    for i in 0..=n {
        let x = rect_start_x + i * spacing;
        let y = rect_start_y + (n - i) * spacing;
        
        draw_circle_fast(frame, Position::new(x as f32, y as f32), 2, Color::new(0, 0, 0));
    }
    
    // Show the formula
    draw_text_fast(frame, &format!("Rectangle area: {} × {} = {}", n, n+1, n*(n+1)), 
                   Position::new(rect_start_x as f32, (rect_start_y + n * spacing + 30) as f32), 
                   text_color, 1.0);
                  
    draw_text_fast(frame, &format!("Triangle area (half): {}/{} = {}", n*(n+1), 2, n*(n+1)/2), 
                   Position::new(rect_start_x as f32, (rect_start_y + n * spacing + 50) as f32), 
                   text_color, 1.0);
}

// Particle fountain effect
#[allow(dead_code)]
pub fn draw_particle_fountain(frame: &mut FrameBuffer, time: f32) {
    let fountain_x = frame.width() as f32 / 2.0;
    let fountain_y = frame.height() as f32 / 2.0;
    
    // Draw fountain base
    for radius in 0..40 {
        let color_intensity = 255 - (radius * 5).min(255);
        draw_circle_fast(frame, Position::new(fountain_x, fountain_y), 40 - radius, 
                        Color::new(255, color_intensity as u8, 0));
    }
    
    let blink = ((time * 5.0).sin() * 0.5 + 0.5) * 255.0;
    draw_text_fast(frame, "FOUNTAIN", 
                   Position::new(fountain_x - 200.0, 50.0), 
                   Color::new(255, blink as u8, blink as u8), 2.0);
    
    let particles = 1000;
    for i in 0..particles {
        let lifetime = 2.0; 
        let particle_time = (time + i as f32 * 0.01) % lifetime;
        let progress = particle_time / lifetime;
        
        let angle = std::f32::consts::PI / 2.0 + (i as f32 / particles as f32 - 0.5) * std::f32::consts::PI * 1.5;
        let speed = 600.0 * (1.0 - progress * 0.3);
        let gravity = 800.0;
        
        let x = fountain_x + angle.cos() * speed * particle_time;
        let y = fountain_y - angle.sin() * speed * particle_time + 0.5 * gravity * particle_time * particle_time;
        
        if x >= 0.0 && x < frame.width() as f32 && y >= 0.0 && y < frame.height() as f32 {
            let fade = if progress < 0.1 {
                progress / 0.1
            } else if progress > 0.7 {
                (1.0 - progress) / 0.3
            } else {
                1.0
            };
            
            let hue = (i as f32 / particles as f32 + time * 0.3) % 1.0;
            let color = hsv_to_rgb(hue, 1.0, 1.0);
            let size = 4 + (10.0 * (1.0 - progress)) as i32;
            
            // Draw bright particle
            let glow_radius = size * 3;
            for dy in -glow_radius..=glow_radius {
                for dx in -glow_radius..=glow_radius {
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq > glow_radius * glow_radius { continue; }
                    
                    let distance = (dist_sq as f32).sqrt();
                    let intensity = if distance <= size as f32 {
                        2.0
                    } else if distance <= glow_radius as f32 {
                        1.5 * (1.0 - (distance - size as f32) / (glow_radius as f32 - size as f32))
                    } else {
                        0.0
                    };
                    
                    let r = (intensity * color.red as f32 * fade * 3.0).min(255.0) as u8;
                    let g = (intensity * color.green as f32 * fade * 3.0).min(255.0) as u8;
                    let b = (intensity * color.blue as f32 * fade * 3.0).min(255.0) as u8;
                    
                    frame.blend_pixel(x as i32 + dx, y as i32 + dy, Color::new(r, g, b), 1.0);
                }
            }
        }
    }
    
    // Draw pulsing border
    let pulse = (time * 10.0).sin() > 0.0;
    let border_color = if pulse { Color::new(255, 0, 0) } else { Color::new(255, 255, 255) };
    
    // Draw border
    let border_width = 3;
    for dy in 0..border_width {
        for dx in 0..frame.width() as i32 {
            frame.set_pixel(dx, dy, border_color);
            frame.set_pixel(dx, frame.height() as i32 - 1 - dy, border_color);
        }
    }
    for dx in 0..border_width {
        for dy in 0..frame.height() as i32 {
            frame.set_pixel(dx, dy, border_color);
            frame.set_pixel(frame.width() as i32 - 1 - dx, dy, border_color);
        }
    }
    
    if pulse {
        for y in 0..frame.height() as usize {
            for x in 0..frame.width() as usize {
                if (x + y) % 20 == 0 {
                    frame.set_pixel(x as i32, y as i32, Color::new(255, 255, 0));
                }
            }
        }
    }
}

// Helper function to draw world to buffer
fn draw_world_to_buffer(world: &crate::types::World, buffer: &mut Vec<u8>) {
    // Ensure buffer is the right size
    let expected_size = crate::types::WIDTH as usize * crate::types::HEIGHT as usize * 4;
    if buffer.len() != expected_size {
        buffer.resize(expected_size, 0);
    }
    
    // Draw the world to the buffer
    world.draw(buffer);
}

// Missing functions that app.rs expects
pub fn draw_original_with_buffer(frame: &mut [u8], world: &crate::types::World, buffer: &mut Vec<u8>) {
    // Clear the buffer
    buffer.fill(0);
    
    // Draw the world to buffer
    draw_world_to_buffer(world, buffer);
    
    // Copy buffer to frame (assuming same dimensions)
    let copy_len = frame.len().min(buffer.len());
    frame[..copy_len].copy_from_slice(&buffer[..copy_len]);
}

pub fn draw_circular_with_buffer(frame: &mut [u8], elapsed: f32, buffer: &mut Vec<u8>) {
    use crate::mesmerise_circular;
    
    // Clear the buffer
    buffer.fill(0);
    
    // Draw circular visualization to buffer
    mesmerise_circular::draw_frame(buffer, elapsed);
    
    // Copy buffer to frame
    let copy_len = frame.len().min(buffer.len());
    frame[..copy_len].copy_from_slice(&buffer[..copy_len]);
}

pub fn draw_full_screen_with_buffer(frame: &mut [u8], world: &crate::types::World, elapsed: f32, buffers: &mut crate::types::Buffers) {
    // Clear frame
    frame.fill(0);
    
    // Draw world to left half
    let width = crate::types::WIDTH as usize;
    let height = crate::types::HEIGHT as usize;
    let half_width = width / 2;
    
    // Draw original on left half
    buffers.original.fill(0);
    world.draw(&mut buffers.original);
    
    // Copy left half
    for y in 0..height {
        for x in 0..half_width {
            let src_idx = 4 * (y * (crate::types::ORIGINAL_WIDTH as usize) + x);
            let dst_idx = 4 * (y * width + x);
            
            if src_idx + 3 < buffers.original.len() && dst_idx + 3 < frame.len() {
                frame[dst_idx..dst_idx + 4].copy_from_slice(&buffers.original[src_idx..src_idx + 4]);
            }
        }
    }
    
    // Draw circular on right half
    buffers.circular.fill(0);
    crate::mesmerise_circular::draw_frame(&mut buffers.circular, elapsed);
    
    // Copy right half
    for y in 0..height {
        for x in 0..half_width {
            let src_idx = 4 * (y * (crate::mesmerise_circular::WIDTH as usize) + x);
            let dst_idx = 4 * (y * width + (half_width + x));
            
            if src_idx + 3 < buffers.circular.len() && dst_idx + 3 < frame.len() {
                frame[dst_idx..dst_idx + 4].copy_from_slice(&buffers.circular[src_idx..src_idx + 4]);
            }
        }
    }
}

pub fn draw_all_visualizations(frame: &mut [u8], world: &crate::types::World, elapsed: f32) {
    // Clear frame
    frame.fill(0);
    
    let width = crate::types::WIDTH as usize;
    let height = crate::types::HEIGHT as usize;
    let quarter_width = width / 4;
    let quarter_height = height / 2;
    
    // Create temporary buffers for each visualization
    let mut temp_buffer = vec![0u8; 4 * quarter_width * quarter_height];
    
    // Draw different visualizations in quarters
    // Top left: Original
    temp_buffer.fill(0);
    world.draw(&mut temp_buffer);
    copy_to_quadrant(frame, &temp_buffer, 0, 0, quarter_width, quarter_height, width);
    
    // Top right: Ray pattern
    temp_buffer.fill(0);
    crate::ray_pattern::draw_frame(&mut temp_buffer, quarter_width as u32, quarter_height as u32, elapsed, 0, quarter_width as u32);
    copy_to_quadrant(frame, &temp_buffer, quarter_width, 0, quarter_width, quarter_height, width);
    
    // Bottom left: Fibonacci spiral (need to create wrapper)
    temp_buffer.fill(0);
    draw_fibonacci_to_buffer(&mut temp_buffer, quarter_width, quarter_height, elapsed);
    copy_to_quadrant(frame, &temp_buffer, 0, quarter_height, quarter_width, quarter_height, width);
    
    // Bottom right: Pythagoras (need to create wrapper)
    temp_buffer.fill(0);
    draw_pythagoras_to_buffer(&mut temp_buffer, quarter_width, quarter_height, elapsed);
    copy_to_quadrant(frame, &temp_buffer, quarter_width, quarter_height, quarter_width, quarter_height, width);
}

fn copy_to_quadrant(dst: &mut [u8], src: &[u8], x_offset: usize, y_offset: usize, quad_width: usize, quad_height: usize, dst_width: usize) {
    for y in 0..quad_height {
        for x in 0..quad_width {
            let src_idx = 4 * (y * quad_width + x);
            let dst_idx = 4 * ((y + y_offset) * dst_width + (x + x_offset));
            
            if src_idx + 3 < src.len() && dst_idx + 3 < dst.len() {
                dst[dst_idx..dst_idx + 4].copy_from_slice(&src[src_idx..src_idx + 4]);
            }
        }
    }
}

// Wrapper functions for visualizations that expect Pixels
fn draw_fibonacci_to_buffer(buffer: &mut [u8], width: usize, height: usize, elapsed: f32) {
    // Clear frame with white background
    for pixel in buffer.chunks_exact_mut(4) {
        pixel[0] = 255; // R
        pixel[1] = 255; // G
        pixel[2] = 255; // B
        pixel[3] = 255; // A
    }
    
    // Use elapsed time to add subtle animation effect
    let animation_offset = (elapsed * 0.5).sin() * 5.0;
    
    // Calculate first few Fibonacci numbers
    let mut fibonacci = vec![1, 1];
    for i in 2..12 {
        fibonacci.push(fibonacci[i-1] + fibonacci[i-2]);
    }
    
    // Colors for each square
    let colors = [
        [255, 0, 0, 255],     // Red
        [0, 255, 0, 255],     // Green
        [0, 0, 255, 255],     // Blue
        [255, 255, 0, 255],   // Yellow
        [255, 0, 255, 255],   // Magenta
        [0, 255, 255, 255],   // Cyan
        [255, 128, 0, 255],   // Orange
        [128, 0, 255, 255],   // Purple
        [255, 192, 203, 255], // Pink
        [128, 128, 128, 255], // Gray
        [255, 165, 0, 255],   // Orange
        [75, 0, 130, 255],    // Indigo
    ];
    
    // Draw fibonacci spiral
    let center_x = width as f32 / 2.0 + animation_offset;
    let center_y = height as f32 / 2.0 + animation_offset;
    let scale = 2.0;
    
    let mut x = center_x;
    let mut y = center_y;
    let mut direction = 0; // 0: right, 1: down, 2: left, 3: up
    
    for (i, &fib) in fibonacci.iter().enumerate() {
        let size = fib as f32 * scale;
        let color = colors[i % colors.len()];
        
        // Draw square
        let start_x = match direction {
            0 => x, // right
            1 => x - size, // down
            2 => x - size, // left
            3 => x, // up
            _ => x, // default
        };
        let start_y = match direction {
            0 => y - size, // right
            1 => y, // down
            2 => y, // left
            3 => y - size, // up
            _ => y, // default
        };
        
        // Fill square
        for dy in 0..(size as i32) {
            for dx in 0..(size as i32) {
                let px = (start_x as i32 + dx) as usize;
                let py = (start_y as i32 + dy) as usize;
                
                if px < width && py < height {
                    let idx = 4 * (py * width + px);
                    buffer[idx] = color[0];
                    buffer[idx + 1] = color[1];
                    buffer[idx + 2] = color[2];
                    buffer[idx + 3] = color[3];
                }
            }
        }
        
        // Update position for next square
        match direction {
            0 => x += size, // right
            1 => y += size, // down
            2 => x -= size, // left
            3 => y -= size, // up
            _ => {}, // default - do nothing
        }
        
        // Change direction for spiral
        direction = (direction + 1) % 4;
    }
}

fn draw_pythagoras_to_buffer(buffer: &mut [u8], width: usize, height: usize, elapsed: f32) {
    // Clear frame with light background
    for pixel in buffer.chunks_exact_mut(4) {
        pixel[0] = 240; // R
        pixel[1] = 240; // G
        pixel[2] = 240; // B
        pixel[3] = 255; // A
    }
    
    let a = 60.0f32;
    let b = 80.0f32;
    let c = (a*a + b*b).sqrt();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let angle = elapsed * 0.3;
    
    // Draw squares for a, b, c
    let colors = [
        [255, 100, 100, 255], // Red for a²
        [100, 255, 100, 255], // Green for b²
        [100, 100, 255, 255], // Blue for c²
    ];
    
    let sizes = [a, b, c];
    
    for (i, &size) in sizes.iter().enumerate() {
        let offset_x = match i {
            0 => -size, // a² on left
            1 => 0.0,   // b² in center
            2 => size,  // c² on right
            _ => 0.0,   // default
        };
        
        let half_size = size / 2.0;
        let color = colors[i];
        
        // Draw square
        for dy in -(half_size as i32)..(half_size as i32) {
            for dx in -(half_size as i32)..(half_size as i32) {
                let px = (center_x + offset_x + dx as f32 + angle.cos() * 10.0) as usize;
                let py = (center_y + dy as f32 + (angle * 0.7).sin() * 5.0) as usize;
                
                if px < width && py < height {
                    let idx = 4 * (py * width + px);
                    buffer[idx] = color[0];
                    buffer[idx + 1] = color[1];
                    buffer[idx + 2] = color[2];
                    buffer[idx + 3] = color[3];
                }
            }
        }
    }
    
    // Draw formula text (simplified)
    let formula = format!("a² + b² = c²");
    // For now, just draw some pixels to represent text
    let text_y = (center_y + 100.0) as usize;
    let text_x = (center_x - 50.0) as usize;
    
    if text_y < height && text_x < width {
        // Draw simple text representation
        for i in 0..formula.len() * 8 {
            let px = text_x + i;
            if px < width {
                let idx = 4 * (text_y * width + px);
                buffer[idx] = 0;     // Black text
                buffer[idx + 1] = 0;
                buffer[idx + 2] = 0;
                buffer[idx + 3] = 255;
            }
        }
    }
}

// Public wrapper functions for individual visualizations
pub fn draw_pythagoras_frame(frame: &mut [u8], elapsed: f32) {
    draw_pythagoras_to_buffer(frame, WIDTH as usize, HEIGHT as usize, elapsed);
}

pub fn draw_fibonacci_frame(frame: &mut [u8], elapsed: f32) {
    draw_fibonacci_to_buffer(frame, WIDTH as usize, HEIGHT as usize, elapsed);
}

pub fn draw_simple_proof_frame(frame: &mut [u8], elapsed: f32) {
    // Simple proof visualization - just show a mathematical proof concept
    // Clear frame with dark background
    for pixel in frame.chunks_exact_mut(4) {
        pixel[0] = 20;  // Dark background
        pixel[1] = 20;
        pixel[2] = 40;
        pixel[3] = 255;
    }
    
    // Draw some geometric shapes to represent a proof
    let center_x = WIDTH as f32 / 2.0;
    let center_y = HEIGHT as f32 / 2.0;
    let time_offset = elapsed * 0.5;
    
    // Draw animated circles representing logical steps
    let colors = [
        [255, 100, 100, 255], // Red
        [100, 255, 100, 255], // Green  
        [100, 100, 255, 255], // Blue
        [255, 255, 100, 255], // Yellow
    ];
    
    for i in 0..4 {
        let angle = (i as f32 * std::f32::consts::PI / 2.0) + time_offset;
        let radius = 80.0 + (time_offset + i as f32).sin() * 20.0;
        let x = center_x + angle.cos() * radius;
        let y = center_y + angle.sin() * radius;
        let color = colors[i];
        
        // Draw circle
        let circle_radius = 25;
        for dy in -circle_radius..=circle_radius {
            for dx in -circle_radius..=circle_radius {
                if dx * dx + dy * dy <= circle_radius * circle_radius {
                    let px = (x as i32 + dx) as usize;
                    let py = (y as i32 + dy) as usize;
                    
                    if px < WIDTH as usize && py < HEIGHT as usize {
                        let idx = 4 * (py * WIDTH as usize + px);
                        frame[idx] = color[0];
                        frame[idx + 1] = color[1];
                        frame[idx + 2] = color[2];
                        frame[idx + 3] = color[3];
                    }
                }
            }
        }
    }
}