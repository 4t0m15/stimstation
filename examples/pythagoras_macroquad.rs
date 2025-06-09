use macroquad::prelude::*;

#[macroquad::main("Pythagorean Dissection")]
async fn main() {
    let mut angle = 0.0f32;
    loop {
        clear_background(WHITE);

        // Parameters
        let a = 100.0;
        let b = 150.0;
        let c = (a*a + b*b).sqrt();
        let center = vec2(screen_width()/2.0, screen_height()/2.0);

        // Draw big square (c × c)
        draw_rectangle(
            center.x - c/2.0, center.y - c/2.0, 
            c, c, 
            LIGHTGRAY
        );

        // Four triangles rotating
        for i in 0..4 {
            let theta = angle + i as f32 * std::f32::consts::FRAC_PI_2;
            let p1 = center + vec2(theta.cos(), theta.sin()) * (c/2.0);
            let p2 = center + vec2((theta + b.to_radians()).cos(), (theta + b.to_radians()).sin()) * (a/2.0);
            let p3 = center + vec2((theta - a.to_radians()).cos(), (theta - a.to_radians()).sin()) * (b/2.0);
            draw_triangle(p1, p2, p3, BLUE);
        }

        angle += 0.01;
        next_frame().await;
    }
} 