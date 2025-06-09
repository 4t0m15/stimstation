// A simple ray pattern generator that creates an image similar to the reference
// Uses only the image crate for minimal dependencies

use std::path::Path;
use image::{ImageBuffer, Rgb};

fn main() {
    // Create a new image
    let width = 800;
    let height = 800;
    let mut img = ImageBuffer::new(width, height);
    
    // Fill the image with black
    for pixel in img.pixels_mut() {
        *pixel = Rgb([0, 0, 0]);
    }
    
    // Parameters
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let radius = 350.0;
    let ray_count = 100;
    
    // Draw white circle outline
    draw_circle_outline(&mut img, center_x as u32, center_y as u32, radius as u32, Rgb([255, 255, 255]));
    
    // Position for yellow source (left side)
    let yellow_x = center_x - radius * 0.4;
    let yellow_y = center_y;
    
    // Position for green source (right side)
    let green_x = center_x + radius * 0.4;
    let green_y = center_y;
    
    // Draw yellow rays
    draw_rays(&mut img, yellow_x as u32, yellow_y as u32, ray_count, radius as u32, Rgb([255, 255, 0]));
    
    // Draw green rays
    draw_rays(&mut img, green_x as u32, green_y as u32, ray_count, radius as u32, Rgb([0, 255, 0]));
    
    // Draw the source points
    draw_filled_circle(&mut img, yellow_x as u32, yellow_y as u32, 8, Rgb([255, 255, 0]));
    draw_filled_circle(&mut img, green_x as u32, green_y as u32, 8, Rgb([0, 255, 0]));
    
    // Save the image
    img.save(Path::new("ray_pattern.png")).unwrap();
    println!("Image saved as ray_pattern.png");
}

// Draw rays from a source point to the edge of the circle
fn draw_rays(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, 
             source_x: u32, source_y: u32, 
             count: usize, radius: u32, 
             color: Rgb<u8>) {
    
    let width = img.width();
    let height = img.height();
    let center_x = width / 2;
    let center_y = height / 2;
    
    for i in 0..count {
        let angle = (i as f32 / count as f32) * 2.0 * std::f32::consts::PI;
        
        // Calculate endpoint on circle
        let end_x = center_x as f32 + angle.cos() * radius as f32;
        let end_y = center_y as f32 + angle.sin() * radius as f32;
        
        // Draw line from source to endpoint
        draw_line(img, source_x, source_y, end_x as u32, end_y as u32, color);
    }
}

// Draw a line using Bresenham's algorithm
fn draw_line(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, 
             x0: u32, y0: u32, 
             x1: u32, y1: u32, 
             color: Rgb<u8>) {
    
    let width = img.width();
    let height = img.height();
    
    // Convert to signed integers for the algorithm
    let (mut x0, mut y0, x1, y1) = (x0 as i32, y0 as i32, x1 as i32, y1 as i32);
    
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    
    loop {
        // Only draw if within bounds
        if x0 >= 0 && x0 < width as i32 && y0 >= 0 && y0 < height as i32 {
            img.put_pixel(x0 as u32, y0 as u32, color);
        }
        
        if x0 == x1 && y0 == y1 { break; }
        
        let e2 = 2 * err;
        if e2 >= dy {
            if x0 == x1 { break; }
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            if y0 == y1 { break; }
            err += dx;
            y0 += sy;
        }
    }
}

// Draw a circle outline
fn draw_circle_outline(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, 
                      center_x: u32, center_y: u32, 
                      radius: u32, 
                      color: Rgb<u8>) {
    
    let width = img.width();
    let height = img.height();
    
    for y in 0..height {
        for x in 0..width {
            let dx = x as i32 - center_x as i32;
            let dy = y as i32 - center_y as i32;
            let distance = ((dx * dx + dy * dy) as f32).sqrt();
            
            // Draw pixels close to the circle radius
            if (distance - radius as f32).abs() < 1.0 {
                img.put_pixel(x, y, color);
            }
        }
    }
}

// Draw a filled circle
fn draw_filled_circle(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, 
                     center_x: u32, center_y: u32, 
                     radius: u32, 
                     color: Rgb<u8>) {
    
    let width = img.width();
    let height = img.height();
    
    for y in 0..height {
        for x in 0..width {
            let dx = x as i32 - center_x as i32;
            let dy = y as i32 - center_y as i32;
            let distance = ((dx * dx + dy * dy) as f32).sqrt();
            
            if distance <= radius as f32 {
                img.put_pixel(x, y, color);
            }
        }
    }
} 