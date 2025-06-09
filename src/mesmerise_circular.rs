use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    keyboard::KeyCode,
    event_loop::{EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;
use std::time::{Duration, Instant};

/// Canvas size (split into three vertical regions)
pub const WIDTH: u32 = 900;
pub const HEIGHT: u32 = 300;

fn main() {
    // Set up window and input handling
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();

    let window = WindowBuilder::new()
        .with_title("Mesmerise - Final with Circular Visuals")
        .with_inner_size(LogicalSize::new(WIDTH as f64, HEIGHT as f64))
        .with_min_inner_size(LogicalSize::new(WIDTH as f64, HEIGHT as f64))
        .build(&event_loop)
        .unwrap();

    // Set up pixel buffer
    let mut pixels = {
        let size = window.inner_size();
        let surface = SurfaceTexture::new(size.width, size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface).unwrap()
    };

    let start_time = Instant::now();
    let mut last_frame = Instant::now();
    let target_frame_time = Duration::from_secs_f32(1.0 / 60.0);

    event_loop.run(move |event, window_target| {
        // Default control flow is Poll
        if input.update(&event) {
            // Exit on Escape or window close
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                window_target.exit();
                return;
            }

            // Handle window resize
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height).unwrap();
            }
        }

        // Draw at ~60 FPS
        if last_frame.elapsed() >= target_frame_time {
            let t = start_time.elapsed().as_secs_f32();
            draw_frame(pixels.frame_mut(), t);
            if pixels.render().is_err() {
                window_target.exit();
                return;
            }
            last_frame = Instant::now();
        }
    }).unwrap(); // Unwrap the result of run
}

/// Composite drawing routine splits the buffer into three regions.
pub fn draw_frame(frame: &mut [u8], t: f32) {
    let w = WIDTH as usize;
    let h = HEIGHT as usize;
    let region_width = w / 3;

    // Clear to opaque black
    for px in frame.chunks_exact_mut(4) {
        px[0] = 0;
        px[1] = 0;
        px[2] = 0;
        px[3] = 255;
    }

    // Region 0: Concentric rainbow circles
    draw_concentric_region(frame, 0, region_width, w, h, t);

    // Region 1: Orbiting dots (spiral / radar)
    draw_orbit_region(frame, region_width, region_width, w, h, t);

    // Region 2: Radial fan of beams
    draw_beams_region(frame, 2 * region_width, region_width, w, h, t);
}

/// Region 0: concentric, rainbow‐colored circles
pub fn draw_concentric_region(
    frame: &mut [u8],
    x_off: usize,
    region_w: usize,
    w: usize,
    h: usize,
    t: f32,
) {
    let _cy = h / 2; // Marked as unused with underscore
    let radius = (region_w.min(h) as f32) * 0.45;

    for py in 0..h {
        for px in 0..region_w {
            let dx = px as f32 - region_w as f32 / 2.0;
            let dy = py as f32 - h as f32 / 2.0;
            let dist = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx);

            let r = (angle * 6.0 + t).sin() * 0.5 + 0.5;
            let g = (angle * 4.0 + t * 1.2).sin() * 0.5 + 0.5;
            let b = (angle * 8.0 + t * 0.8).sin() * 0.5 + 0.5;

            let alpha = if dist <= radius {
                1.0
            } else {
                1.0 - ((dist - radius) / (radius * 0.4)).clamp(0.0, 1.0)
            };

            let x = x_off + px;
            let idx = 4 * (py * w + x);
            frame[idx]     = (255.0 * r * alpha) as u8;
            frame[idx + 1] = (255.0 * g * alpha) as u8;
            frame[idx + 2] = (255.0 * b * alpha) as u8;
            frame[idx + 3] = 255;
        }
    }
}

/// Region 1: a few dots orbiting in a circle
pub fn draw_orbit_region(
    frame: &mut [u8],
    x_off: usize,
    region_w: usize,
    w: usize,
    h: usize,
    t: f32,
) {
    let cx = x_off + region_w / 2;
    let cy = h / 2;
    let radius = (region_w.min(h) as f32) * 0.4;
    let count = 5;
    let size = 3;

    for i in 0..count {
        let phase = t * 1.2 + (i as f32 / count as f32) * std::f32::consts::TAU;
        let sx = (cx as f32 + radius * phase.cos()) as i32;
        let sy = (cy as f32 + radius * phase.sin()) as i32;

        for oy in -size..=size {
            for ox in -size..=size {
                let px = sx + ox;
                let py = sy + oy;
                if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                    let idx = 4 * (py as usize * w + px as usize);
                    frame[idx]     = 50;
                    frame[idx + 1] = 200;
                    frame[idx + 2] = 50;
                    frame[idx + 3] = 255;
                }
            }
        }
    }
}

/// Region 2: radial white beams spinning slowly
pub fn draw_beams_region(
    frame: &mut [u8],
    x_off: usize,
    region_w: usize,
    w: usize,
    h: usize,
    t: f32,
) {
    let cx = x_off + region_w / 2;
    let cy = h / 2;
    let beams = 16;
    let length = (region_w.min(h) as f32) * 0.45;

    for k in 0..beams {
        let angle = k as f32 * (std::f32::consts::TAU / beams as f32) + t * 0.2;
        let dx = angle.cos();
        let dy = angle.sin();
        let mut x = cx as f32;
        let mut y = cy as f32;

        for _ in 0..(length as usize) {
            let ix = x as i32;
            let iy = y as i32;
            if ix >= 0 && ix < w as i32 && iy >= 0 && iy < h as i32 {
                let idx = 4 * (iy as usize * w + ix as usize);
                frame[idx]     = 255;
                frame[idx + 1] = 255;
                frame[idx + 2] = 255;
                frame[idx + 3] = 255;
            }
            x += dx;
            y += dy;
        }
    }
}
