use macroquad::prelude::*;

const RAYS: usize = 60;  // Number of rays for each source

#[macroquad::main("Ray Clock")]
async fn main() {
    // Set up window with black background
    let circle_radius = 300.0;
    
    loop {
        clear_background(BLACK);
        
        let center_x = screen_width() / 2.0;
        let center_y = screen_height() / 2.0;
        
        // Draw the outer circle (white border)
        draw_circle_lines(center_x, center_y, circle_radius, 2.0, WHITE);
        
        // Calculate positions for the two light sources
        let left_source_x = center_x - circle_radius * 0.4;
        let left_source_y = center_y;
        
        let right_source_x = center_x + circle_radius * 0.4;
        let right_source_y = center_y;
        
        // Draw yellow rays from left source
        draw_rays(left_source_x, left_source_y, circle_radius, YELLOW, RAYS);
        
        // Draw green rays from right source
        draw_rays(right_source_x, right_source_y, circle_radius, GREEN, RAYS);
        
        // Draw the light sources
        draw_circle(left_source_x, left_source_y, 10.0, YELLOW);
        draw_circle(right_source_x, right_source_y, 10.0, GREEN);
        
        // Time display
        let time = get_time();
        let rotation_speed = 0.1;
        
        // Draw a rotating ray to simulate clock hand
        let angle = time * rotation_speed;
        let end_x = center_x + angle.cos() as f32 * circle_radius * 0.9;
        let end_y = center_y + angle.sin() as f32 * circle_radius * 0.9;
        
        draw_line(center_x, center_y, end_x, end_y, 2.0, WHITE);
        
        next_frame().await;
    }
}

fn draw_rays(source_x: f32, source_y: f32, max_radius: f32, color: Color, count: usize) {
    // Add some glow effect to the rays
    let glow_color = Color::new(
        color.r, 
        color.g, 
        color.b, 
        0.7
    );
    
    // Draw rays from the source to the circle edge
    for i in 0..count {
        let angle = (i as f32 / count as f32) * std::f32::consts::PI * 2.0;
        
        // Calculate intersection with the circle
        let dir_x = angle.cos();
        let dir_y = angle.sin();
        
        // Find intersection with circle
        let center_x = screen_width() / 2.0;
        let center_y = screen_height() / 2.0;
        
        // Calculate the ray endpoint using vector from source to circle edge
        let dx = source_x - center_x;
        let dy = source_y - center_y;
        
        // Calculate the vector from source to center
        let to_center_x = center_x - source_x;
        let to_center_y = center_y - source_y;
        
        // Project ray direction onto circle
        let ray_end_x = center_x + dir_x * max_radius;
        let ray_end_y = center_y + dir_y * max_radius;
        
        // Draw the ray with a slight glow effect
        for width in (0..3).rev() {
            let alpha = if width == 0 { 1.0 } else { 0.3 / width as f32 };
            let ray_color = Color::new(color.r, color.g, color.b, alpha);
            let thickness = 1.0 + width as f32 * 0.5;
            draw_line(source_x, source_y, ray_end_x, ray_end_y, thickness, ray_color);
        }
    }
} 